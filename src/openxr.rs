use std::borrow::Cow;

use bevy::{prelude::*, utils::HashMap};
use bevy_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::OxrActionSetSyncSet,
    helper_traits::ToTransform,
    resources::{OxrFrameState, OxrInstance, Pipelined},
    session::OxrSession,
    spaces::OxrSpaceSyncSet,
};
use bevy_xr::{
    session::{session_available, session_running, status_changed_to, XrSessionCreated, XrStatus},
    spaces::{XrPrimaryReferenceSpace, XrReferenceSpace, XrSpace},
    types::XrPose,
};
use openxr::SpaceLocationFlags;

use crate::{
    ActionName, ActionSet, ActionSetName, BoolActionValue, F32ActionValue, LocalizedActionName,
    LocalizedActionSetName, SchminputSet, Vec2ActionValue,
};

pub const OCULUS_TOUCH_PROFILE: &str = "/interaction_profiles/oculus/touch_controller";
pub const META_TOUCH_PRO_PROFILE: &str = "/interaction_profiles/meta/touch_pro_controller";
pub const META_TOUCH_PLUS_PROFILE: &str = "/interaction_profiles/meta/touch_plus_controller";

impl Plugin for OxrInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                sync_action_sets.before(OxrActionSetSyncSet),
                sync_input_actions.after(OxrActionSetSyncSet),
            )
                .chain()
                .run_if(session_running)
                .in_set(SchminputSet::SyncInputActions)
                .before(OxrSpaceSyncSet),
        );
        app.add_systems(
            XrSessionCreated,
            attach_action_sets.run_if(status_changed_to(XrStatus::Ready)),
        );
        app.add_systems(OxrSendActionBindings, suggest_bindings);
        app.add_systems(PostStartup, create_input_actions.run_if(session_available));
    }
}

fn sync_action_sets(query: Query<&OxrActionSet>, session: Res<OxrSession>) {
    let sets = query
        .iter()
        .map(|set| openxr::ActiveActionSet::new(set))
        .collect::<Vec<_>>();
    let result = session.sync_actions(&sets);
    if let Err(err) = result {
        warn!("Unable to sync action sets: {}", err.to_string())
    }
}

fn attach_action_sets(query: Query<&OxrActionSet>, mut suggest: EventWriter<OxrAttachActionSet>) {
    for set in &query {
        suggest.send(OxrAttachActionSet(set.0.clone()));
    }
}

fn suggest_bindings(
    query: Query<(&OxrActionBlueprint, &OxrAction, Entity), Without<BindingsSuggested>>,
    mut suggest: EventWriter<OxrSuggestActionBinding>,
    mut cmds: Commands,
) {
    // panic!("suggesting bindings");
    for (blueprint, action, entity) in &query {
        for (profile, bindings) in blueprint.bindings.iter() {
            suggest.send(OxrSuggestActionBinding {
                action: action.as_raw(),
                interaction_profile: Cow::from(*profile),
                bindings: bindings.iter().map(|b| (*b).into()).collect(),
            });
        }
        cmds.entity(entity).insert(BindingsSuggested);
    }
}

#[allow(clippy::type_complexity)]
fn create_input_actions(
    mut cmds: Commands,
    query: Query<(
        Entity,
        &ActionSet,
        &ActionName,
        Option<&LocalizedActionName>,
        Has<BoolActionValue>,
        Has<Vec2ActionValue>,
        Has<F32ActionValue>,
        Has<PoseActionValue>,
        Has<SetPoseOfEntity>,
    )>,
    action_set_query: Query<(&ActionSetName, Option<&LocalizedActionSetName>)>,
    instance: Res<OxrInstance>,
) {
    let mut set_map: HashMap<Entity, openxr::ActionSet> = HashMap::new();
    for (
        entity,
        action_set,
        action_id,
        action_name,
        has_bool,
        has_vec2,
        has_f32,
        has_pose,
        has_set_pose,
    ) in &query
    {
        let Ok((set_id, set_name)) = action_set_query.get(action_set.0) else {
            error!("OpenXR action has an invalid Action Set at Setup!");
            continue;
        };
        let action_name: &str = action_name.map(|v| &v.0).unwrap_or(&action_id.0);
        let set_name: &str = set_name.map(|v| &v.0).unwrap_or(&set_id.0);
        let action_set = set_map
            .entry(action_set.0)
            .or_insert_with(|| instance.create_action_set(set_id, set_name, 0).unwrap());
        let action = match (has_bool, has_f32, has_vec2, has_pose || has_set_pose) {
            (true, false, false, false) => OxrAction::Bool(
                action_set
                    .create_action(action_id, action_name, &[])
                    .unwrap(),
            ),
            (false, true, false, false) => OxrAction::F32(
                action_set
                    .create_action(action_id, action_name, &[])
                    .unwrap(),
            ),
            (false, false, true, false) => OxrAction::Vec2(
                action_set
                    .create_action(action_id, action_name, &[])
                    .unwrap(),
            ),
            (false, false, false, true) => OxrAction::Pose(
                action_set
                    .create_action(action_id, action_name, &[])
                    .unwrap(),
                None,
            ),
            (false, false, false, false) => {
                error!("OpenXR action has no ActionValue!");
                continue;
            }

            _ => {
                error!("OpenXR action has to many ActionValues!");
                continue;
            }
        };
        cmds.entity(entity).insert(action);
    }
    for (e, set) in set_map.into_iter() {
        cmds.entity(e).insert(OxrActionSet(set));
    }
}

