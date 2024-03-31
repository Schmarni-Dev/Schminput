use bevy::prelude::*;

use crate::{
    BoolActionValue, InputAxis, InputAxisDirection, ButtonInputBeheavior,
    F32ActionValue, SchminputSet, Vec2ActionValue,
};

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, sync_actions.in_set(SchminputSet::SyncActions));
    }
}

#[allow(clippy::type_complexity)]
pub fn sync_actions(
    mut action_query: Query<(
        &KeyboardBindings,
        Option<&mut BoolActionValue>,
        Option<&mut F32ActionValue>,
        Option<&mut Vec2ActionValue>,
    )>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (bindings, mut bool_value, mut f32_value, mut vec2_value) in &mut action_query {
        for binding in &bindings.0 {
            let delta_multiplier = match binding.premultiply_delta_time {
                true => time.delta_seconds(),
                false => 1.0,
            };
            if let Some(button) = bool_value.as_mut() {
                button.0 |= binding.behavior.apply(&input, binding.key);
            }
            if let Some(float) = f32_value.as_mut() {
                if binding.axis == InputAxis::X {
                    let val = binding.behavior.apply(&input, binding.key) as u8 as f32;

                    float.0 += val * binding.axis_dir.as_multipier() * delta_multiplier;
                }
            }
            if let Some(vec) = vec2_value.as_mut() {
                let val = binding.behavior.apply(&input, binding.key) as u8 as f32;
                match binding.axis {
                    InputAxis::X => {
                        vec.x += val * binding.axis_dir.as_multipier() * delta_multiplier
                    }
                    InputAxis::Y => {
                        vec.y += val * binding.axis_dir.as_multipier() * delta_multiplier
                    }
                };
            }
        }
    }
}

#[derive(Clone, Debug, Default, DerefMut, Deref, Component, Reflect)]
pub struct KeyboardBindings(pub Vec<KeyboardBinding>);

impl KeyboardBindings {
    pub fn add_binding(mut self, binding: KeyboardBinding) -> Self {
        self.0.push(binding);
        self
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct KeyboardBinding {
    pub key: KeyCode,
    pub axis: InputAxis,
    pub axis_dir: InputAxisDirection,
    pub premultiply_delta_time: bool,
    pub behavior: ButtonInputBeheavior,
    pub multiplier: f32,
}

impl KeyboardBinding {
    pub fn new(key_code: KeyCode) -> KeyboardBinding {
        KeyboardBinding {
            key: key_code,
            premultiply_delta_time: false,
            multiplier: 1.0,
            axis: default(),
            axis_dir: default(),
            behavior: default(),
        }
    }

    pub fn x_axis(mut self) -> Self {
        self.axis = InputAxis::X;
        self
    }

    pub fn y_axis(mut self) -> Self {
        self.axis = InputAxis::Y;
        self
    }

    pub fn positive_axis_dir(mut self) -> Self {
        self.axis_dir = InputAxisDirection::Positive;
        self
    }

    pub fn negative_axis_dir(mut self) -> Self {
        self.axis_dir = InputAxisDirection::Negative;
        self
    }

    pub fn multiplier(mut self, multiplier: f32) -> Self {
        self.multiplier = multiplier;
        self
    }

    pub fn premultiply_delta_time(mut self) -> KeyboardBinding {
        self.premultiply_delta_time = true;
        self
    }

    pub fn just_pressed(mut self) -> KeyboardBinding {
        self.behavior = ButtonInputBeheavior::JustPressed;
        self
    }

    pub fn just_released(mut self) -> KeyboardBinding {
        self.behavior = ButtonInputBeheavior::JustReleased;
        self
    }
}

pub struct KeyboardPlugin;
