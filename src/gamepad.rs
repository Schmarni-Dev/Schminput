use std::time::Duration;

use bevy::{
    input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest},
    prelude::*,
};

use crate::{
    prelude::RequestedSubactionPaths,
    subaction_paths::{SubactionPath, SubactionPathCreated, SubactionPathMap, SubactionPathStr},
    ActionSetEnabled, BoolActionValue, ButtonInputBeheavior, F32ActionValue, InActionSet,
    InputAxis, InputAxisDirection, SchminputSet, Vec2ActionValue,
};

pub struct GamepadPlugin;

/// Use the index of a gamepad in this resource in a subaction path to referebce
/// a specific gamepad
#[derive(Default, Resource, Clone)]
pub struct GamepadRegistery(pub Vec<Gamepad>);

impl Plugin for GamepadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GamepadRegistery>();
        app.add_systems(
            PreUpdate,
            sync_actions.in_set(SchminputSet::SyncInputActions),
        );
        app.add_systems(PreUpdate, clear_haptic.in_set(SchminputSet::ClearValues));
        app.add_systems(
            PostUpdate,
            sync_haptics.in_set(SchminputSet::SyncOutputActions),
        );
        app.add_systems(
            PreUpdate,
            handle_new_subaction_paths.in_set(SchminputSet::HandleNewSubactionPaths),
        );
    }
}

fn handle_new_subaction_paths(
    query: Query<&SubactionPathStr>,
    mut reader: EventReader<SubactionPathCreated>,
    mut cmds: Commands,
) {
    for (e, str) in reader
        .read()
        .filter_map(|e| Some((e.0 .0, query.get(e.0 .0).ok()?)))
    {
        let Some(str) = str.0.strip_prefix("/gamepad") else {
            continue;
        };

        let (index_str, path_str) = {
            let Some(stripped_str) = str.strip_prefix('/') else {
                cmds.entity(e).insert(GamepadPathSelector::All);
                continue;
            };
            stripped_str.split_once('/').unwrap_or((stripped_str, ""))
        };

        match index_str {
            "*" | "" => {
                cmds.entity(e).insert(GamepadPathSelector::All);
            }
            v if v.parse::<usize>().is_ok() => {
                let Ok(num) = v.parse::<usize>() else {
                    unreachable!()
                };

                cmds.entity(e).insert(GamepadPathSelector::Gamepad(num));
            }
            v => {
                error!(
                    "unable to parse gamepad id, use a positive integer or *: {}",
                    v
                );
                continue;
            }
        }

        match path_str {
            "" => {}
            "thumbstick" | "thumbstick/*" => {
                cmds.entity(e).insert(GamepadPathTarget::Thumbstick);
            }
            "thumbstick/left" => {
                cmds.entity(e).insert(GamepadPathTarget::Thumbstick);
                cmds.entity(e).insert(GamepadPathTargetSide::Left);
            }
            "thumbstick/right" => {
                cmds.entity(e).insert(GamepadPathTarget::Thumbstick);
                cmds.entity(e).insert(GamepadPathTargetSide::Right);
            }
            "dpad" => {
                cmds.entity(e).insert(GamepadPathTarget::Dpad);
            }
            "buttons" => {
                cmds.entity(e).insert(GamepadPathTarget::Buttons);
            }
            "trigger" | "trigger/*" => {
                cmds.entity(e).insert(GamepadPathTarget::Trigger);
            }
            "trigger/left" => {
                cmds.entity(e).insert(GamepadPathTarget::Trigger);
                cmds.entity(e).insert(GamepadPathTargetSide::Left);
            }
            "trigger/right" => {
                cmds.entity(e).insert(GamepadPathTarget::Trigger);
                cmds.entity(e).insert(GamepadPathTargetSide::Right);
            }
            "secondary_trigger" | "secondray_trigger/*" => {
                cmds.entity(e).insert(GamepadPathTarget::SecondaryTrigger);
            }
            "secondary_trigger/left" => {
                cmds.entity(e).insert(GamepadPathTarget::SecondaryTrigger);
                cmds.entity(e).insert(GamepadPathTargetSide::Left);
            }
            "secondary_trigger/right" => {
                cmds.entity(e).insert(GamepadPathTarget::SecondaryTrigger);
                cmds.entity(e).insert(GamepadPathTargetSide::Right);
            }
            v => {
                error!("invalid path: {}", v);
                continue;
            }
        }
    }
}

