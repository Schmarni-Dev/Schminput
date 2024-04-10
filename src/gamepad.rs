use std::time::Duration;

use bevy::{
    input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest},
    prelude::*,
    utils::HashMap,
};

use crate::{
    BoolActionValue, ButtonInputBeheavior, F32ActionValue, InputAxis, InputAxisDirection,
    SchminputSet, Vec2ActionValue,
};

pub struct GamepadPlugin;

impl Plugin for GamepadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            sync_actions.in_set(SchminputSet::SyncInputActions),
        );
        app.add_systems(PreUpdate, clear_haptic.in_set(SchminputSet::ClearValues));
        app.add_systems(
            PostUpdate,
            sync_haptics.in_set(SchminputSet::SyncOutputActions),
        );
    }
}

fn clear_haptic(mut query: Query<&mut GamepadHapticOutput>) {
    for mut out in &mut query {
        out.haptic_feedbacks.clear();
    }
}

fn sync_haptics(
    mut gamepad_haptic_event: EventWriter<GamepadRumbleRequest>,
    haptic_query: Query<(&GamepadHapticOutputBindings, &GamepadHapticOutput)>,
    gamepads: Res<Gamepads>,
) {
    for (bindings, out) in &haptic_query {
        for (device, binding) in bindings
            .bindings
            .iter()
            .flat_map(|(device, bindings)| bindings.iter().map(|b| (*device, b)))
        {
            match device {
                GamepadBindingDevice::Any => {
                    for gamepad in gamepads.iter() {
                        for e in &out.haptic_feedbacks {
                            gamepad_haptic_event.send(match e {
                                GamepadHapticValue::Add {
                                    duration,
                                    intensity,
                                } => GamepadRumbleRequest::Add {
                                    duration: *duration,
                                    intensity: binding.as_rumble_intensity(*intensity),
                                    gamepad,
                                },
                                GamepadHapticValue::Stop => todo!(),
                            });
                        }
                    }
                }
                GamepadBindingDevice::Gamepad(gamepad) => {
                    for e in &out.haptic_feedbacks {
                        gamepad_haptic_event.send(match e {
                            GamepadHapticValue::Add {
                                duration,
                                intensity,
                            } => GamepadRumbleRequest::Add {
                                duration: *duration,
                                intensity: binding.as_rumble_intensity(*intensity),
                                gamepad,
                            },
                            GamepadHapticValue::Stop => todo!(),
                        });
                    }
                }
            };
        }
    }
}

