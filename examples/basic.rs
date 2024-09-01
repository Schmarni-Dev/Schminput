use std::time::Duration;

use bevy::prelude::Camera3dBundle;
use bevy::prelude::*;
use schminput::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultSchminputPlugins);
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
    let set = cmds.spawn(ActionSetBundle::new("core", "core")).id();
    use schminput::keyboard::KeyboardBinding as KbB;
    cmds.spawn((
        ActionBundle::new("move", "Move", set),
        Vec2ActionValue::default(),
        KeyboardBindings::default()
            .add_binding(KbB::new(KeyCode::KeyW).y_axis().positive_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyS).y_axis().negative_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyA).x_axis().negative_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyD).x_axis().positive_axis_dir()),
        GamepadBindings::default()
            .add_binding(
                GamepadBinding::new(GamepadBindingSource::LeftStickX)
                    .x_axis()
                    .positive(),
            )
            .add_binding(
                GamepadBinding::new(GamepadBindingSource::LeftStickY)
                    .y_axis()
                    .positive(),
            ),
        MoveAction,
    ));
    cmds.spawn(ActionBundle::new("look", "Look", set)).insert((
        Vec2ActionValue::default(),
        MouseBindings::default().delta_motion(),
        GamepadBindings::default()
            .add_binding(
                GamepadBinding::new(GamepadBindingSource::RightStickX)
                    .x_axis()
                    .positive(),
            )
            .add_binding(
                GamepadBinding::new(GamepadBindingSource::RightStickY)
                    .y_axis()
                    .positive(),
            ),
        LookAction,
    ));
    cmds.spawn(ActionBundle::new("jump", "Jump", set)).insert((
        JumpAction,
        BoolActionValue::default(),
        GamepadBindings::default().add_binding(
            GamepadBinding::new(GamepadBindingSource::South),
        ),
        KeyboardBindings::default().add_binding(KbB::new(KeyCode::Space)),
    ));
    cmds.spawn(ActionBundle::new(
        "jump_haptic",
        "Jump Haptic Feedback",
        set,
    ))
    .insert((
        JumpHapticAction,
        GamepadHapticOutput::default(),
        GamepadHapticOutputBindings::default().weak(),
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
        info!("move: {}", action.any);
    }
    for action in look_action.into_iter() {
        info!("look: {}", action.any);
    }
    for action in jump_action.into_iter() {
        info!("jump: {}", action.any);
        if action.any {
            //panics if action doesn't exist
            jump_haptic_action
                .single_mut()
                .add(Duration::from_millis(50), 1.0);
        }
    }
}
