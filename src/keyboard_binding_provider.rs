use std::borrow::BorrowMut;

use bevy::{input::keyboard, prelude::*, utils::HashMap};

use crate::ActionTrait;

pub struct KeyboardBindingProvider;

pub enum KeyboardActivationType {
    JustPressed,
    Held,
    JustReleased,
}

pub enum KeyboardBinding {
    Simple(KeyboardActivationType, KeyCode),
    Number {
        activation: KeyboardActivationType,
        positive: KeyCode,
        negative: KeyCode,
    },
    Dpad {
        up: KeyCode,
        down: KeyCode,
        left: KeyCode,
        right: KeyCode,
    },
}

#[derive(Resource, Default)]
pub struct KeyboardBingings {
    #[allow(clippy::type_complexity)]
    bindings: HashMap<(&'static str, &'static str), Vec<KeyboardBinding>>,
}

impl KeyboardBingings {
    // I don't understand why this 'static is needed or why it works
    pub fn add_binding<T: 'static>(
        &mut self,
        action: &dyn ActionTrait<T = T>,
        binding: KeyboardBinding,
    ) {
        self.bindings
            .entry((action.action_key(), action.action_set_key()))
            .or_default()
            .push(binding);
    }
}

impl Plugin for KeyboardBindingProvider {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyboardBingings::default());
        app.add_systems(PreUpdate, sync_keaboard_actions_bool);
        app.add_systems(PreUpdate, sync_keaboard_actions_float);
        app.add_systems(PreUpdate, sync_keaboard_actions_vec2);
    }
}

fn sync_keaboard_actions_bool(
    mut actions: Query<&mut dyn ActionTrait<T = bool>>,
    bindings: Res<KeyboardBingings>,
    keyboard: Res<Input<KeyCode>>,
) {
    fn f(
        action: &dyn ActionTrait<T = bool>,
        input_type: &KeyboardActivationType,
        key: KeyCode,
        keyboard: &Input<KeyCode>,
    ) -> bool {
        match input_type {
            KeyboardActivationType::JustPressed => {
                let value = *action.get_value();
                value || keyboard.just_pressed(key)
            }
            KeyboardActivationType::Held => {
                let value = *action.get_value();
                value || keyboard.pressed(key)
            }
            KeyboardActivationType::JustReleased => {
                let value = *action.get_value();
                value || keyboard.just_released(key)
            }
        }
    }
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            for binding in bindings
                .bindings
                .get(&(action.action_key(), action.action_set_key()))
                .unwrap_or(&Vec::new())
            {
                #[allow(clippy::single_match)]
                match binding {
                    KeyboardBinding::Simple(input_type, key) => {
                        let v = f(&*action, input_type, *key, &keyboard);
                        action.set_value(v);
                    }
                    _ => (),
                }
            }
        })
    });
}
fn sync_keaboard_actions_float(
    mut actions: Query<&mut dyn ActionTrait<T = f32>>,
    bindings: Res<KeyboardBingings>,
    keyboard: Res<Input<KeyCode>>,
) {
    fn f(input_type: &KeyboardActivationType, key: KeyCode, keyboard: &Input<KeyCode>) -> f32 {
        match input_type {
            KeyboardActivationType::JustPressed => keyboard.just_pressed(key) as u8 as f32,
            KeyboardActivationType::Held => keyboard.pressed(key) as u8 as f32,
            KeyboardActivationType::JustReleased => keyboard.just_released(key) as u8 as f32,
        }
    }
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            for binding in bindings
                .bindings
                .get(&(action.action_key(), action.action_set_key()))
                .unwrap_or(&Vec::new())
            {
                match binding {
                    KeyboardBinding::Simple(input_type, key) => {
                        let mut v = *action.get_value();
                        v += f(input_type, *key, &keyboard);
                        action.set_value(v);
                    }
                    KeyboardBinding::Number {
                        activation,
                        positive,
                        negative,
                    } => {
                        let mut v = *action.get_value();
                        v += f(activation, *positive, &keyboard);
                        v += f(activation, *negative, &keyboard) * -1f32;
                        action.set_value(v);
                    }
                    _ => (),
                }
            }
        })
    });
}
fn sync_keaboard_actions_vec2(
    mut actions: Query<&mut dyn ActionTrait<T = Vec2>>,
    bindings: Res<KeyboardBingings>,
    keyboard: Res<Input<KeyCode>>,
) {
    fn f(input_type: &KeyboardActivationType, key: KeyCode, keyboard: &Input<KeyCode>) -> f32 {
        match input_type {
            KeyboardActivationType::JustPressed => keyboard.just_pressed(key) as u8 as f32,
            KeyboardActivationType::Held => keyboard.pressed(key) as u8 as f32,
            KeyboardActivationType::JustReleased => keyboard.just_released(key) as u8 as f32,
        }
    }
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            for binding in bindings
                .bindings
                .get(&(action.action_key(), action.action_set_key()))
                .unwrap_or(&Vec::new())
            {
                match binding {
                    KeyboardBinding::Simple(input_type, key) => {
                        let mut v = *action.get_value();
                        v.x += f(input_type, *key, &keyboard);
                        action.set_value(v);
                    }
                    KeyboardBinding::Number {
                        activation,
                        positive,
                        negative,
                    } => {
                        let mut v = *action.get_value();
                        v.x += f(activation, *positive, &keyboard);
                        v.x += f(activation, *negative, &keyboard) * -1f32;
                        action.set_value(v);
                    }
                    KeyboardBinding::Dpad {
                        up,
                        down,
                        left,
                        right,
                    } => {
                        let mut v = *action.get_value();
                        v.x += f(&KeyboardActivationType::Held, *up, &keyboard);
                        v.x -= f(&KeyboardActivationType::Held, *down, &keyboard);
                        v.y += f(&KeyboardActivationType::Held, *left, &keyboard);
                        v.y -= f(&KeyboardActivationType::Held, *right, &keyboard);
                        action.set_value(v);
                    }
                }
            }
        })
    });
}