#[allow(clippy::type_complexity)]
fn sync_actions(
    axis: Res<Axis<GamepadAxis>>,
    button: Res<ButtonInput<GamepadButton>>,
    gamepads: Res<Gamepads>,
    mut query: Query<(
        &GamepadBindings,
        Option<&mut BoolActionValue>,
        Option<&mut F32ActionValue>,
        Option<&mut Vec2ActionValue>,
    )>,
    time: Res<Time>,
) {
    for (gamepad_bindings, mut bool_value, mut float_value, mut vec2_value) in &mut query {
        for (device, bindings) in &gamepad_bindings.bindings {
            match device {
                GamepadBindingDevice::Any => {
                    for gamepad in gamepads.iter() {
                        for binding in bindings {
                            handle_gamepad_inputs(
                                gamepad,
                                binding,
                                &axis,
                                &button,
                                bool_value.as_deref_mut(),
                                float_value.as_deref_mut(),
                                vec2_value.as_deref_mut(),
                                &time,
                            );
                        }
                    }
                }
                GamepadBindingDevice::Gamepad(gamepad) => {
                    for binding in bindings {
                        handle_gamepad_inputs(
                            *gamepad,
                            binding,
                            &axis,
                            &button,
                            bool_value.as_deref_mut(),
                            float_value.as_deref_mut(),
                            vec2_value.as_deref_mut(),
                            &time,
                        );
                    }
                }
            };
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_gamepad_inputs(
    gamepad: Gamepad,
    binding: &GamepadBinding,
    axis: &Axis<GamepadAxis>,
    button: &ButtonInput<GamepadButton>,
    bool_value: Option<&mut BoolActionValue>,
    float_value: Option<&mut F32ActionValue>,
    vec2_value: Option<&mut Vec2ActionValue>,
    time: &Time,
) {
    match binding.source {
        GamepadBindingSource::Axis(axis_type) => {
            let delta_multiplier = match binding.premultiply_delta_time {
                true => time.delta_seconds(),
                false => 1.0,
            };
            let Some(v) = (match binding.unbounded {
                true => axis.get_unclamped(GamepadAxis::new(gamepad, axis_type)),
                false => axis.get(GamepadAxis::new(gamepad, axis_type)),
            }) else {
                warn!("axis.get returned None, idk what that means");
                return;
            };

            if let Some(bool_value) = bool_value {
                bool_value.0 |= v > 0.0;
            }
            if let Some(float_value) = float_value {
                float_value.0 += v * binding.axis_dir.as_multipier() * delta_multiplier;
            }
            if let Some(vec2_value) = vec2_value {
                match binding.axis {
                    InputAxis::X => {
                        vec2_value.x += v * binding.axis_dir.as_multipier() * delta_multiplier
                    }
                    InputAxis::Y => {
                        vec2_value.y += v * binding.axis_dir.as_multipier() * delta_multiplier
                    }
                }
            }
        }
        GamepadBindingSource::Button(button_type) => {
            let delta_multiplier = match binding.premultiply_delta_time {
                true => time.delta_seconds(),
                false => 1.0,
            };
            let v = button.pressed(GamepadButton::new(gamepad, button_type));

            if let Some(bool_value) = bool_value {
                bool_value.0 |= v;
            }
            if let Some(float_value) = float_value {
                float_value.0 +=
                    v as u8 as f32 * binding.axis_dir.as_multipier() * delta_multiplier;
            }
            if let Some(vec2_value) = vec2_value {
                match binding.axis {
                    InputAxis::X => {
                        vec2_value.x +=
                            v as u8 as f32 * binding.axis_dir.as_multipier() * delta_multiplier
                    }
                    InputAxis::Y => {
                        vec2_value.y +=
                            v as u8 as f32 * binding.axis_dir.as_multipier() * delta_multiplier
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum GamepadHapticType {
    Weak,
    Strong,
}

impl GamepadHapticType {
    pub fn as_rumble_intensity(&self, intensity: f32) -> GamepadRumbleIntensity {
        match self {
            GamepadHapticType::Weak => GamepadRumbleIntensity::weak_motor(intensity),
            GamepadHapticType::Strong => GamepadRumbleIntensity::strong_motor(intensity),
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq)]
pub enum GamepadHapticValue {
    Add { duration: Duration, intensity: f32 },
    Stop,
}

#[derive(Clone, Component, Debug, Reflect, Default)]
pub struct GamepadHapticOutputBindings {
    pub bindings: HashMap<GamepadBindingDevice, Vec<GamepadHapticType>>,
}

impl GamepadHapticOutputBindings {
    pub fn new() -> GamepadHapticOutputBindings {
        GamepadHapticOutputBindings::default()
    }
    pub fn weak(mut self, device: GamepadBindingDevice) -> Self {
        self.bindings
            .entry(device)
            .or_default()
            .push(GamepadHapticType::Weak);
        self
    }
    pub fn strong(mut self, device: GamepadBindingDevice) -> Self {
        self.bindings
            .entry(device)
            .or_default()
            .push(GamepadHapticType::Strong);
        self
    }
}

#[derive(Clone, Component, Debug, Reflect, Default)]
pub struct GamepadHapticOutput {
    pub haptic_feedbacks: Vec<GamepadHapticValue>,
}

impl GamepadHapticOutput {
    pub fn add(&mut self, duration: Duration, intensity: f32) -> &mut Self {
        self.haptic_feedbacks.push(GamepadHapticValue::Add {
            duration,
            intensity,
        });
        self
    }
    pub fn stop(&mut self) -> &mut Self {
        self.haptic_feedbacks.push(GamepadHapticValue::Stop);
        self
    }
}

#[derive(Clone, Component, Debug, Reflect, Default)]
pub struct GamepadBindings {
    pub bindings: HashMap<GamepadBindingDevice, Vec<GamepadBinding>>,
}

impl GamepadBindings {
    pub fn add_binding(mut self, device: GamepadBindingDevice, binding: GamepadBinding) -> Self {
        self.bindings.entry(device).or_default().push(binding);
        self
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub struct GamepadBinding {
    pub source: GamepadBindingSource,
    pub button_behavior: ButtonInputBeheavior,
    // pub device: GamepadBindingDevice,
    pub unbounded: bool,
    pub premultiply_delta_time: bool,
    pub axis: InputAxis,
    pub axis_dir: InputAxisDirection,
}

impl GamepadBinding {
    pub fn button(button: GamepadButtonType) -> GamepadBinding {
        GamepadBinding {
            source: GamepadBindingSource::Button(button),
            // device: GamepadBindingDevice::Any,
            unbounded: false,
            premultiply_delta_time: false,
            button_behavior: default(),
            axis: default(),
            axis_dir: default(),
        }
    }

    pub fn button_just_pressed(mut self) -> Self {
        self.button_behavior = ButtonInputBeheavior::JustPressed;
        self
    }

    pub fn button_pressed(mut self) -> Self {
        self.button_behavior = ButtonInputBeheavior::Pressed;
        self
    }

    pub fn button_just_released(mut self) -> Self {
        self.button_behavior = ButtonInputBeheavior::JustReleased;
        self
    }

    pub fn axis(axis: GamepadAxisType) -> GamepadBinding {
        GamepadBinding {
            source: GamepadBindingSource::Axis(axis),
            // device: GamepadBindingDevice::Any,
            unbounded: false,
            premultiply_delta_time: false,
            button_behavior: default(),
            axis: default(),
            axis_dir: default(),
        }
    }

    // pub fn from_gamepad(mut self, gamepad: Gamepad) -> Self {
    //     self.device = GamepadBindingDevice::Gamepad(gamepad);
    //     self
    // }

    pub fn unbounded(mut self) -> Self {
        self.unbounded = true;
        self
    }

    pub fn x_axis(mut self) -> Self {
        self.axis = InputAxis::X;
        self
    }

    pub fn y_axis(mut self) -> Self {
        self.axis = InputAxis::Y;
        self
    }

    pub fn positive(mut self) -> Self {
        self.axis_dir = InputAxisDirection::Positive;
        self
    }

    pub fn negative(mut self) -> Self {
        self.axis_dir = InputAxisDirection::Negative;
        self
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum GamepadBindingSource {
    Axis(GamepadAxisType),
    Button(GamepadButtonType),
}
#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum GamepadBindingDevice {
    Any,
    Gamepad(Gamepad),
}

// struct GamePadOldVec2Value(Vec2);
