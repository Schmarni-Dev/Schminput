use std::{path::PathBuf, time::Duration};

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use schminput::prelude::*;
use schminput_rebinding::{
    DefaultSchminputRebindingPlugins, config::ConfigFilePath, egui_window::ShowEguiRebindingWindow,
};
fn main() {
    let mut app = App::new();
    app.insert_resource(ShowEguiRebindingWindow(true));
    app.insert_resource(ConfigFilePath::Path(PathBuf::from(
        "./config/egui_minimal.toml",
    )));
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultSchminputPlugins);
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(DefaultSchminputRebindingPlugins);

    app.add_systems(Startup, setup);
    app.add_systems(Update, read_actions);
    app.run();
}
#[derive(Resource)]
struct Actions {
    _set: Entity,
    move_action: Entity,
    look: Entity,
    jump: Entity,
    jump_haptic: Entity,
}
fn setup(mut cmds: Commands) {
    let set = cmds.spawn(ActionSet::new("core", "Core", 0)).id();
    use schminput::keyboard::KeyboardBinding as KbB;
    let move_action = cmds
        .spawn((
            Action::new("move", "Move", set),
            Vec2ActionValue::new(),
            KeyboardBindings::new()
                .bind(KbB::new(KeyCode::KeyW).y_axis().positive_axis_dir())
                .bind(KbB::new(KeyCode::KeyS).y_axis().negative_axis_dir())
                .bind(KbB::new(KeyCode::KeyA).x_axis().negative_axis_dir())
                .bind(KbB::new(KeyCode::KeyD).x_axis().positive_axis_dir()),
            GamepadBindings::new()
                .bind(
                    GamepadBinding::new(GamepadBindingSource::LeftStickX)
                        .x_axis()
                        .positive(),
                )
                .bind(
                    GamepadBinding::new(GamepadBindingSource::LeftStickY)
                        .y_axis()
                        .positive(),
                ),
        ))
        .id();
    let look = cmds
        .spawn((
            Action::new("look", "Look", set),
            Vec2ActionValue::new(),
            MouseBindings::new().delta_motion(),
            GamepadBindings::new()
                .bind(
                    GamepadBinding::new(GamepadBindingSource::RightStickX)
                        .x_axis()
                        .positive(),
                )
                .bind(
                    GamepadBinding::new(GamepadBindingSource::RightStickY)
                        .y_axis()
                        .positive(),
                ),
        ))
        .id();
    let jump = cmds
        .spawn((
            Action::new("jump", "Jump", set),
            BoolActionValue::new(),
            GamepadBindings::new()
                .bind(GamepadBinding::new(GamepadBindingSource::South))
                .bind(GamepadBinding::new(GamepadBindingSource::OtherButton(128))),
            KeyboardBindings::new().bind(KbB::new(KeyCode::Space).just_pressed()),
            MouseBindings::new().bind(MouseButtonBinding::new(MouseButton::Left)),
        ))
        .id();
    let jump_haptic = cmds
        .spawn((
            Action::new("jump_haptic", "Jump Haptic Feedback", set),
            GamepadHapticOutput::new(),
            GamepadHapticOutputBindings::new().weak(),
        ))
        .id();
    cmds.insert_resource(Actions {
        _set: set,
        move_action,
        look,
        jump,
        jump_haptic,
    });
    cmds.spawn(Camera3d::default());
}

fn read_actions(
    actions: Res<Actions>,
    vec2_action: Query<&Vec2ActionValue>,
    bool_action: Query<&BoolActionValue>,
    mut haptic_action: Query<&mut GamepadHapticOutput>,
) {
    info!(
        "move: {}",
        vec2_action.get(actions.move_action).unwrap().any
    );
    info!("look: {}", vec2_action.get(actions.look).unwrap().any);

    let jumping = bool_action.get(actions.jump).unwrap().any;
    info!("jump: {}", jumping);
    if jumping {
        // and maybe get_single_mut here
        haptic_action
            .get_mut(actions.jump_haptic)
            .unwrap()
            .add(Duration::from_millis(50), 10.0);
    }
}
