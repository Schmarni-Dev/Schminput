use bevy::prelude::*;
use bevy_mod_xr::{session::XrPreSessionEnd, spaces::XrSpace};

use crate::subaction_paths::SubactionPathMap;

#[derive(Component, DerefMut, Deref, Clone, Copy)]
pub struct AttachSpaceToEntity(pub Entity);

#[derive(Component, DerefMut, Deref, Clone, Default)]
pub struct SpaceActionValue(pub SubactionPathMap<Option<XrSpace>>);

impl SpaceActionValue {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct GenericXrInputPlugin;
impl Plugin for GenericXrInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            XrPreSessionEnd,
            (reset_space_values, destroy_spaces_on_target_entities),
        );
    }
}

// should this really be pub(crate) instead of pub?
#[cfg_attr(target_family = "wasm", allow(dead_code))]
pub(crate) fn attach_spaces_to_target_entities(
    query: Query<(&AttachSpaceToEntity, &SpaceActionValue)>,
    check_query: Query<Has<XrSpace>>,
    mut cmds: Commands,
) {
    for (target, value) in query.iter() {
        let Some(space) = value.any else {
            warn!("no space to attach to entity");
            continue;
        };
        if !check_query.get(target.0).unwrap_or(true) {
            cmds.entity(target.0).insert(space);
        }
    }
}

fn destroy_spaces_on_target_entities(
    query: Query<&AttachSpaceToEntity>,
    check_query: Query<Has<XrSpace>>,
    mut cmds: Commands,
) {
    for target in query.iter() {
        if check_query.get(target.0).unwrap_or(false) {
            cmds.entity(target.0).remove::<XrSpace>();
        }
    }
}

fn reset_space_values(mut query: Query<&mut SpaceActionValue>) {
    for mut s in query.iter_mut() {
        s.any = None;
        s.paths.clear();
    }
}
