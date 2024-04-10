use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::{
    BoolActionValue, InputAxis, InputAxisDirection, ButtonInputBeheavior,
    F32ActionValue, SchminputSet, Vec2ActionValue,
};

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, sync_actions.in_set(SchminputSet::SyncInputActions));
    }
}

#[allow(clippy::type_complexity)]
pub fn sync_actions(
    mut action_query: Query<(
        &MouseBindings,
        Option<&mut BoolActionValue>,
        Option<&mut F32ActionValue>,
        Option<&mut Vec2ActionValue>,
    )>,
    time: Res<Time>,
    input: Res<ButtonInput<MouseButton>>,
    mut delta_motion: EventReader<MouseMotion>,
) {
    for (binding, mut bool_value, mut f32_value, mut vec2_value) in &mut action_query {
        for button in &binding.buttons {
            let delta_mutiplier = match button.premultipy_delta_time {
                true => time.delta_seconds(),
                false => 1.0,
            };
            if let Some(boolean) = bool_value.as_mut() {
                boolean.0 |= button.behavior.apply(&input, button.button);
            }
            if let Some(float) = f32_value.as_mut() {
                if button.axis == InputAxis::X {
                    let val = button.behavior.apply(&input, button.button) as u8 as f32;

                    float.0 += val * button.axis_dir.as_multipier() * delta_mutiplier;
                }
            }
            if let Some(vec) = vec2_value.as_mut() {
                let val = button.behavior.apply(&input, button.button) as u8 as f32;

                *button.axis.vec_axis_mut(vec) +=
                    val * button.axis_dir.as_multipier() * delta_mutiplier;
            }
        }

        let Some(movement) = binding.movement else {
            continue;
        };
        info!("mouse");
        if movement.motion_type == MouseMotionType::DeltaMotion {
            let mut delta = Vec2::ZERO;
            for e in delta_motion.read() {
                let mut v = e.delta;
                v.y *= -1.0;
                delta += v * movement.multiplier;
            }
            if let Some(boolean) = bool_value.as_mut() {
                boolean.0 |= delta != Vec2::ZERO;
            }
            if let Some(float) = f32_value.as_mut() {
                float.0 += delta.x;
            }
            if let Some(vec2) = vec2_value.as_mut() {
                vec2.0 += delta;
            }
        }
    }
}

#[derive(Clone, Default, Debug, Reflect, Component)]
pub struct MouseBindings {
    pub buttons: Vec<MouseButtonBinding>,
    // should i support multiple motion bindings per action?
    pub movement: Option<MouseMotionBinding>,
}

impl MouseBindings {
    pub fn add_binding(mut self, binding: MouseButtonBinding) -> Self {
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
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct MouseButtonBinding {
    pub axis: InputAxis,
    pub axis_dir: InputAxisDirection,
    pub button: MouseButton,
    pub premultipy_delta_time: bool,
    pub behavior: ButtonInputBeheavior,
}

#[derive(Clone, Copy, Default, Debug, Reflect)]
pub struct MouseMotionBinding {
    pub motion_type: MouseMotionType,
    pub multiplier: f32,
}

#[derive(Clone, Copy, Default, Debug, Reflect, PartialEq, Eq)]
pub enum MouseMotionType {
    #[default]
    DeltaMotion,
}
