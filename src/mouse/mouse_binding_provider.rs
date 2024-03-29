use bevy::prelude::*;

use crate::ActionTrait;

use super::MouseBindings;

#[derive(Clone)]
pub enum MouseBinding {
    /// Supported Action types: bool. f32
    JustPressed(MouseButton),
    /// Supported Action types: bool. f32
    Held(MouseButton),
    /// Supported Action types: bool. f32
    JustReleased(MouseButton),
}

pub struct MouseBindingProvider;

impl Plugin for MouseBindingProvider {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseBindings::default());
        app.add_systems(PreUpdate, sync_actions_bool);
        app.add_systems(PreUpdate, sync_actions_f32);
    }
}

fn sync_actions_bool(
    mut actions: Query<&mut dyn ActionTrait<T = bool>>,
    bindings: Res<MouseBindings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            for binding in bindings
                .bindings
                .get(&(action.action_key(), action.action_set_key()))
                .unwrap_or(&Vec::new())
            {
                #[allow(clippy::single_match)]
                match binding.clone() {
                    MouseBinding::JustPressed(button) => {
                        let v = *action.get_value();
                        action.set_value(v || mouse_buttons.just_pressed(button))
                    }
                    MouseBinding::Held(button) => {
                        let v = *action.get_value();
                        action.set_value(v || mouse_buttons.pressed(button))
                    }
                    MouseBinding::JustReleased(button) => {
                        let v = *action.get_value();
                        action.set_value(v || mouse_buttons.just_released(button))
                    }
                }
            }
        })
    });
}

fn sync_actions_f32(
    mut actions: Query<&mut dyn ActionTrait<T = f32>>,
    bindings: Res<MouseBindings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            for binding in bindings
                .bindings
                .get(&(action.action_key(), action.action_set_key()))
                .unwrap_or(&Vec::new())
            {
                #[allow(clippy::single_match)]
                match binding.clone() {
                    MouseBinding::JustPressed(button) => {
                        let v = *action.get_value();
                        action.set_value(v + mouse_buttons.just_pressed(button) as u8 as f32)
                    }
                    MouseBinding::Held(button) => {
                        let v = *action.get_value();
                        action.set_value(v + mouse_buttons.pressed(button) as u8 as f32)
                    }
                    MouseBinding::JustReleased(button) => {
                        let v = *action.get_value();
                        action.set_value(v + mouse_buttons.just_released(button) as u8 as f32)
                    }
                }
            }
        })
    });
}
