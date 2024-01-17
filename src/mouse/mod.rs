pub mod motion;
pub mod mouse_binding_provider;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use mouse_binding_provider::MouseBinding;

use crate::ActionTrait;

use self::motion::MouseMotionAction;

pub enum MouseBindingType {
    Button(MouseBinding),
    Motion,
}
#[derive(Resource, Default)]
pub struct MouseBindings {
    #[allow(clippy::type_complexity)]
    bindings: HashMap<(&'static str, &'static str), Vec<MouseBinding>>,
    motion_bindings: HashSet<(&'static str, &'static str)>,
}

impl MouseBindings {
    // I don't understand why this 'static is needed or why it works
    pub fn add_binding<T: 'static>(
        &mut self,
        action: &dyn ActionTrait<T = T>,
        binding: MouseBinding,
    ) {
        self.bindings
            .entry((action.action_key(), action.action_set_key()))
            .or_default()
            .push(binding);
    }
    pub fn add_motion_binding(&mut self, action: &dyn MouseMotionAction) {
        self.motion_bindings
            .insert((action.action_key(), action.action_set_key()));
    }
    pub fn drop_bindings<T: 'static>(&mut self, action: &dyn ActionTrait<T = T>) {
        self.bindings
            .remove(&(action.action_key(), action.action_set_key()));
    }
    pub fn drop_motion_binding(&mut self, action: &dyn MouseMotionAction) {
        self.motion_bindings
            .remove(&(action.action_key(), action.action_set_key()));
    }
}
