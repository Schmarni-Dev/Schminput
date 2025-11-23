use std::time::Duration;

use bevy::prelude::*;
use schminput::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultSchminputPlugins);
    app.add_systems(Startup, setup);
    app.add_systems(Update, read_actions);

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
    let set = cmds.spawn(ActionSet::new("core", "core", 0)).id();
    cmds.spawn((
        Action::new("move", "Move", set),
        Vec2ActionValue::new(),
        KeyboardBindings::new().add_dpad(
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyA,
            KeyCode::KeyD,
        ),
        GamepadBindings::new().add_stick(
            GamepadBindingSource::LeftStickX,
            GamepadBindingSource::LeftStickX,
        ),
        MoveAction,
    ));
    cmds.spawn((
        Action::new("look", "Look", set),
        Vec2ActionValue::new(),
        MouseBindings::new().delta_motion(),
        GamepadBindings::new().add_stick(
            GamepadBindingSource::RightStickX,
            GamepadBindingSource::RightStickY,
        ),
        LookAction,
    ));
    cmds.spawn((
        Action::new("jump", "Jump", set),
        GamepadBindings::new()
            .bind(GamepadBinding::new(GamepadBindingSource::South).button_just_pressed()),
        KeyboardBindings::new().bind(KeyboardBinding::new(KeyCode::Space).just_pressed()),
        BoolActionValue::new(),
        JumpAction,
    ));
    cmds.spawn((
        Action::new("jump_haptic", "Jump Haptic Feedback", set),
        GamepadHapticOutputBindings::new().weak(),
        GamepadHapticOutput::new(),
        JumpHapticAction,
    ));
    cmds.spawn(Camera3d::default());
}

fn read_actions(
    move_action: Query<&Vec2ActionValue, With<MoveAction>>,
    look_action: Query<&Vec2ActionValue, With<LookAction>>,
    jump_action: Query<&BoolActionValue, With<JumpAction>>,
    mut jump_haptic_action: Query<&mut GamepadHapticOutput, With<JumpHapticAction>>,
) {
    // you might want to use .get_single instead to handle a case where the action was destroyed
    // (which never happens in the crate itself)
    info!("move: {}", move_action.single().unwrap().any);
    info!("look: {}", look_action.single().unwrap().any);

    let jumping = jump_action.single().unwrap().any;
    info!("jump: {}", jumping);
    if jumping {
        // and maybe get_single_mut here
        jump_haptic_action
            .single_mut();
            //.add(Duration::from_millis(50), 1.0);
    }
}