#[allow(clippy::type_complexity)]
fn sync_input_actions(
    session: Res<OxrSession>,
    mut query: Query<(
        &mut OxrAction,
        Option<&mut BoolActionValue>,
        Option<&mut F32ActionValue>,
        Option<&mut Vec2ActionValue>,
        Option<&mut PoseActionValue>,
        Option<&mut SetPoseOfEntity>,
        Option<&mut SpaceActionValue>,
        Option<&XrReferenceSpace>,
    )>,
    // mut transform_query: Query<&mut Transform>,
    primary_ref_space: Res<XrPrimaryReferenceSpace>,
    frame_state: Res<OxrFrameState>,
    pipelined: Option<Res<Pipelined>>,
    mut cmds: Commands,
) {
    let time = openxr::Time::from_nanos(
        frame_state.predicted_display_time.as_nanos()
            + (frame_state.predicted_display_period.as_nanos() * (pipelined.is_some() as i64)),
    );
    for (mut action, bool_val, f32_val, vec2_val, pose_val, pos_on_entity, space_val, ref_space) in
        &mut query
    {
        match action.as_mut() {
            OxrAction::Bool(action) => {
                match action.state(&session, openxr::Path::NULL) {
                    Ok(v) => {
                        if let Some(mut val) = bool_val {
                            val.0 |= v.current_state;
                        } else {
                            warn!("Bool action but no bool Value!");
                        }
                    }
                    Err(e) => warn!("unable to get data from action: {}", e.to_string()),
                };
            }
            OxrAction::F32(action) => {
                match action.state(&session, openxr::Path::NULL) {
                    Ok(v) => {
                        if let Some(mut val) = f32_val {
                            val.0 += v.current_state;
                        } else {
                            warn!("F32 action but no f32 Value!");
                        }
                    }
                    Err(e) => warn!("unable to get data from action: {}", e.to_string()),
                };
            }
            OxrAction::Vec2(action) => {
                match action.state(&session, openxr::Path::NULL) {
                    Ok(v) => {
                        if let Some(mut val) = vec2_val {
                            val.0.x += v.current_state.x;
                            val.0.y += v.current_state.y;
                        } else {
                            warn!("Vec2 action but no Vec2 Value!");
                        }
                    }
                    Err(e) => warn!("unable to get data from action: {}", e.to_string()),
                };
            }
            OxrAction::Pose(action, space) => {
                let ref_space = match ref_space {
                    Some(s) => &s.0,
                    None => &primary_ref_space.0 .0,
                };
                let space = match space {
                    Some(s) => s,
                    None => {
                        match session.create_action_space(
                            action,
                            openxr::Path::NULL,
                            XrPose::IDENTITY,
                        ) {
                            Ok(s) => {
                                space.replace(s);
                            }
                            Err(e) => {
                                warn!("unable to create space from action: {}", e.to_string());
                                continue;
                            }
                        };
                        space.as_mut().expect("Should be impossible to hit")
                    }
                };
                if let Some(mut val) = pose_val {
                    let location = match session.locate_space(space, ref_space, time) {
                        Ok(pose) => pose,
                        Err(e) => {
                            warn!("Unable to Locate Action Space: {}", e.to_string());
                            continue;
                        }
                    };
                    if !location.location_flags.contains(
                        SpaceLocationFlags::POSITION_VALID | SpaceLocationFlags::ORIENTATION_VALID,
                    ) {
                        warn!("Pose has invalid Position and or Orientation, skipping");
                        continue;
                    }
                    val.0 = location.pose.to_transform();
                }
                if let Some(e) = pos_on_entity {
                    cmds.entity(e.0).insert(*space);
                }
                if let Some(mut e) = space_val {
                    e.0 = Some(*space);
                }
            }
            OxrAction::Haptic(_) => warn!("Haptic Unimplemented"),
        }
    }
}

#[derive(Component, Default)]
pub struct OxrActionBlueprint {
    bindings: HashMap<&'static str, Vec<&'static str>>,
}

impl OxrActionBlueprint {
    pub fn interaction_profile(self, profile: &'static str) -> OxrActionDeviceBindingBuilder {
        OxrActionDeviceBindingBuilder {
            builder: self,
            curr_interaction_profile: profile,
        }
    }
}

pub struct OxrActionDeviceBindingBuilder {
    builder: OxrActionBlueprint,
    curr_interaction_profile: &'static str,
}
impl OxrActionDeviceBindingBuilder {
    pub fn binding(mut self, binding: &'static str) -> Self {
        self.builder
            .bindings
            .entry(self.curr_interaction_profile)
            .or_default()
            .push(binding);
        self
    }

    pub fn end(self) -> OxrActionBlueprint {
        self.builder
    }
}

#[derive(Component, DerefMut, Deref, Clone, Copy)]
pub struct SetPoseOfEntity(pub Entity);

#[derive(Component, DerefMut, Deref, Clone, Copy)]
pub struct PoseActionValue(pub Transform);

#[derive(Component, DerefMut, Deref, Clone, Copy)]
pub struct SpaceActionValue(pub Option<XrSpace>);

#[derive(Component)]
pub enum OxrAction {
    Bool(openxr::Action<bool>),
    F32(openxr::Action<f32>),
    Vec2(openxr::Action<openxr::Vector2f>),
    Pose(openxr::Action<openxr::Posef>, Option<XrSpace>),
    Haptic(openxr::Action<openxr::Haptic>),
}

impl OxrAction {
    fn as_raw(&self) -> openxr::sys::Action {
        match self {
            OxrAction::Bool(a) => a.as_raw(),
            OxrAction::F32(a) => a.as_raw(),
            OxrAction::Vec2(a) => a.as_raw(),
            OxrAction::Pose(a, _) => a.as_raw(),
            OxrAction::Haptic(a) => a.as_raw(),
        }
    }
}

#[derive(Clone, Copy, Component)]
pub struct BindingsSuggested;

#[derive(Component, Deref)]
pub struct OxrActionSet(pub openxr::ActionSet);

pub struct OxrInputPlugin;
