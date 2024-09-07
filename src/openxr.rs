use std::borrow::Cow;

use bevy::{prelude::*, utils::HashMap};
use bevy_mod_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::{OxrActionSetSyncSet, OxrSyncActionSet},
    helper_traits::ToVec2 as _,
    resources::OxrInstance,
    session::OxrSession,
    spaces::OxrSpaceSyncSet,
};
use bevy_mod_xr::{
    session::{session_available, session_running, XrPreSessionEnd, XrSessionCreated},
    spaces::XrSpace,
    types::XrPose,
};

use crate::{
    binding_modification::{BindingModifiactions, PremultiplyDeltaTimeSecondsModification},
    subaction_paths::{RequestedSubactionPaths, SubactionPathMap, SubactionPathStr},
    ActionName, ActionSetEnabled, ActionSetName, BoolActionValue, F32ActionValue, InActionSet,
    LocalizedActionName, LocalizedActionSetName, SchminputSet, Vec2ActionValue,
};

pub const OCULUS_TOUCH_PROFILE: &str = "/interaction_profiles/oculus/touch_controller";
pub const META_TOUCH_PRO_PROFILE: &str = "/interaction_profiles/facebook/touch_controller_pro";
pub const META_TOUCH_PLUS_PROFILE: &str = "/interaction_profiles/meta/touch_controller_plus";

impl Plugin for OxrInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                sync_action_sets.before(OxrActionSetSyncSet),
                sync_input_actions.after(OxrActionSetSyncSet),
                attach_spaces_to_target_entities,
            )
                .chain()
                .run_if(session_running)
                .in_set(SchminputSet::SyncInputActions)
                .before(OxrSpaceSyncSet),
        );
        app.add_systems(
            XrSessionCreated,
            (create_input_actions, attach_action_sets).chain(),
        );
        app.add_systems(OxrSendActionBindings, suggest_bindings);
        app.add_systems(
            PreUpdate,
            insert_xr_subaction_paths.run_if(session_available),
        );
        app.add_systems(XrPreSessionEnd, reset_space_values);
        app.add_systems(XrPreSessionEnd, clean_actions);
        app.add_systems(XrPreSessionEnd, clean_action_sets);
        app.add_systems(XrPreSessionEnd, test);
    }
}

fn test(query: Query<Entity, With<XrSpace>>, mut cmds: Commands) {
    for e in &query {
        cmds.entity(e).remove::<XrSpace>();
    }
}

fn clean_action_sets(query: Query<Entity, With<OxrActionSet>>, mut cmds: Commands) {
    for e in &query {
        cmds.entity(e).remove::<OxrActionSet>();
    }
}
fn clean_actions(query: Query<Entity, With<OxrAction>>, mut cmds: Commands) {
    for e in &query {
        cmds.entity(e)
            .remove::<OxrAction>()
            .remove::<BindingsSuggested>();
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Component)]
struct NonOxrSubationPath;

fn attach_spaces_to_target_entities(
    query: Query<(&AttachSpaceToEntity, &SpaceActionValue)>,
    check_query: Query<Has<XrSpace>>,
    mut cmds: Commands,
) {
    for (target, value) in query.iter() {
        let Some(space) = value.any else {
            info!("no space to attach to entity");
            continue;
        };
        if !check_query
            .get(target.0)
            .expect("target entity should exist")
        {
            info!("attaching space!");
            cmds.entity(target.0).insert(space);
        }
    }
}

fn reset_space_values(mut query: Query<&mut SpaceActionValue>) {
    for mut s in query.iter_mut() {
        s.any = None;
        s.paths.clear();
    }
}

#[derive(Component, Clone)]
pub struct IsOxrSubactionPath;

#[derive(Component, Clone)]
pub struct OxrSubactionPath(pub openxr::Path);

fn insert_xr_subaction_paths(
    query: Query<
        (Entity, &SubactionPathStr),
        (Without<IsOxrSubactionPath>, Without<NonOxrSubationPath>),
    >,
    mut cmds: Commands,
    instance: Res<OxrInstance>,
) {
    for (e, path) in &query {
        if let Some(xr_path) = path.0.strip_prefix("/oxr") {
            cmds.entity(e).insert(IsOxrSubactionPath);
            if xr_path.is_empty() || xr_path == "/*" {
                continue;
            }
            cmds.entity(e)
                .insert(OxrSubactionPath(match instance.string_to_path(xr_path) {
                    Ok(v) => v,
                    Err(err) => {
                        error!("can't convert ({}) to openxr path: {}", xr_path, err);
                        continue;
                    }
                }));
        } else {
            cmds.entity(e).insert(NonOxrSubationPath);
        }
    }
}

