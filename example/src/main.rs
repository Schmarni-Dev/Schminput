use std::time::Duration;

use bevy::prelude::Camera3dBundle;
use bevy::prelude::*;
use bevy_schminput::{
    gamepad::{
        GamepadBinding, GamepadBindingDevice, GamepadBindings, GamepadHapticOutput,
        GamepadHapticOutputBindings,
    },
    keyboard::KeyboardBindings,
    mouse::MouseBindings,
    ActionHeaderBuilder, BoolActionValue, DefaultSchmugins, Vec2ActionValue,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultSchmugins);
    app.add_systems(Startup, setup);
    app.add_systems(Update, run);

    app.run();
}

#[derive(Component, Clone, Copy)]
struct MoveAction;
#[derive(Component, Clone, Copy)]
struct LookAction;
#[derive(Component, Clone, Copy)]
struct JumpAction;
#[derive(Component, Clone, Copy)]
struct JumpHapticAction;

fn setup(mut cmds: Commands) {
    let mut move_e = ActionHeaderBuilder::new("move")
        .with_name("Move")
        .build(&mut cmds);
    use bevy_schminput::keyboard::KeyboardBinding as KbB;
    move_e.insert((
        Vec2ActionValue::default(),
        KeyboardBindings::default()
            .add_binding(KbB::new(KeyCode::KeyW).y_axis().positive_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyS).y_axis().negative_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyA).x_axis().negative_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyD).x_axis().positive_axis_dir()),
        GamepadBindings::default()
            .add_binding(
                GamepadBindingDevice::Any,
                GamepadBinding::axis(GamepadAxisType::LeftStickX)
                    .x_axis()
                    .positive(),
            )
            .add_binding(
                GamepadBindingDevice::Any,
                GamepadBinding::axis(GamepadAxisType::LeftStickY)
                    .y_axis()
                    .positive(),
            ),
        MoveAction,
    ));
    let mut look_e = ActionHeaderBuilder::new("look")
        .with_name("Look")
        .build(&mut cmds);
    look_e.insert((
        Vec2ActionValue::default(),
        MouseBindings::default().delta_motion(),
        GamepadBindings::default()
            .add_binding(
                GamepadBindingDevice::Any,
                GamepadBinding::axis(GamepadAxisType::RightStickX)
                    .x_axis()
                    .positive(),
            )
            .add_binding(
                GamepadBindingDevice::Any,
                GamepadBinding::axis(GamepadAxisType::RightStickY)
                    .y_axis()
                    .positive(),
            ),
        LookAction,
    ));
    ActionHeaderBuilder::new("jump")
        .with_name("Jump")
        .build(&mut cmds)
        .insert((
            JumpAction,
            BoolActionValue::default(),
            GamepadBindings::default().add_binding(
                GamepadBindingDevice::Any,
                GamepadBinding::button(GamepadButtonType::South),
            ),
            KeyboardBindings::default().add_binding(KbB::new(KeyCode::Space)),
        ));
    ActionHeaderBuilder::new("jump_haptic")
        .with_name("Jump Haptic Feedback")
        .build(&mut cmds)
        .insert((
            JumpHapticAction,
            GamepadHapticOutput::default(),
            GamepadHapticOutputBindings::default().weak(GamepadBindingDevice::Any),
        ));
    cmds.spawn(Camera3dBundle::default());
}

fn run(
    move_action: Query<&Vec2ActionValue, With<MoveAction>>,
    look_action: Query<&Vec2ActionValue, With<LookAction>>,
    jump_action: Query<&BoolActionValue, With<JumpAction>>,
    mut jump_haptic_action: Query<&mut GamepadHapticOutput, With<JumpHapticAction>>,
) {
    for action in move_action.into_iter() {
        info!("move: {:?}", action);
    }
    for action in look_action.into_iter() {
        info!("look: {:?}", action);
    }
    for action in jump_action.into_iter() {
        info!("jump: {:?}", action);
        if action.0 {
            //panics if action doesn't exist
            jump_haptic_action
                .single_mut()
                .add(Duration::from_millis(50), 1.0);
        }
    }
}

// fn setup(
//     mut commands: Commands,
// ) {
//     let example = ExampleAction::default();
//     let bool = BoolAction::default();
//     keyboard.add_binding(
//         &bool,
//         KeyboardBinding::Simple(KeyBinding::JustPressed(KeyCode::KeyC)),
//     );
//     keyboard.add_binding(
//         &example,
//         KeyboardBinding::Dpad {
//             up: KeyBinding::Held(KeyCode::KeyW),
//             down: KeyBinding::Held(KeyCode::KeyS),
//             left: KeyBinding::Held(KeyCode::KeyA),
//             right: KeyBinding::Held(KeyCode::KeyD),
//         },
//     );
//     mouse.add_binding(&bool, MouseBinding::JustPressed(MouseButton::Left));
//     mouse.add_binding(&bool, MouseBinding::JustReleased(MouseButton::Left));
//     mouse.add_binding(&bool, MouseBinding::Held(MouseButton::Right));
//     commands.spawn((example, bool));
// }
