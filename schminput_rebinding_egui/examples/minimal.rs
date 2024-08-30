use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use schminput::prelude::*;
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    // app.add_plugins(DefaultSchminputRebindingPlugins);
    app.add_plugins(EguiPlugin);

    app.add_systems(Startup, setup);
    app.add_systems(Update, draw_ui);
    app.run();
}
fn setup(mut cmds: Commands) {
    let set = cmds.spawn(ActionSetBundle::new("core", "Core")).id();
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
    ));
    cmds.spawn(ActionBundle::new("look", "Look", set)).insert((
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
    ));
    cmds.spawn(ActionBundle::new("jump", "Jump", set)).insert((
        BoolActionValue::default(),
        GamepadBindings::default().add_binding(
            GamepadBindingDevice::Any,
            GamepadBinding::button(GamepadButtonType::South),
        ),
        KeyboardBindings::default().add_binding(KbB::new(KeyCode::Space)),
    ));
    cmds.spawn(ActionBundle::new(
        "jump_haptic",
        "Jump Haptic Feedback",
        set,
    ))
    .insert((
        GamepadHapticOutput::default(),
        GamepadHapticOutputBindings::default().weak(GamepadBindingDevice::Any),
    ));
    // should not be needed? but something hates me, official bevy_egui simple example is also broken
    cmds.spawn(Camera3dBundle { ..default() });
}

fn draw_ui(
    mut ctxs: EguiContexts,
    mut action_query: Query<(
        Entity,
        Option<&mut KeyboardBindings>,
        Option<&mut MouseBindings>,
        Option<&mut GamepadBindings>,
        &ActionName,
        &LocalizedActionName,
        Has<BoolActionValue>,
    )>,
    set_query: Query<(&LocalizedActionSetName, &ActionsInSet)>,
) {
    egui::Window::new("Schminput Rebinding Ui").show(ctxs.ctx_mut(), |ui| {
        // ui.label("hello wowld");
        schminput_rebinding_egui::egui::draw_rebinding_ui(ui, &mut action_query, &set_query);
    });
}
