use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;

use crate::{
    impl_helpers::{BindingValue, ProviderParam},
    priorities::PriorityAppExt,
    subaction_paths::{SubactionPathCreated, SubactionPathStr},
    ButtonInputBeheavior, InputAxis, InputAxisDirection, SchminputSet,
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
        app.add_binding_id_system(
            "schminput:keyboard",
            |entity: In<Entity>, query: Query<&KeyboardBindings>| {
                let Ok(bindings) = query.get(entity.0) else {
                    return Vec::new();
                };
                bindings.0.iter().map(get_binding_id).collect()
            },
        );
    }
}

fn get_binding_id(binding: &KeyboardBinding) -> u64 {
    let mut hasher = DefaultHasher::new();
    binding.key.hash(&mut hasher);
    hasher.finish()
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
    mut query: ProviderParam<&KeyboardBindings, Has<KeyboardSubactionPath>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
) {
    query.run(
        "schminput:keyboard",
        get_binding_id,
        |_, v| *v,
        |bindings| bindings.0.clone(),
        |binding, _, _, data| {
            let delta_multiplier = match data.modifications.premul_delta_time {
                true => time.delta_secs(),
                false => 1.0,
            };
            let bool = data
                .is_bool
                .then(|| binding.behavior.apply(&input, binding.key));
            let f32 = data.is_f32.then(|| {
                binding.behavior.apply(&input, binding.key) as u8 as f32
                    * binding.axis_dir.as_multipier()
                    * delta_multiplier
            });
            let vec2 = data.is_vec2.then(|| {
                let val = binding.behavior.apply(&input, binding.key) as u8 as f32;
                match binding.axis {
                    InputAxis::X => Vec2::new(
                        val * binding.axis_dir.as_multipier() * delta_multiplier,
                        0.0,
                    ),
                    InputAxis::Y => Vec2::new(
                        0.0,
                        val * binding.axis_dir.as_multipier() * delta_multiplier,
                    ),
                }
            });

            vec![BindingValue { vec2, bool, f32 }]
        },
    );
}

// TODO: switch binding behavior to use subaction paths?
#[derive(Clone, Copy, Debug, Default, Component, Reflect)]
pub struct KeyboardSubactionPath;

#[derive(Clone, Debug, Default, Component, Reflect)]
pub struct KeyboardBindings(pub Vec<KeyboardBinding>);

impl KeyboardBindings {
    pub fn bind(mut self, binding: KeyboardBinding) -> Self {
        self.0.push(binding);
        self
    }

    pub fn new() -> Self {
        Self::default()
    }
}

//helper functions
impl KeyboardBindings {
    /// helper function for adding a dpad style binding, internally this just calls add_binding
    pub fn add_dpad(self, up: KeyCode, down: KeyCode, left: KeyCode, right: KeyCode) -> Self {
        self.bind(KeyboardBinding::new(up).y_axis().positive_axis_dir())
            .bind(KeyboardBinding::new(down).y_axis().negative_axis_dir())
            .bind(KeyboardBinding::new(right).x_axis().positive_axis_dir())
            .bind(KeyboardBinding::new(left).x_axis().negative_axis_dir())
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct KeyboardBinding {
    pub key: KeyCode,
    pub axis: InputAxis,
    pub axis_dir: InputAxisDirection,
    pub behavior: ButtonInputBeheavior,
    pub multiplier: f32,
}

impl KeyboardBinding {
    pub fn new(key_code: KeyCode) -> KeyboardBinding {
        KeyboardBinding {
            key: key_code,
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
