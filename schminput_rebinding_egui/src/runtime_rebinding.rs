use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct PendingKeyboardRebinding {
    pub(crate) binding_index: usize,
    pub(crate) action: Entity,
}
