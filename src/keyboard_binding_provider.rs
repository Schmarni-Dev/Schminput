use bevy::{prelude::*, utils::HashMap};

use crate::ActionTrait;

pub struct KeyboardBindingProvider;

#[derive(Clone)]
pub enum KeyBinding {
    JustPressed(KeyCode),
    Held(KeyCode),
    JustReleased(KeyCode),
}

#[derive(Clone)]
pub enum KeyboardBinding {
    Simple(KeyBinding),
    Number {
        positive: KeyBinding,
        negative: KeyBinding,
    },
    Dpad {
        up: KeyBinding,
        down: KeyBinding,
        left: KeyBinding,
        right: KeyBinding,
    },
}

#[derive(Resource, Default)]
pub struct KeyboardBindings {
    #[allow(clippy::type_complexity)]
    bindings: HashMap<(&'static str, &'static str), Vec<KeyboardBinding>>,
}

impl KeyboardBindings {
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
        app.insert_resource(KeyboardBindings::default());
        app.add_systems(PreUpdate, sync_actions_bool);
        app.add_systems(PreUpdate, sync_actions_f32);
        app.add_systems(PreUpdate, sync_actions_vec2);
    }
}

fn sync_actions_bool(
    mut actions: Query<&mut dyn ActionTrait<T = bool>>,
    bindings: Res<KeyboardBindings>,
    keyboard: Res<Input<KeyCode>>,
) {
    fn f(
        action: &dyn ActionTrait<T = bool>,
        input_type: KeyBinding,
        keyboard: &Input<KeyCode>,
    ) -> bool {
        match input_type {
            KeyBinding::JustPressed(key) => {
                let value = *action.get_value();
                value || keyboard.just_pressed(key)
            }
            KeyBinding::Held(key) => {
                let value = *action.get_value();
                value || keyboard.pressed(key)
            }
            KeyBinding::JustReleased(key) => {
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
                match binding.clone() {
                    KeyboardBinding::Simple(input) => {
                        let v = f(&*action, input, &keyboard);
                        action.set_value(v);
                    }
                    _ => (),
                }
            }
        })
    });
}
fn sync_actions_f32(
    mut actions: Query<&mut dyn ActionTrait<T = f32>>,
    bindings: Res<KeyboardBindings>,
    keyboard: Res<Input<KeyCode>>,
) {
    fn f(input_type: KeyBinding, keyboard: &Input<KeyCode>) -> f32 {
        match input_type {
            KeyBinding::JustPressed(key) => keyboard.just_pressed(key) as u8 as f32,
            KeyBinding::Held(key) => keyboard.pressed(key) as u8 as f32,
            KeyBinding::JustReleased(key) => keyboard.just_released(key) as u8 as f32,
        }
    }
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            for binding in bindings
                .bindings
                .get(&(action.action_key(), action.action_set_key()))
                .unwrap_or(&Vec::new())
            {
                match binding.clone() {
                    KeyboardBinding::Simple(input) => {
                        let mut v = *action.get_value();
                        v += f(input, &keyboard);
                        action.set_value(v);
                    }
                    KeyboardBinding::Number { positive, negative } => {
                        let mut v = *action.get_value();
                        v += f(positive, &keyboard);
                        v += f(negative, &keyboard) * -1f32;
                        action.set_value(v);
                    }
                    _ => (),
                }
            }
        })
    });
}
fn sync_actions_vec2(
    mut actions: Query<&mut dyn ActionTrait<T = Vec2>>,
    bindings: Res<KeyboardBindings>,
    keyboard: Res<Input<KeyCode>>,
) {
    fn f(input_type: KeyBinding, keyboard: &Input<KeyCode>) -> f32 {
        match input_type {
            KeyBinding::JustPressed(key) => keyboard.just_pressed(key) as u8 as f32,
            KeyBinding::Held(key) => keyboard.pressed(key) as u8 as f32,
            KeyBinding::JustReleased(key) => keyboard.just_released(key) as u8 as f32,
        }
    }
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            for binding in bindings
                .bindings
                .get(&(action.action_key(), action.action_set_key()))
                .unwrap_or(&Vec::new())
            {
                match binding.clone() {
                    KeyboardBinding::Simple(input) => {
                        let mut v = *action.get_value();
                        v.x += f(input, &keyboard);
                        action.set_value(v);
                    }
                    KeyboardBinding::Number { positive, negative } => {
                        let mut v = *action.get_value();
                        v.x += f(positive, &keyboard);
                        v.x += f(negative, &keyboard) * -1f32;
                        action.set_value(v);
                    }
                    KeyboardBinding::Dpad {
                        up,
                        down,
                        left,
                        right,
                    } => {
                        let mut v = *action.get_value();
                        v.x += f(up, &keyboard);
                        v.x -= f(down, &keyboard);
                        v.y -= f(left, &keyboard);
                        v.y += f(right, &keyboard);
                        action.set_value(v);
                    }
                }
            }
        })
    });
}
