use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::{
    impl_helpers::{BindingValue, ProviderParam}, priorities::PriorityAppExt as _, subaction_paths::{SubactionPathCreated, SubactionPathStr}, ButtonInputBeheavior, InputAxis, InputAxisDirection, SchminputSet
};

pub struct MousePlugin;

impl Plugin for MousePlugin {
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
            "schminput:mouse",
            |entity: In<Entity>, query: Query<&MouseBindings>| {
                let Ok(bindings) = query.get(entity.0) else {
                    return Vec::new();
                };
                bindings
                    .buttons
                    .iter()
                    .cloned()
                    .map(AnyMouseBinding::Button)
                    .chain(bindings.movement.map(AnyMouseBinding::Motion))
                    .map(|v| get_binding_id(&v))
                    .collect()
            },
        );
    }
}

fn get_binding_id(binding: &AnyMouseBinding) -> u64 {
    let mut hasher = DefaultHasher::new();
    match binding {
        AnyMouseBinding::Button(MouseButtonBinding { button, .. }) => button.hash(&mut hasher),
        AnyMouseBinding::Motion(MouseMotionBinding { motion_type, .. }) => {
            motion_type.hash(&mut hasher)
        }
    }
    hasher.finish()
}

fn handle_new_subaction_paths(
    query: Query<&SubactionPathStr>,
    mut event: MessageReader<SubactionPathCreated>,
    mut cmds: Commands,
) {
    for (entity, path) in event
        .read()
        .filter_map(|e| Some((e.0 .0, query.get(e.0 .0).ok()?)))
    {
        if let Some(sub_path) = path.0.strip_prefix("/mouse") {
            if sub_path.is_empty() || sub_path == "/*" {
                cmds.entity(entity).insert(MouseSubactionPath::All);
                continue;
            }
            if sub_path == "/motion" {
                cmds.entity(entity).insert(MouseSubactionPath::DeltaMotion);
                continue;
            }
            if sub_path == "/button" {
                cmds.entity(entity).insert(MouseSubactionPath::Button);
                continue;
            }
            // if sub_path == "/scroll" {
            //     cmds.entity(entity).insert(MouseSubactionPath::Scroll);
            //     continue;
            // }
        }
    }
}

enum AnyMouseBinding {
    Button(MouseButtonBinding),
    Motion(MouseMotionBinding),
}

#[allow(clippy::type_complexity)]
pub fn sync_actions(
    mut query: ProviderParam<&MouseBindings, &MouseSubactionPath>,
    time: Res<Time>,
    input: Res<ButtonInput<MouseButton>>,
    mut delta_motion: MessageReader<MouseMotion>,
) {
    query.run(
        "schminput:mouse",
        get_binding_id,
        |binding, path| {
            matches!(
                (binding, path),
                (
                    AnyMouseBinding::Motion(MouseMotionBinding {
                        motion_type: MouseMotionType::DeltaMotion,
                        ..
                    }),
                    MouseSubactionPath::DeltaMotion
                ) | (AnyMouseBinding::Button(_), MouseSubactionPath::Button)
                    | (_, MouseSubactionPath::All)
            )
        },
        |bindings| {
            bindings
                .buttons
                .iter()
                .cloned()
                .map(AnyMouseBinding::Button)
                .chain(bindings.movement.map(AnyMouseBinding::Motion))
                .collect()
        },
        |binding, _, _, data| {
            let time_mutiplier = match data.modifications.premul_delta_time {
                true => time.delta_secs(),
                false => 1.0,
            };
            match binding {
                AnyMouseBinding::Button(button) => {
                    let bool = data
                        .is_bool
                        .then(|| button.behavior.apply(&input, button.button));
                    let f32 = data.is_f32.then(|| {
                        button.behavior.apply(&input, button.button) as u8 as f32
                            * button.axis_dir.as_multipier()
                            * time_mutiplier
                    });
                    let vec2 = data.is_vec2.then(|| {
                        let val = button.behavior.apply(&input, button.button) as u8 as f32;
                        button
                            .axis
                            .new_vec(val * button.axis_dir.as_multipier() * time_mutiplier)
                    });
                    vec![BindingValue { vec2, bool, f32 }]
                }
                AnyMouseBinding::Motion(MouseMotionBinding {
                    motion_type,
                    multiplier,
                }) => match motion_type {
                    MouseMotionType::DeltaMotion => {
                        let mut delta = Vec2::ZERO;
                        for e in delta_motion.read() {
                            let mut v = e.delta;
                            v.y *= -1.0;
                            delta += v * multiplier * time_mutiplier;
                        }
                        let bool = data.is_bool.then_some(delta != Vec2::ZERO);
                        let f32 = data.is_f32.then_some(delta.x);
                        let vec2 = data.is_vec2.then_some(delta);

                        vec![BindingValue { vec2, bool, f32 }]
                    }
                },
            }
        },
    );
}

#[derive(Clone, Debug, Reflect, Component, Copy, PartialEq, Eq)]
pub enum MouseSubactionPath {
    DeltaMotion,
    Button,
    // Scroll,
    All,
}

#[derive(Clone, Default, Debug, Reflect, Component)]
pub struct MouseBindings {
    pub buttons: Vec<MouseButtonBinding>,
    pub movement: Option<MouseMotionBinding>,
}

impl MouseBindings {
    pub fn bind(mut self, binding: MouseButtonBinding) -> Self {
        self.buttons.push(binding);
        self
    }
    pub fn delta_motion(mut self) -> Self {
        let mut mmb = match self.movement {
            Some(v) => v,
            None => MouseMotionBinding {
                motion_type: MouseMotionType::DeltaMotion,
                multiplier: 1.0,
            },
        };
        mmb.motion_type = MouseMotionType::DeltaMotion;
        self.movement = Some(mmb);
        self
    }
    pub fn motion_multiplier(mut self, multiplier: f32) -> Self {
        let mut mmb = match self.movement {
            Some(v) => v,
            None => MouseMotionBinding {
                motion_type: MouseMotionType::DeltaMotion,
                multiplier: 1.0,
            },
        };
        mmb.multiplier = multiplier;
        self.movement = Some(mmb);
        self
    }

    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct MouseButtonBinding {
    pub axis: InputAxis,
    pub axis_dir: InputAxisDirection,
    pub button: MouseButton,
    pub behavior: ButtonInputBeheavior,
}

impl MouseButtonBinding {
    pub fn new(button: MouseButton) -> MouseButtonBinding {
        MouseButtonBinding {
            axis: default(),
            axis_dir: default(),
            button,
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

    pub fn just_pressed(mut self) -> Self {
        self.behavior = ButtonInputBeheavior::JustPressed;
        self
    }

    pub fn just_released(mut self) -> Self {
        self.behavior = ButtonInputBeheavior::JustReleased;
        self
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct MouseMotionBinding {
    pub motion_type: MouseMotionType,
    pub multiplier: f32,
}
impl Default for MouseMotionBinding {
    fn default() -> Self {
        Self {
            motion_type: MouseMotionType::DeltaMotion,
            multiplier: 1.0,
        }
    }
}
impl MouseMotionBinding {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, Default, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum MouseMotionType {
    #[default]
    DeltaMotion,
}
