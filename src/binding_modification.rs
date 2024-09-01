use bevy::prelude::*;

use crate::subaction_paths::SubactionPath;

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Deref, Hash)]
#[repr(transparent)]
pub struct BindingModifiaction(pub Entity);

/// attached to an action
#[derive(Debug, Clone, Reflect, PartialEq, Eq, Hash, Component, Default)]
pub struct BindingModifiactions {
    pub all_paths: Option<BindingModifiaction>,
    pub per_path: Vec<(SubactionPath, BindingModifiaction)>,
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Hash, Component)]
pub struct PremultiplyDeltaTimeSecondsModification;

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Hash, Component)]
pub struct UnboundedModification;
