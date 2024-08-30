use bevy::prelude::*;

use crate::{
    subaction_paths::{RequestedSubactionPaths, SubactionPathCreated, SubactionPathStr}, InActionSet, ActionSetEnabled, BoolActionValue, ButtonInputBeheavior, F32ActionValue, InputAxis, InputAxisDirection, SchminputSet, Vec2ActionValue
};

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            sync_actions.in_set(SchminputSet::SyncInputActions),
        );
        app.add_systems(
            PreUpdate,
            handle_new_subaction_paths.in_set(SchminputSet::HandleNewSubactionPaths),
        );
    }
}

pub fn handle_new_subaction_paths(
    query: Query<&SubactionPathStr>,
    mut reader: EventReader<SubactionPathCreated>,
    mut cmds: Commands,
) {
    for (e, str) in reader
        .read()
        .filter_map(|e| Some((e.0 .0, query.get(e.0 .0).ok()?)))
    {
        if str.0.strip_prefix("/keyboard").is_some() {
            cmds.entity(e).insert(KeyboardSubactionPath);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn sync_actions(
    mut action_query: Query<(
        &KeyboardBindings,
        &InActionSet,
        Option<&mut BoolActionValue>,
        Option<&mut F32ActionValue>,
        Option<&mut Vec2ActionValue>,
        &RequestedSubactionPaths,
    )>,
    path_query: Query<Has<KeyboardSubactionPath>>,
    set_query: Query<&ActionSetEnabled>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (bindings, set, mut bool_value, mut f32_value, mut vec2_value, requested_paths) in
        &mut action_query
    {
        if !(set_query.get(set.0).is_ok_and(|v| v.0)) {
            continue;
        };
        let paths = requested_paths
            .0
            .iter()
            .filter(|p| path_query.get(p.0).unwrap_or(false))
            .collect::<Vec<_>>();
        for binding in &bindings.0 {
            let delta_multiplier = match binding.premultiply_delta_time {
                true => time.delta_seconds(),
                false => 1.0,
            };
            if let Some(button) = bool_value.as_mut() {
                button.any |= binding.behavior.apply(&input, binding.key);
                for p in paths.iter() {
                    *button.entry_with_path(**p).or_default() |=
                        binding.behavior.apply(&input, binding.key);
                }
            }
            if let Some(float) = f32_value.as_mut() {
                if binding.axis == InputAxis::X {
                    let val = binding.behavior.apply(&input, binding.key) as u8 as f32;

                    float.any += val * binding.axis_dir.as_multipier() * delta_multiplier;
                    for p in paths.iter() {
                        *float.entry_with_path(**p).or_default() +=
                            val * binding.axis_dir.as_multipier() * delta_multiplier;
                    }
                }
            }
            if let Some(vec) = vec2_value.as_mut() {
                let val = binding.behavior.apply(&input, binding.key) as u8 as f32;
                match binding.axis {
                    InputAxis::X => {
                        vec.any.x += val * binding.axis_dir.as_multipier() * delta_multiplier
                    }
                    InputAxis::Y => {
                        vec.any.y += val * binding.axis_dir.as_multipier() * delta_multiplier
                    }
                };
                for p in paths.iter() {
                    match binding.axis {
                        InputAxis::X => {
                            vec.entry_with_path(**p).or_default().x +=
                                val * binding.axis_dir.as_multipier() * delta_multiplier
                        }
                        InputAxis::Y => {
                            vec.entry_with_path(**p).or_default().y +=
                                val * binding.axis_dir.as_multipier() * delta_multiplier
                        }
                    };
                }
            }
        }
    }
}

// TODO: switch binding behavior to use subaction paths?
#[derive(Clone, Copy, Debug, Default, Component, Reflect)]
pub struct KeyboardSubactionPath;

#[derive(Clone, Debug, Default, Component, Reflect)]
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
