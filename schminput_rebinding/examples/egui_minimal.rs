use std::path::PathBuf;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use schminput::prelude::*;
use schminput_rebinding::{
    config::ConfigFilePath, egui_window::ShowEguiRebindingWindow, DefaultSchminputRebindingPlugins,
};
fn main() {
    let mut app = App::new();
    app.insert_resource(ShowEguiRebindingWindow(true));
    app.insert_resource(ConfigFilePath::Path(PathBuf::from(
        "./config/egui_minimal.toml",
    )));
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultSchminputPlugins);
    app.add_plugins(EguiPlugin);
    app.add_plugins(DefaultSchminputRebindingPlugins);

    app.add_systems(Startup, setup);
    app.run();
}
fn setup(mut cmds: Commands) {
    let set = cmds.spawn(ActionSet::new("core", "Core")).id();
    use schminput::keyboard::KeyboardBinding as KbB;
    cmds.spawn((
        Action::new("move", "Move", set),
        Vec2ActionValue::new(),
        KeyboardBindings::new()
            .add_binding(KbB::new(KeyCode::KeyW).y_axis().positive_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyS).y_axis().negative_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyA).x_axis().negative_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyD).x_axis().positive_axis_dir()),
        GamepadBindings::new()
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
    ));
    cmds.spawn((
        Action::new("look", "Look", set),
        Vec2ActionValue::new(),
        MouseBindings::new().delta_motion(),
        GamepadBindings::new()
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
    ));
    cmds.spawn((
        Action::new("jump", "Jump", set),
        BoolActionValue::new(),
        GamepadBindings::new()
            .add_binding(GamepadBinding::new(GamepadBindingSource::South))
            .add_binding(GamepadBinding::new(GamepadBindingSource::OtherButton(128))),
        KeyboardBindings::new().add_binding(KbB::new(KeyCode::Space).just_pressed()),
        MouseBindings::new().add_binding(MouseButtonBinding::new(MouseButton::Left)),
    ));
    cmds.spawn((
        Action::new("jump_haptic", "Jump Haptic Feedback", set),
        GamepadHapticOutput::new(),
        GamepadHapticOutputBindings::new().weak(),
    ));
    cmds.spawn(Camera3d::default());
}