fn sync_action_sets(
    query: Query<(&OxrActionSet, &ActionSetEnabled)>,
    mut sync_set: EventWriter<OxrSyncActionSet>,
) {
    let sets = query
        .iter()
        .filter(|(_, v)| v.0)
        .map(|(set, _)| OxrSyncActionSet(set.0.clone()));
    sync_set.send_batch(sets);
    // let result = session.sync_actions(sets);
    // if let Err(err) = result {
    //     warn!("Unable to sync action sets: {}", err.to_string())
    // }
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
    for (blueprint, action, entity) in &query {
        for (profile, bindings) in blueprint.bindings.iter() {
            suggest.send(OxrSuggestActionBinding {
                action: action.as_raw(),
                interaction_profile: profile.clone(),
                bindings: bindings.clone(),
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
        &InActionSet,
        &ActionName,
        Option<&LocalizedActionName>,
        &RequestedSubactionPaths,
        Has<BoolActionValue>,
        Has<Vec2ActionValue>,
        Has<F32ActionValue>,
        Has<SpaceActionValue>,
    )>,
    path_query: Query<&OxrSubactionPath>,
    action_set_query: Query<(&ActionSetName, Option<&LocalizedActionSetName>)>,
    instance: Res<OxrInstance>,
) {
    let mut set_map: HashMap<Entity, openxr::ActionSet> = HashMap::new();
    for (
        entity,
        action_set,
        action_id,
        action_name,
        requested_subaction_paths,
        has_bool,
        has_vec2,
        has_f32,
        has_space,
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

        let paths = requested_subaction_paths
            .iter()
            .filter_map(|p| path_query.get(p.0).ok())
            .map(|p| p.0)
            .collect::<Vec<_>>();
        let action = match (has_bool, has_f32, has_vec2, has_space) {
            (true, false, false, false) => OxrAction::Bool(
                action_set
                    .create_action(action_id, action_name, &paths)
                    .unwrap(),
            ),
            (false, true, false, false) => OxrAction::F32(
                action_set
                    .create_action(action_id, action_name, &paths)
                    .unwrap(),
            ),
            (false, false, true, false) => OxrAction::Vec2(
                action_set
                    .create_action(action_id, action_name, &paths)
                    .unwrap(),
            ),
            (false, false, false, true) => OxrAction::Space(
                action_set
                    .create_action(action_id, action_name, &paths)
                    .unwrap(),
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
        Option<&mut SpaceActionValue>,
        &RequestedSubactionPaths,
        &BindingModifiactions,
    )>,
    path_query: Query<&OxrSubactionPath>,
    simple_path_query: Query<Has<IsOxrSubactionPath>>,
    modification_query: Query<Has<PremultiplyDeltaTimeSecondsModification>>,
    time: Res<Time>,
) {
    for (
        mut action,
        mut bool_val,
        mut f32_val,
        mut vec2_val,
        mut space_val,
        requested_subaction_paths,
        modifications,
    ) in &mut query
    {
        let paths = requested_subaction_paths
            .iter()
            .filter_map(|p| Some((*p, path_query.get(p.0).ok()?)))
            .map(|(sub_path, path)| (sub_path, path.0))
            .collect::<Vec<_>>();
        let mut pre_mul_delta_time = modifications
            .all_paths
            .as_ref()
            .and_then(|v| modification_query.get(v.0).ok())
            .unwrap_or_default();
        for (_, modification) in modifications
            .per_path
            .iter()
            .filter(|(p, _)| simple_path_query.get(p.0).unwrap_or(false))
        {
            pre_mul_delta_time |= modification_query.get(modification.0).unwrap_or(false);
        }
        let delta_multiplier = match pre_mul_delta_time {
            true => time.delta_seconds(),
            false => 1.0,
        };
        match action.as_mut() {
            OxrAction::Bool(action) => {
                match action.state(&session, openxr::Path::NULL) {
                    Ok(v) => {
                        if let Some(val) = bool_val.as_mut() {
                            val.any |= v.current_state;
                        } else {
                            warn!("Bool action but no bool Value!");
                        }
                    }
                    Err(e) => warn!("unable to get data from action: {}", e.to_string()),
                };
                for (sub_action_path, path) in paths.into_iter() {
                    match action.state(&session, path) {
                        Ok(v) => {
                            if let Some(val) = bool_val.as_mut() {
                                *val.entry_with_path(sub_action_path).or_default() |=
                                    v.current_state;
                            } else {
                                warn!("Bool action but no bool Value!");
                            }
                        }
                        Err(e) => warn!("unable to get data from action: {}", e.to_string()),
                    };
                }
            }
            OxrAction::F32(action) => {
                match action.state(&session, openxr::Path::NULL) {
                    Ok(v) => {
                        if let Some(val) = f32_val.as_mut() {
                            val.any += v.current_state * delta_multiplier;
                        } else {
                            warn!("F32 action but no f32 Value!");
                        }
                    }
                    Err(e) => warn!("unable to get data from action: {}", e.to_string()),
                };
                for (sub_action_path, path) in paths.into_iter() {
                    match action.state(&session, path) {
                        Ok(v) => {
                            if let Some(val) = f32_val.as_mut() {
                                *val.entry_with_path(sub_action_path).or_default() +=
                                    v.current_state * delta_multiplier;
                            } else {
                                warn!("F32 action but no f32 Value!");
                            }
                        }
                        Err(e) => warn!("unable to get data from action: {}", e.to_string()),
                    };
                }
            }
            OxrAction::Vec2(action) => {
                match action.state(&session, openxr::Path::NULL) {
                    Ok(v) => {
                        if let Some(val) = vec2_val.as_mut() {
                            // This might be broken!
                            val.any += v.current_state.to_vec2() * delta_multiplier;
                        } else {
                            warn!("Vec2 action but no Vec2 Value!");
                        }
                    }
                    Err(e) => warn!("unable to get data from action: {}", e.to_string()),
                };
                for (sub_action_path, path) in paths.into_iter() {
                    match action.state(&session, path) {
                        Ok(v) => {
                            if let Some(val) = vec2_val.as_mut() {
                                // This might be broken!
                                *val.entry_with_path(sub_action_path).or_default() +=
                                    v.current_state.to_vec2() * delta_multiplier;
                            } else {
                                warn!("Vec2 action but no Vec2 Value!");
                            }
                        }
                        Err(e) => warn!("unable to get data from action: {}", e.to_string()),
                    };
                }
            }
            // TODO: Add support for XrPose offets (per subaction path?)
            OxrAction::Space(action) => {
                if let Some(val) = space_val.as_mut() {
                    if val.is_none() {
                        match session.create_action_space(
                            action,
                            openxr::Path::NULL,
                            XrPose::IDENTITY,
                        ) {
                            Ok(s) => {
                                val.replace(s);
                            }
                            Err(e) => {
                                warn!("unable to create space from action: {}", e);
                                continue;
                            }
                        };
                    }
                    for (sub_path, path) in paths.into_iter() {
                        if val
                            .get_with_path(&sub_path)
                            .and_then(|v| v.as_ref())
                            .is_none()
                        {
                            match session.create_action_space(action, path, XrPose::IDENTITY) {
                                Ok(s) => {
                                    val.set_value_for_path(sub_path, Some(s));
                                }
                                Err(e) => {
                                    warn!("unable to create space from action: {}", e);
                                    continue;
                                }
                            };
                        }
                    }
                } else {
                    warn!("Space action but no Space Value!");
                }
            }
            OxrAction::Haptic(_) => warn!("Haptic Unimplemented"),
        }

        info!("finnished action value read");
    }
}

#[derive(Component, DerefMut, Deref, Clone, Copy)]
pub struct AttachSpaceToEntity(pub Entity);

#[derive(Component, Default)]
pub struct OxrActionBlueprint {
    pub bindings: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
}

impl OxrActionBlueprint {
    pub fn interaction_profile(
        self,
        profile: impl Into<Cow<'static, str>>,
    ) -> OxrActionDeviceBindingBuilder {
        OxrActionDeviceBindingBuilder {
            builder: self,
            curr_interaction_profile: profile.into(),
        }
    }
}

pub struct OxrActionDeviceBindingBuilder {
    builder: OxrActionBlueprint,
    curr_interaction_profile: Cow<'static, str>,
}
impl OxrActionDeviceBindingBuilder {
    pub fn binding(mut self, binding: impl Into<Cow<'static, str>>) -> Self {
        self.builder
            .bindings
            .entry(self.curr_interaction_profile.clone())
            .or_default()
            .push(binding.into());
        self
    }

    pub fn end(self) -> OxrActionBlueprint {
        self.builder
    }
}

#[derive(Component, DerefMut, Deref, Clone, Default)]
pub struct SpaceActionValue(pub SubactionPathMap<Option<XrSpace>>);

#[derive(Component)]
pub enum OxrAction {
    Bool(openxr::Action<bool>),
    F32(openxr::Action<f32>),
    Vec2(openxr::Action<openxr::Vector2f>),
    Space(openxr::Action<openxr::Posef>),
    Haptic(openxr::Action<openxr::Haptic>),
}

impl OxrAction {
    fn as_raw(&self) -> openxr::sys::Action {
        match self {
            OxrAction::Bool(a) => a.as_raw(),
            OxrAction::F32(a) => a.as_raw(),
            OxrAction::Vec2(a) => a.as_raw(),
            OxrAction::Space(a) => a.as_raw(),
            OxrAction::Haptic(a) => a.as_raw(),
        }
    }
}

#[derive(Clone, Copy, Component)]
pub struct BindingsSuggested;

#[derive(Component, Deref)]
pub struct OxrActionSet(pub openxr::ActionSet);

pub struct OxrInputPlugin;