// Might need a better name than Behavior
#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash, Component)]
pub enum GamepadPathTarget {
    Thumbstick,
    Trigger,
    SecondaryTrigger,
    Buttons,
    Dpad,
}
#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash, Component)]
pub enum GamepadPathTargetSide {
    Left,
    Right,
}

fn clear_haptic(mut query: Query<&mut GamepadHapticOutput>) {
    for mut out in &mut query {
        out.haptic_feedbacks.clear();
    }
}

fn sync_haptics(
    mut gamepad_haptic_event: EventWriter<GamepadRumbleRequest>,
    haptic_query: Query<(
        &GamepadHapticOutputBindings,
        &GamepadHapticOutput,
        &InActionSet,
        &RequestedSubactionPaths,
    )>,
    path_query: Query<&GamepadPathSelector>,
    set_query: Query<&ActionSetEnabled>,
    gamepad_registry: Res<GamepadRegistery>,
    gamepads: Res<Gamepads>,
) {
    for (bindings, out, set, sub_paths) in &haptic_query {
        if !(set_query.get(set.0).is_ok_and(|v| v.0)) {
            continue;
        };
        for binding in bindings.bindings.iter() {
            for gamepad in gamepads.iter() {
                for e in &out.haptic_feedbacks.any {
                    gamepad_haptic_event.send(match e {
                        GamepadHapticValue::Add {
                            duration,
                            intensity,
                        } => GamepadRumbleRequest::Add {
                            duration: *duration,
                            intensity: binding.as_rumble_intensity(*intensity),
                            gamepad,
                        },
                        GamepadHapticValue::Stop => GamepadRumbleRequest::Stop { gamepad },
                    });
                }
            }
        }
        for sub_path in sub_paths.iter() {
            let Ok(device) = path_query.get(**sub_path) else {
                continue;
            };
            for binding in bindings.bindings.iter() {
                match device {
                    GamepadPathSelector::All => {
                        for gamepad in gamepads.iter() {
                            for e in out
                                .haptic_feedbacks
                                .get_with_path(sub_path)
                                .unwrap_or(&Vec::new())
                            {
                                gamepad_haptic_event.send(match e {
                                    GamepadHapticValue::Add {
                                        duration,
                                        intensity,
                                    } => GamepadRumbleRequest::Add {
                                        duration: *duration,
                                        intensity: binding.as_rumble_intensity(*intensity),
                                        gamepad,
                                    },
                                    GamepadHapticValue::Stop => {
                                        GamepadRumbleRequest::Stop { gamepad }
                                    }
                                });
                            }
                        }
                    }
                    GamepadPathSelector::Gamepad(gamepad) => {
                        for e in out
                            .haptic_feedbacks
                            .get_with_path(sub_path)
                            .unwrap_or(&Vec::new())
                        {
                            let Some(gamepad) = gamepad_registry.0.get(*gamepad).copied() else {
                                continue;
                            };
                            gamepad_haptic_event.send(match e {
                                GamepadHapticValue::Add {
                                    duration,
                                    intensity,
                                } => GamepadRumbleRequest::Add {
                                    duration: *duration,
                                    intensity: binding.as_rumble_intensity(*intensity),
                                    gamepad,
                                },
                                GamepadHapticValue::Stop => GamepadRumbleRequest::Stop { gamepad },
                            });
                        }
                    }
                };
            }
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
        &InActionSet,
        &RequestedSubactionPaths,
        Option<&mut BoolActionValue>,
        Option<&mut F32ActionValue>,
        Option<&mut Vec2ActionValue>,
    )>,
    gamepad_registry: Res<GamepadRegistery>,
    set_query: Query<&ActionSetEnabled>,
    path_query: Query<(
        &GamepadPathSelector,
        Option<&GamepadPathTarget>,
        Option<&GamepadPathTargetSide>,
    )>,
    time: Res<Time>,
) {
    for (gamepad_bindings, set, sub_paths, mut bool_value, mut float_value, mut vec2_value) in
        &mut query
    {
        if !(set_query.get(set.0).is_ok_and(|v| v.0)) {
            continue;
        };
        for binding in &gamepad_bindings.bindings {
            for gamepad in gamepads.iter() {
                handle_gamepad_inputs(
                    gamepad,
                    binding,
                    &axis,
                    &button,
                    bool_value.as_deref_mut(),
                    float_value.as_deref_mut(),
                    vec2_value.as_deref_mut(),
                    None,
                    &time,
                );
            }
        }
        for sub_path in sub_paths.iter() {
            let Ok((device, target, target_side)) = path_query.get(**sub_path) else {
                continue;
            };
            for binding in &gamepad_bindings.bindings {
                if let Some(target) = target {
                    if !target.matches(&binding.source, target_side) {
                        continue;
                    }
                }
                match *device {
                    GamepadPathSelector::All => {
                        for gamepad in gamepads.iter() {
                            handle_gamepad_inputs(
                                gamepad,
                                binding,
                                &axis,
                                &button,
                                bool_value.as_deref_mut(),
                                float_value.as_deref_mut(),
                                vec2_value.as_deref_mut(),
                                Some(*sub_path),
                                &time,
                            );
                        }
                    }
                    GamepadPathSelector::Gamepad(gamepad) => {
                        let Some(gamepad) = gamepad_registry.0.get(gamepad).copied() else {
                            continue;
                        };
                        handle_gamepad_inputs(
                            gamepad,
                            binding,
                            &axis,
                            &button,
                            bool_value.as_deref_mut(),
                            float_value.as_deref_mut(),
                            vec2_value.as_deref_mut(),
                            Some(*sub_path),
                            &time,
                        );
                    }
                };
            }
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
    path: Option<SubactionPath>,
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
            info!(v);
            if let Some(bool_value) = bool_value {
                match path {
                    Some(path) => *bool_value.0.entry_with_path(path).or_default() |= v > 0.0,
                    None => *bool_value.0 |= v > 0.0,
                }
            }
            if let Some(float_value) = float_value {
                match path {
                    Some(path) => {
                        *float_value.0.entry_with_path(path).or_default() +=
                            v * binding.axis_dir.as_multipier() * delta_multiplier
                    }
                    None => {
                        *float_value.0 += v * binding.axis_dir.as_multipier() * delta_multiplier
                    }
                }
            }
            if let Some(vec2_value) = vec2_value {
                match binding.axis {
                    InputAxis::X => match path {
                        Some(path) => {
                            vec2_value.0.entry_with_path(path).or_default().x +=
                                v * binding.axis_dir.as_multipier() * delta_multiplier
                        }
                        None => {
                            vec2_value.0.x += v * binding.axis_dir.as_multipier() * delta_multiplier
                        }
                    },
                    InputAxis::Y => match path {
                        Some(path) => {
                            vec2_value.0.entry_with_path(path).or_default().y +=
                                v * binding.axis_dir.as_multipier() * delta_multiplier
                        }
                        None => {
                            vec2_value.0.y += v * binding.axis_dir.as_multipier() * delta_multiplier
                        }
                    },
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
                *bool_value.0 |= v;
            }
            if let Some(float_value) = float_value {
                *float_value.0 +=
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
    pub bindings: Vec<GamepadHapticType>,
}

impl GamepadHapticOutputBindings {
    pub fn new() -> GamepadHapticOutputBindings {
        GamepadHapticOutputBindings::default()
    }
    pub fn weak(mut self) -> Self {
        self.bindings.push(GamepadHapticType::Weak);
        self
    }
    pub fn strong(mut self) -> Self {
        self.bindings.push(GamepadHapticType::Strong);
        self
    }
}

#[derive(Clone, Component, Debug, Reflect, Default)]
pub struct GamepadHapticOutput {
    pub haptic_feedbacks: SubactionPathMap<Vec<GamepadHapticValue>>,
}

impl GamepadHapticOutput {
    pub fn add_with_path(
        &mut self,
        duration: Duration,
        intensity: f32,
        path: SubactionPath,
    ) -> &mut Self {
        self.haptic_feedbacks
            .entry_with_path(path)
            .or_default()
            .push(GamepadHapticValue::Add {
                duration,
                intensity,
            });
        self
    }
    pub fn stop_with_path(&mut self, path: SubactionPath) -> &mut Self {
        self.haptic_feedbacks
            .entry_with_path(path)
            .or_default()
            .push(GamepadHapticValue::Stop);
        self
    }
    pub fn add(&mut self, duration: Duration, intensity: f32) -> &mut Self {
        self.haptic_feedbacks.any.push(GamepadHapticValue::Add {
            duration,
            intensity,
        });
        self
    }
    pub fn stop(&mut self) -> &mut Self {
        self.haptic_feedbacks.any.push(GamepadHapticValue::Stop);
        self
    }
}

#[derive(Clone, Component, Debug, Reflect, Default)]
pub struct GamepadBindings {
    pub bindings: Vec<GamepadBinding>,
}

impl GamepadBindings {
    pub fn add_binding(mut self, binding: GamepadBinding) -> Self {
        self.bindings.push(binding);
        self
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub struct GamepadBinding {
    pub source: GamepadBindingSource,
    pub button_behavior: ButtonInputBeheavior,
    pub unbounded: bool,
    pub premultiply_delta_time: bool,
    pub axis: InputAxis,
    pub axis_dir: InputAxisDirection,
}

impl GamepadBinding {
    pub fn button(button: GamepadButtonType) -> GamepadBinding {
        GamepadBinding {
            source: GamepadBindingSource::Button(button),
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
            unbounded: false,
            premultiply_delta_time: false,
            button_behavior: default(),
            axis: default(),
            axis_dir: default(),
        }
    }

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
impl std::fmt::Display for GamepadBindingSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GamepadBindingSource::Axis(GamepadAxisType::LeftZ) => "Left Trigger",
            GamepadBindingSource::Axis(GamepadAxisType::RightZ) => "Right Trigger",
            GamepadBindingSource::Axis(GamepadAxisType::LeftStickX) => "Left Stick X",
            GamepadBindingSource::Axis(GamepadAxisType::LeftStickY) => "Left Stick Y",
            GamepadBindingSource::Axis(GamepadAxisType::RightStickX) => "Right Stick X",
            GamepadBindingSource::Axis(GamepadAxisType::RightStickY) => "Right Stick Y",
            GamepadBindingSource::Axis(GamepadAxisType::Other(axis)) => {
                return f.write_str(&format!("Axis {}", axis))
            }
            GamepadBindingSource::Button(GamepadButtonType::Other(button)) => {
                return f.write_str(&format!("Button {}", button))
            }

            GamepadBindingSource::Button(v) => return v.debug(f),
        })
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash, Component)]
pub enum GamepadPathSelector {
    All,
    Gamepad(usize),
}

impl GamepadPathTarget {
    pub fn matches(
        &self,
        source: &GamepadBindingSource,
        side: Option<&GamepadPathTargetSide>,
    ) -> bool {
        match source {
            GamepadBindingSource::Axis(axis) => self.axis_matches(axis, side),
            GamepadBindingSource::Button(button) => self.button_matches(button, side),
        }
    }
    pub fn axis_matches(
        &self,
        axis: &GamepadAxisType,
        side: Option<&GamepadPathTargetSide>,
    ) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self, side, axis) {
            (GamepadPathTarget::Thumbstick, None, GamepadAxisType::LeftStickX) => true,
            (GamepadPathTarget::Thumbstick, None, GamepadAxisType::LeftStickY) => true,
            (GamepadPathTarget::Thumbstick, None, GamepadAxisType::RightStickX) => true,
            (GamepadPathTarget::Thumbstick, None, GamepadAxisType::RightStickY) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Left),
                GamepadAxisType::LeftStickX,
            ) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Left),
                GamepadAxisType::LeftStickY,
            ) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Right),
                GamepadAxisType::RightStickX,
            ) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Right),
                GamepadAxisType::RightStickY,
            ) => true,
            (GamepadPathTarget::Trigger, None, GamepadAxisType::LeftZ) => true,
            (GamepadPathTarget::Trigger, None, GamepadAxisType::RightZ) => true,
            (
                GamepadPathTarget::Trigger,
                Some(GamepadPathTargetSide::Left),
                GamepadAxisType::LeftZ,
            ) => true,
            (
                GamepadPathTarget::Trigger,
                Some(GamepadPathTargetSide::Right),
                GamepadAxisType::RightZ,
            ) => true,
            _ => false,
        }
    }

    pub fn button_matches(
        &self,
        button: &GamepadButtonType,
        side: Option<&GamepadPathTargetSide>,
    ) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self, side, button) {
            (GamepadPathTarget::Trigger, None, GamepadButtonType::LeftTrigger) => true,
            (GamepadPathTarget::Trigger, None, GamepadButtonType::RightTrigger) => true,
            (
                GamepadPathTarget::Trigger,
                Some(GamepadPathTargetSide::Left),
                GamepadButtonType::LeftTrigger,
            ) => true,
            (
                GamepadPathTarget::Trigger,
                Some(GamepadPathTargetSide::Right),
                GamepadButtonType::RightTrigger,
            ) => true,
            (GamepadPathTarget::SecondaryTrigger, None, GamepadButtonType::LeftTrigger2) => true,
            (GamepadPathTarget::SecondaryTrigger, None, GamepadButtonType::RightTrigger2) => true,
            (
                GamepadPathTarget::SecondaryTrigger,
                Some(GamepadPathTargetSide::Left),
                GamepadButtonType::LeftTrigger2,
            ) => true,
            (
                GamepadPathTarget::SecondaryTrigger,
                Some(GamepadPathTargetSide::Right),
                GamepadButtonType::RightTrigger2,
            ) => true,
            (GamepadPathTarget::Thumbstick, None, GamepadButtonType::LeftThumb) => true,
            (GamepadPathTarget::Thumbstick, None, GamepadButtonType::RightThumb) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Left),
                GamepadButtonType::LeftThumb,
            ) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Right),
                GamepadButtonType::RightThumb,
            ) => true,
            (GamepadPathTarget::Buttons, None, GamepadButtonType::South) => true,
            (GamepadPathTarget::Buttons, None, GamepadButtonType::East) => true,
            (GamepadPathTarget::Buttons, None, GamepadButtonType::North) => true,
            (GamepadPathTarget::Buttons, None, GamepadButtonType::West) => true,
            (GamepadPathTarget::Buttons, Some(_), GamepadButtonType::South) => true,
            (GamepadPathTarget::Buttons, Some(_), GamepadButtonType::East) => true,
            (GamepadPathTarget::Buttons, Some(_), GamepadButtonType::North) => true,
            (GamepadPathTarget::Buttons, Some(_), GamepadButtonType::West) => true,
            (GamepadPathTarget::Dpad, None, GamepadButtonType::DPadUp) => true,
            (GamepadPathTarget::Dpad, None, GamepadButtonType::DPadDown) => true,
            (GamepadPathTarget::Dpad, None, GamepadButtonType::DPadLeft) => true,
            (GamepadPathTarget::Dpad, None, GamepadButtonType::DPadRight) => true,
            (GamepadPathTarget::Dpad, Some(_), GamepadButtonType::DPadUp) => true,
            (GamepadPathTarget::Dpad, Some(_), GamepadButtonType::DPadDown) => true,
            (GamepadPathTarget::Dpad, Some(_), GamepadButtonType::DPadLeft) => true,
            (GamepadPathTarget::Dpad, Some(_), GamepadButtonType::DPadRight) => true,
            _ => false,
        }
    }
}
