use std::time::Duration;

use atomicow::CowArc;
use bevy::{
    input::gamepad::{GamepadInput, GamepadRumbleIntensity, GamepadRumbleRequest},
    prelude::*,
};

use crate::{
    binding_modification::{
        BindingModifiactions, PremultiplyDeltaTimeSecondsModification, UnboundedModification,
    },
    prelude::RequestedSubactionPaths,
    subaction_paths::{SubactionPath, SubactionPathCreated, SubactionPathMap, SubactionPathStr},
    ActionSetEnabled, BoolActionValue, ButtonInputBeheavior, F32ActionValue, InActionSet,
    InputAxis, InputAxisDirection, SchminputSet, Vec2ActionValue,
};

pub struct GamepadPlugin;

/// Use the index of a gamepad in this resource in a subaction path to referebce
/// a specific gamepad
#[derive(Component, Clone, Debug, Deref)]
pub struct GamepadIdentifier(pub CowArc<'static, str>);

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
            v => {
                cmds.entity(e)
                    .insert(GamepadPathSelector::Gamepad(v.to_owned()));
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
    gamepads: Query<(Entity, &Gamepad, Option<&GamepadIdentifier>)>,
) {
    for (bindings, out, set, sub_paths) in &haptic_query {
        if !(set_query.get(set.0).is_ok_and(|v| v.0)) {
            continue;
        };
        for binding in bindings.bindings.iter() {
            for (gamepad, _, _) in gamepads.iter() {
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
                        for (gamepad, _, _) in gamepads.iter() {
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
                            let Some((gamepad, _)) = gamepads
                                .iter()
                                .filter_map(|(e, _, v)| Some((e, v?)))
                                .find(|(_, v)| v.as_ref() == gamepad.as_str())
                            else {
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
    gamepads: Query<(Entity, &Gamepad, Option<&GamepadIdentifier>)>,
    mut query: Query<(
        &GamepadBindings,
        &InActionSet,
        &RequestedSubactionPaths,
        &BindingModifiactions,
        Option<&mut BoolActionValue>,
        Option<&mut F32ActionValue>,
        Option<&mut Vec2ActionValue>,
    )>,
    set_query: Query<&ActionSetEnabled>,
    path_query: Query<(
        &GamepadPathSelector,
        Option<&GamepadPathTarget>,
        Option<&GamepadPathTargetSide>,
    )>,
    modification_query: Query<(
        Has<PremultiplyDeltaTimeSecondsModification>,
        Has<UnboundedModification>,
    )>,
    time: Res<Time>,
) {
    for (
        gamepad_bindings,
        set,
        sub_paths,
        modifications,
        mut bool_value,
        mut float_value,
        mut vec2_value,
    ) in &mut query
    {
        if !(set_query.get(set.0).is_ok_and(|v| v.0)) {
            continue;
        };

        let (pre_mul_delta_time_all, unbounded_all) = modifications
            .all_paths
            .as_ref()
            .and_then(|v| modification_query.get(v.0).ok())
            .unwrap_or_default();
        for binding in &gamepad_bindings.bindings {
            let (mut pre_mul_delta_time, mut unbounded) = (pre_mul_delta_time_all, unbounded_all);
            for (mod_sub_path, modification) in modifications.per_path.iter().copied() {
                let Ok((_, target, target_side)) = path_query.get(*mod_sub_path) else {
                    continue;
                };
                if let Some(target) = target {
                    if target.matches(&binding.source, target_side) {
                        let Ok((pre_mul, unbound)) = modification_query.get(*modification) else {
                            continue;
                        };
                        pre_mul_delta_time |= pre_mul;
                        unbounded |= unbound;
                    }
                }
            }
            for (_, gamepad, _) in gamepads.iter() {
                handle_gamepad_inputs(
                    gamepad,
                    binding,
                    bool_value.as_deref_mut(),
                    float_value.as_deref_mut(),
                    vec2_value.as_deref_mut(),
                    None,
                    &time,
                    pre_mul_delta_time,
                    unbounded,
                );
            }
        }
        for sub_path in sub_paths.iter() {
            let Ok((device, target, target_side)) = path_query.get(**sub_path) else {
                continue;
            };
            for binding in &gamepad_bindings.bindings {
                let (mut pre_mul_delta_time, mut unbounded) =
                    (pre_mul_delta_time_all, unbounded_all);
                for (mod_sub_path, modification) in modifications.per_path.iter().copied() {
                    let Ok((_, target, target_side)) = path_query.get(*mod_sub_path) else {
                        continue;
                    };
                    if let Some(target) = target {
                        if target.matches(&binding.source, target_side) {
                            let Ok((pre_mul, unbound)) = modification_query.get(*modification)
                            else {
                                continue;
                            };
                            pre_mul_delta_time |= pre_mul;
                            unbounded |= unbound;
                        }
                    }
                }
                if let Some(target) = target {
                    if !target.matches(&binding.source, target_side) {
                        continue;
                    }
                }
                match device {
                    GamepadPathSelector::All => {
                        for (_, gamepad, _) in gamepads.iter() {
                            handle_gamepad_inputs(
                                gamepad,
                                binding,
                                bool_value.as_deref_mut(),
                                float_value.as_deref_mut(),
                                vec2_value.as_deref_mut(),
                                Some(*sub_path),
                                &time,
                                pre_mul_delta_time,
                                unbounded,
                            );
                        }
                    }
                    GamepadPathSelector::Gamepad(gamepad) => {
                        let Some((gamepad, _)) = gamepads
                            .iter()
                            .filter_map(|(_, e, v)| Some((e, v?)))
                            .find(|(_, v)| v.as_ref() == gamepad.as_str())
                        else {
                            continue;
                        };
                        handle_gamepad_inputs(
                            gamepad,
                            binding,
                            bool_value.as_deref_mut(),
                            float_value.as_deref_mut(),
                            vec2_value.as_deref_mut(),
                            Some(*sub_path),
                            &time,
                            pre_mul_delta_time,
                            unbounded,
                        );
                    }
                };
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_gamepad_inputs(
    gamepad: &Gamepad,
    binding: &GamepadBinding,
    mut bool_value: Option<&mut BoolActionValue>,
    mut float_value: Option<&mut F32ActionValue>,
    mut vec2_value: Option<&mut Vec2ActionValue>,
    path: Option<SubactionPath>,
    time: &Time,
    pre_mul_delta_time: bool,
    unbounded: bool,
) {
    let delta_multiplier = match pre_mul_delta_time {
        true => time.delta_secs(),
        false => 1.0,
    };
    let Some(v) = (match unbounded {
        true => gamepad.get_unclamped(binding.source),
        false => gamepad.get(binding.source),
    }) else {
        warn!("gamepad.get returned None, idk what that means");
        return;
    };
    if let Some(bool_value) = bool_value.as_mut() {
        match path {
            Some(path) => *bool_value.0.entry_with_path(path).or_default() |= v > 0.1,
            None => *bool_value.0 |= v > 0.1,
        }
    }
    if let Some(float_value) = float_value.as_mut() {
        match path {
            Some(path) => {
                *float_value.0.entry_with_path(path).or_default() +=
                    v * binding.axis_dir.as_multipier() * delta_multiplier
            }
            None => *float_value.0 += v * binding.axis_dir.as_multipier() * delta_multiplier,
        }
    }
    if let Some(vec2_value) = vec2_value.as_mut() {
        match binding.axis {
            InputAxis::X => match path {
                Some(path) => {
                    vec2_value.0.entry_with_path(path).or_default().x +=
                        v * binding.axis_dir.as_multipier() * delta_multiplier
                }
                None => vec2_value.0.x += v * binding.axis_dir.as_multipier() * delta_multiplier,
            },
            InputAxis::Y => match path {
                Some(path) => {
                    vec2_value.0.entry_with_path(path).or_default().y +=
                        v * binding.axis_dir.as_multipier() * delta_multiplier
                }
                None => vec2_value.0.y += v * binding.axis_dir.as_multipier() * delta_multiplier,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum GamepadHapticType {
    Weak,
    Strong,
}

impl std::fmt::Display for GamepadHapticType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GamepadHapticType::Weak => "Weak",
            GamepadHapticType::Strong => "Strong",
        })
    }
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
    pub axis: InputAxis,
    pub axis_dir: InputAxisDirection,
}

impl GamepadBinding {
    pub fn new(source: GamepadBindingSource) -> GamepadBinding {
        GamepadBinding {
            source,
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

// Mashup of bevys GamepadButtonType and GamepadAxisType
#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum GamepadBindingSource {
    /// The horizontal value of the left stick.
    LeftStickX,
    /// The vertical value of the left stick.
    LeftStickY,
    /// The horizontal value of the right stick.
    RightStickX,
    /// The vertical value of the right stick.
    RightStickY,
    /// The bottom action button of the action pad (i.e. PS: Cross, Xbox: A).
    South,
    /// The right action button of the action pad (i.e. PS: Circle, Xbox: B).
    East,
    /// The upper action button of the action pad (i.e. PS: Triangle, Xbox: Y).
    North,
    /// The left action button of the action pad (i.e. PS: Square, Xbox: X).
    West,
    /// The primary left trigger.
    LeftTrigger,
    /// The secondary left trigger.
    LeftSecondaryTrigger,
    /// The primary right trigger.
    RightTrigger,
    /// The secondary right trigger.
    RightSecondaryTrigger,
    /// The left thumb stick button.
    LeftStickClick,
    /// The right thumb stick button.
    RightStickClick,
    /// The up button of the D-Pad.
    DPadUp,
    /// The down button of the D-Pad.
    DPadDown,
    /// The left button of the D-Pad.
    DPadLeft,
    /// The right button of the D-Pad.
    DPadRight,
    /// The select button.
    Select,
    /// The start button.
    Start,
    /// The mode button.
    Mode,

    /// The value of the left `Z` button.
    LeftZ,
    /// The value of the right `Z` button.
    RightZ,
    /// The C button.
    C,
    /// The Z button.
    Z,
    /// Non-standard support for other axis types (i.e. HOTAS sliders, potentiometers, etc).
    OtherAxis(u8),
    /// Miscellaneous buttons, considered non-standard (i.e. Extra buttons on a flight stick that do not have a gamepad equivalent).
    OtherButton(u8),
}
impl From<GamepadBindingSource> for GamepadInput {
    fn from(value: GamepadBindingSource) -> Self {
        match (value.as_axis_type(), value.as_button_type()) {
            (None, Some(v)) => Self::Button(v),
            (Some(v), None) => Self::Axis(v),
            (Some(_), Some(_)) | (None, None) => unreachable!(),
        }
    }
}
impl std::fmt::Display for GamepadBindingSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GamepadBindingSource::LeftTrigger => "Left Trigger",
            GamepadBindingSource::RightTrigger => "Right Trigger",
            GamepadBindingSource::LeftStickX => "Left Stick X",
            GamepadBindingSource::LeftStickY => "Left Stick Y",
            GamepadBindingSource::RightStickX => "Right Stick X",
            GamepadBindingSource::RightStickY => "Right Stick Y",
            GamepadBindingSource::OtherAxis(axis) => return f.write_str(&format!("Axis {}", axis)),
            GamepadBindingSource::OtherButton(button) => {
                return f.write_str(&format!("Button {}", button))
            }
            GamepadBindingSource::South => "South",
            GamepadBindingSource::East => "East",
            GamepadBindingSource::North => "North",
            GamepadBindingSource::West => "West",
            GamepadBindingSource::LeftSecondaryTrigger => "Left Secondary Trigger",
            GamepadBindingSource::RightSecondaryTrigger => "Right Secondary Trigger",
            GamepadBindingSource::LeftStickClick => "Left Stick Click",
            GamepadBindingSource::RightStickClick => "Right Stick Click",
            GamepadBindingSource::DPadUp => "Dpad Up",
            GamepadBindingSource::DPadDown => "Dpad Down",
            GamepadBindingSource::DPadLeft => "Dpad Left",
            GamepadBindingSource::DPadRight => "Dpad Right",
            GamepadBindingSource::Select => "Select",
            GamepadBindingSource::Start => "Start",
            GamepadBindingSource::Mode => "Mode",
            GamepadBindingSource::LeftZ => "Left Z Axis",
            GamepadBindingSource::RightZ => "Right Z Axis",
            GamepadBindingSource::C => "C Button",
            GamepadBindingSource::Z => "Z Button",
        })
    }
}

impl GamepadBindingSource {
    pub fn as_axis_type(&self) -> Option<GamepadAxis> {
        Some(match self {
            GamepadBindingSource::LeftStickX => GamepadAxis::LeftStickX,
            GamepadBindingSource::LeftStickY => GamepadAxis::LeftStickY,
            GamepadBindingSource::RightStickX => GamepadAxis::RightStickX,
            GamepadBindingSource::RightStickY => GamepadAxis::RightStickY,
            GamepadBindingSource::LeftZ => GamepadAxis::LeftZ,
            GamepadBindingSource::RightZ => GamepadAxis::RightZ,
            GamepadBindingSource::OtherAxis(v) => GamepadAxis::Other(*v),
            _ => return None,
        })
    }
    pub fn from_axis_type(axis: &GamepadAxis) -> GamepadBindingSource {
        match axis {
            GamepadAxis::LeftStickX => GamepadBindingSource::LeftStickX,
            GamepadAxis::LeftStickY => GamepadBindingSource::LeftStickY,
            GamepadAxis::RightStickX => GamepadBindingSource::RightStickX,
            GamepadAxis::RightStickY => GamepadBindingSource::RightStickY,
            GamepadAxis::LeftZ => GamepadBindingSource::LeftZ,
            GamepadAxis::RightZ => GamepadBindingSource::RightZ,
            GamepadAxis::Other(v) => GamepadBindingSource::OtherAxis(*v),
        }
    }

    pub fn as_button_type(&self) -> Option<GamepadButton> {
        Some(match self {
            GamepadBindingSource::South => GamepadButton::South,
            GamepadBindingSource::East => GamepadButton::East,
            GamepadBindingSource::North => GamepadButton::North,
            GamepadBindingSource::West => GamepadButton::West,
            GamepadBindingSource::LeftTrigger => GamepadButton::LeftTrigger2,
            GamepadBindingSource::LeftSecondaryTrigger => GamepadButton::LeftTrigger,
            GamepadBindingSource::RightTrigger => GamepadButton::RightTrigger2,
            GamepadBindingSource::RightSecondaryTrigger => GamepadButton::RightTrigger,
            GamepadBindingSource::LeftStickClick => GamepadButton::LeftThumb,
            GamepadBindingSource::RightStickClick => GamepadButton::RightThumb,
            GamepadBindingSource::DPadUp => GamepadButton::DPadUp,
            GamepadBindingSource::DPadDown => GamepadButton::DPadDown,
            GamepadBindingSource::DPadLeft => GamepadButton::DPadLeft,
            GamepadBindingSource::DPadRight => GamepadButton::DPadRight,
            GamepadBindingSource::Select => GamepadButton::Select,
            GamepadBindingSource::Start => GamepadButton::Start,
            GamepadBindingSource::Mode => GamepadButton::Mode,
            GamepadBindingSource::C => GamepadButton::C,
            GamepadBindingSource::Z => GamepadButton::Z,
            GamepadBindingSource::OtherButton(v) => GamepadButton::Other(*v),
            _ => return None,
        })
    }
    pub fn from_button_type(button: &GamepadButton) -> GamepadBindingSource {
        match button {
            GamepadButton::South => GamepadBindingSource::South,
            GamepadButton::East => GamepadBindingSource::East,
            GamepadButton::North => GamepadBindingSource::North,
            GamepadButton::West => GamepadBindingSource::West,
            GamepadButton::LeftTrigger2 => GamepadBindingSource::LeftTrigger,
            GamepadButton::LeftTrigger => GamepadBindingSource::LeftSecondaryTrigger,
            GamepadButton::RightTrigger2 => GamepadBindingSource::RightTrigger,
            GamepadButton::RightTrigger => GamepadBindingSource::RightSecondaryTrigger,
            GamepadButton::LeftThumb => GamepadBindingSource::LeftStickClick,
            GamepadButton::RightThumb => GamepadBindingSource::RightStickClick,
            GamepadButton::DPadUp => GamepadBindingSource::DPadUp,
            GamepadButton::DPadDown => GamepadBindingSource::DPadDown,
            GamepadButton::DPadLeft => GamepadBindingSource::DPadLeft,
            GamepadButton::DPadRight => GamepadBindingSource::DPadRight,
            GamepadButton::Select => GamepadBindingSource::Select,
            GamepadButton::Start => GamepadBindingSource::Start,
            GamepadButton::Mode => GamepadBindingSource::Mode,
            GamepadButton::C => GamepadBindingSource::C,
            GamepadButton::Z => GamepadBindingSource::Z,
            GamepadButton::Other(v) => GamepadBindingSource::OtherButton(*v),
        }
    }
}

#[derive(Clone, Debug, Reflect, PartialEq, Eq, Hash, Component)]
pub enum GamepadPathSelector {
    All,
    Gamepad(String),
}

impl GamepadPathTarget {
    pub fn matches(
        &self,
        source: &GamepadBindingSource,
        side: Option<&GamepadPathTargetSide>,
    ) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self, side, source) {
            (GamepadPathTarget::Thumbstick, None, GamepadBindingSource::LeftStickX) => true,
            (GamepadPathTarget::Thumbstick, None, GamepadBindingSource::LeftStickY) => true,
            (GamepadPathTarget::Thumbstick, None, GamepadBindingSource::RightStickX) => true,
            (GamepadPathTarget::Thumbstick, None, GamepadBindingSource::RightStickY) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Left),
                GamepadBindingSource::LeftStickX,
            ) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Left),
                GamepadBindingSource::LeftStickY,
            ) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Right),
                GamepadBindingSource::RightStickX,
            ) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Right),
                GamepadBindingSource::RightStickY,
            ) => true,
            (GamepadPathTarget::Trigger, None, GamepadBindingSource::LeftTrigger) => true,
            (GamepadPathTarget::Trigger, None, GamepadBindingSource::RightTrigger) => true,
            (
                GamepadPathTarget::Trigger,
                Some(GamepadPathTargetSide::Left),
                GamepadBindingSource::LeftTrigger,
            ) => true,
            (
                GamepadPathTarget::Trigger,
                Some(GamepadPathTargetSide::Right),
                GamepadBindingSource::RightTrigger,
            ) => true,
            (
                GamepadPathTarget::SecondaryTrigger,
                None,
                GamepadBindingSource::LeftSecondaryTrigger,
            ) => true,
            (
                GamepadPathTarget::SecondaryTrigger,
                None,
                GamepadBindingSource::RightSecondaryTrigger,
            ) => true,
            (
                GamepadPathTarget::SecondaryTrigger,
                Some(GamepadPathTargetSide::Left),
                GamepadBindingSource::LeftSecondaryTrigger,
            ) => true,
            (
                GamepadPathTarget::SecondaryTrigger,
                Some(GamepadPathTargetSide::Right),
                GamepadBindingSource::RightSecondaryTrigger,
            ) => true,
            (GamepadPathTarget::Thumbstick, None, GamepadBindingSource::LeftStickClick) => true,
            (GamepadPathTarget::Thumbstick, None, GamepadBindingSource::RightStickClick) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Left),
                GamepadBindingSource::LeftStickClick,
            ) => true,
            (
                GamepadPathTarget::Thumbstick,
                Some(GamepadPathTargetSide::Right),
                GamepadBindingSource::RightStickClick,
            ) => true,
            (GamepadPathTarget::Buttons, None, GamepadBindingSource::South) => true,
            (GamepadPathTarget::Buttons, None, GamepadBindingSource::East) => true,
            (GamepadPathTarget::Buttons, None, GamepadBindingSource::North) => true,
            (GamepadPathTarget::Buttons, None, GamepadBindingSource::West) => true,
            (GamepadPathTarget::Buttons, Some(_), GamepadBindingSource::South) => true,
            (GamepadPathTarget::Buttons, Some(_), GamepadBindingSource::East) => true,
            (GamepadPathTarget::Buttons, Some(_), GamepadBindingSource::North) => true,
            (GamepadPathTarget::Buttons, Some(_), GamepadBindingSource::West) => true,
            (GamepadPathTarget::Dpad, None, GamepadBindingSource::DPadUp) => true,
            (GamepadPathTarget::Dpad, None, GamepadBindingSource::DPadDown) => true,
            (GamepadPathTarget::Dpad, None, GamepadBindingSource::DPadLeft) => true,
            (GamepadPathTarget::Dpad, None, GamepadBindingSource::DPadRight) => true,
            (GamepadPathTarget::Dpad, Some(_), GamepadBindingSource::DPadUp) => true,
            (GamepadPathTarget::Dpad, Some(_), GamepadBindingSource::DPadDown) => true,
            (GamepadPathTarget::Dpad, Some(_), GamepadBindingSource::DPadLeft) => true,
            (GamepadPathTarget::Dpad, Some(_), GamepadBindingSource::DPadRight) => true,
            _ => false,
        }
    }
}
