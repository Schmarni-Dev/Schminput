use bevy::prelude::*;

use crate::subaction_paths::SubactionPath;

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Deref, Hash)]
#[repr(transparent)]
pub struct BindingModifiaction(pub Entity);

/// attached to an action
#[derive(Debug, Clone, Reflect, PartialEq, Eq, Hash, Component, Default)]
pub struct BindingModifications {
    pub all_paths: Option<BindingModifiaction>,
    pub per_path: Vec<(SubactionPath, BindingModifiaction)>,
}
impl BindingModifications {
    pub fn with_path_modification(mut self, path: SubactionPath, modification: Entity) -> Self {
        self.path_modification(path, modification);
        self
    }
    pub fn path_modification(&mut self, path: SubactionPath, modification: Entity) {
        self.per_path
            .push((path, BindingModifiaction(modification)));
    }
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Hash, Component)]
pub struct PremultiplyDeltaSecsModification;

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Hash, Component)]
pub struct UnboundedModification;
