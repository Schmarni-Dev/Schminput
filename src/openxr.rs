use std::borrow::Cow;

use bevy::{platform::collections::HashMap, prelude::*};
#[cfg(not(target_family = "wasm"))]
use bevy_mod_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::{OxrActionSetSyncSet, OxrSyncActionSet},
    helper_traits::ToVec2 as _,
    resources::OxrInstance,
    session::OxrSession,
};
#[cfg(not(target_family = "wasm"))]
use bevy_mod_xr::session::{XrPreSessionEnd, XrSessionCreated};

use crate::ActionsInSet;
#[cfg(not(target_family = "wasm"))]
use crate::{
    binding_modification::{BindingModifications, PremultiplyDeltaSecsModification},
    subaction_paths::{RequestedSubactionPaths, SubactionPathStr},
    xr::SpaceActionValue,
    Action, ActionSet, BoolActionValue, F32ActionValue, SchminputSet, Vec2ActionValue,
};

pub const OCULUS_TOUCH_PROFILE: &str = "/interaction_profiles/oculus/touch_controller";
pub const META_TOUCH_PRO_PROFILE: &str = "/interaction_profiles/facebook/touch_controller_pro";
pub const META_TOUCH_PLUS_PROFILE: &str = "/interaction_profiles/meta/touch_controller_plus";

impl Plugin for OxrInputPlugin {
    #[cfg(not(target_family = "wasm"))]
    fn build(&self, app: &mut App) {
        use crate::xr::attach_spaces_to_target_entities;
        use bevy_mod_openxr::{openxr_session_available, openxr_session_running};
        use bevy_mod_xr::spaces::XrSpaceSyncSet;

        app.add_systems(
            PreUpdate,
            (
                sync_action_sets.before(OxrActionSetSyncSet),
                sync_non_blocking_action_sets.before(OxrActionSetSyncSet),
                sync_input_actions.after(OxrActionSetSyncSet),
                attach_spaces_to_target_entities,
            )
                .chain()
                .run_if(openxr_session_running)
                .in_set(SchminputSet::SyncInputActions)
                .before(XrSpaceSyncSet),
        );
        app.add_systems(
            XrSessionCreated,
            (create_input_actions, attach_action_sets)
                .chain()
                .run_if(openxr_session_available),
        );
        app.add_systems(OxrSendActionBindings, suggest_bindings);
        app.add_systems(
            PreUpdate,
            insert_xr_subaction_paths.run_if(openxr_session_available),
        );
        app.add_systems(XrPreSessionEnd, clean_actions);
        app.add_systems(XrPreSessionEnd, clean_action_sets);
    }
    #[cfg(all(feature = "xr", target_family = "wasm"))]
    fn build(&self, _app: &mut App) {}
}

#[cfg(not(target_family = "wasm"))]
fn clean_action_sets(query: Query<Entity, With<OxrActionSet>>, mut cmds: Commands) {
    for e in &query {
        cmds.entity(e).remove::<OxrActionSet>();
    }
}
#[cfg(not(target_family = "wasm"))]
fn clean_actions(query: Query<Entity, With<OxrAction>>, mut cmds: Commands) {
    for e in &query {
        cmds.entity(e)
            .remove::<OxrAction>()
            .remove::<BindingsSuggested>();
    }
}

#[cfg(not(target_family = "wasm"))]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Component)]
struct NonOxrSubationPath;

#[cfg(not(target_family = "wasm"))]
#[derive(Component, Clone)]
pub struct IsOxrSubactionPath;

#[cfg(not(target_family = "wasm"))]
#[derive(Component, Clone)]
pub struct OxrSubactionPath(pub openxr::Path);

#[cfg(not(target_family = "wasm"))]
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

fn sync_non_blocking_action_sets(world: &mut World) {
    let query = world
        .query::<(&OxrActionSet, &ActionSet, &ActionsInSet)>()
        .iter(world)
        .filter(|(_, v, _)| v.enabled && v.transparent)
        .map(|(set, _, actions)| (set.0.clone(), actions.0.iter().copied().collect::<Vec<_>>()))
        .collect::<Vec<_>>();

    for (set, actions) in query.into_iter() {
        let Some(session) = world.get_resource::<OxrSession>() else {
            continue;
        };
        if let Err(err) = session.sync_actions(&[openxr::ActiveActionSet::new(&set)]) {
            error!("error while syncing non blocking action set: {err}");
            continue;
        }
        for action in actions.into_iter() {
            if let Err(err) = world.run_system_cached_with(sync_input_action, action) {
                error!("{err}");
            }
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn sync_action_sets(
    query: Query<(&OxrActionSet, &ActionSet)>,
    mut sync_set: EventWriter<OxrSyncActionSet>,
) {
    let sets = query
        .iter()
        .filter(|(_, v)| v.enabled && !v.transparent)
        .map(|(set, _)| OxrSyncActionSet(set.0.clone()));
    sync_set.write_batch(sets);
}

#[cfg(not(target_family = "wasm"))]
fn attach_action_sets(query: Query<&OxrActionSet>, mut suggest: EventWriter<OxrAttachActionSet>) {
    for set in &query {
        suggest.write(OxrAttachActionSet(set.0.clone()));
    }
}

#[cfg(not(target_family = "wasm"))]
fn suggest_bindings(
    query: Query<(&OxrBindings, &OxrAction, Entity), Without<BindingsSuggested>>,
    mut suggest: EventWriter<OxrSuggestActionBinding>,
    mut cmds: Commands,
) {
    for (bindings, action, entity) in &query {
        for (profile, bindings) in bindings.bindings.iter() {
            suggest.write(OxrSuggestActionBinding {
                action: action.as_raw(),
                interaction_profile: profile.clone(),
                bindings: bindings.clone(),
            });
        }
        cmds.entity(entity).insert(BindingsSuggested);
    }
}

#[cfg(not(target_family = "wasm"))]
#[allow(clippy::type_complexity)]
fn create_input_actions(
    mut cmds: Commands,
    query: Query<(
        Entity,
        &Action,
        &RequestedSubactionPaths,
        Has<BoolActionValue>,
        Has<Vec2ActionValue>,
        Has<F32ActionValue>,
        Has<SpaceActionValue>,
    )>,
    path_query: Query<&OxrSubactionPath>,
    action_set_query: Query<&ActionSet>,
    instance: Res<OxrInstance>,
) {
    use bevy::platform::collections::HashMap;

    let mut set_map: HashMap<Entity, openxr::ActionSet> = HashMap::new();
    for (entity, action, requested_subaction_paths, has_bool, has_vec2, has_f32, has_space) in
        &query
    {
        let Ok(action_set) = action_set_query.get(action.set) else {
            error!("OpenXR action has an invalid Action Set at Setup!");
            continue;
        };
        let action_set = match set_map.get(&action.set) {
            Some(v) => v,
            None => {
                let set = match instance.create_action_set(
                    &action_set.name,
                    &action_set.localized_name,
                    action_set.priority,
                ) {
                    Ok(v) => v,
                    Err(err) => {
                        error!("error while creating action set: {err}");
                        continue;
                    }
                };
                set_map.insert(action.set, set);
                set_map.get(&action.set).unwrap()
            }
        };

        let paths = requested_subaction_paths
            .iter()
            .filter_map(|p| path_query.get(p.0).ok())
            .map(|p| p.0)
            .collect::<Vec<_>>();
        let action = match (has_bool, has_f32, has_vec2, has_space) {
            (true, false, false, false) => OxrAction::Bool(
                action_set
                    .create_action(&action.name, &action.localized_name, &paths)
                    .unwrap(),
            ),
            (false, true, false, false) => OxrAction::F32(
                action_set
                    .create_action(&action.name, &action.localized_name, &paths)
                    .unwrap(),
            ),
            (false, false, true, false) => OxrAction::Vec2(
                action_set
                    .create_action(&action.name, &action.localized_name, &paths)
                    .unwrap(),
            ),
            (false, false, false, true) => OxrAction::Space(
                action_set
                    .create_action(&action.name, &action.localized_name, &paths)
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
#[cfg(not(target_family = "wasm"))]
fn sync_input_actions(world: &mut World) {
    use crate::ActionsInSet;

    let entities = world
        .query::<(&ActionSet, &ActionsInSet)>()
        .iter(world)
        .filter(|&(set, _)| set.enabled && !set.transparent)
        .flat_map(|(_, actions)| actions.0.iter())
        .copied()
        .collect::<Vec<_>>();
    entities.into_iter().for_each(|e| {
        if let Err(err) = world.run_system_cached_with(sync_input_action, e) {
            error!("{err}");
        }
    });
}

#[cfg(not(target_family = "wasm"))]
#[allow(clippy::type_complexity)]
fn sync_input_action(
    action: In<Entity>,
    session: Res<OxrSession>,
    mut query: Query<(
        &mut OxrAction,
        Option<&mut BoolActionValue>,
        Option<&mut F32ActionValue>,
        Option<&mut Vec2ActionValue>,
        Option<&mut SpaceActionValue>,
        &RequestedSubactionPaths,
        &BindingModifications,
    )>,
    path_query: Query<&OxrSubactionPath>,
    simple_path_query: Query<Has<IsOxrSubactionPath>>,
    modification_query: Query<Has<PremultiplyDeltaSecsModification>>,
    time: Res<Time>,
) {
    let Ok((
        mut action,
        mut bool_val,
        mut f32_val,
        mut vec2_val,
        mut space_val,
        requested_subaction_paths,
        modifications,
    )) = query.get_mut(action.0)
    else {
        return;
    };

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
        true => time.delta_secs(),
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
                            *val.entry_with_path(sub_action_path).or_default() |= v.current_state;
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
                        Isometry3d::IDENTITY,
                    ) {
                        Ok(s) => {
                            val.replace(s);
                        }
                        Err(e) => {
                            warn!("unable to create space from action: {}", e);
                            return;
                        }
                    };
                }
                for (sub_path, path) in paths.into_iter() {
                    if val
                        .get_with_path(&sub_path)
                        .and_then(|v| v.as_ref())
                        .is_none()
                    {
                        match session.create_action_space(action, path, Isometry3d::IDENTITY) {
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
}

#[derive(Component, Default, Clone)]
pub struct OxrBindings {
    pub bindings: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
}

impl OxrBindings {
    pub fn interaction_profile(
        self,
        profile: impl Into<Cow<'static, str>>,
    ) -> OxrActionDeviceBindingBuilder {
        OxrActionDeviceBindingBuilder {
            builder: self,
            curr_interaction_profile: profile.into(),
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}

impl OxrBindings {
    /// alternative abstraction over the builder pattern
    pub fn bindings(
        self,
        profile: impl Into<Cow<'static, str>>,
        bindings: impl IntoIterator<Item = impl Into<Cow<'static, str>>>,
    ) -> Self {
        let mut profile = self.interaction_profile(profile);
        for binding in bindings.into_iter() {
            profile = profile.binding(binding)
        }
        profile.end()
    }
}

pub struct OxrActionDeviceBindingBuilder {
    builder: OxrBindings,
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

    pub fn end(self) -> OxrBindings {
        self.builder
    }
}

#[cfg(not(target_family = "wasm"))]
#[derive(Component)]
pub enum OxrAction {
    Bool(openxr::Action<bool>),
    F32(openxr::Action<f32>),
    Vec2(openxr::Action<openxr::Vector2f>),
    Space(openxr::Action<openxr::Posef>),
    Haptic(openxr::Action<openxr::Haptic>),
}

#[cfg(not(target_family = "wasm"))]
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

#[cfg(not(target_family = "wasm"))]
#[derive(Component, Deref)]
pub struct OxrActionSet(pub openxr::ActionSet);

pub struct OxrInputPlugin;
