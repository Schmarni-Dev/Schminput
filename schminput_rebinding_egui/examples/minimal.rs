use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use schminput::prelude::*;
use schminput_rebinding_egui::{
    default_bindings::{RebindingDefaultBindingsPlugin, ResetToDefautlBindings},
    egui::ActionStateQuery,
    runtime_rebinding::{
        RequestGamepadRebinding, RequestKeyboardRebinding, RequestMouseRebinding, RuntimeRebindingPlugin, WaitingForInput
    },
};
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultSchminputPlugins);
    app.add_plugins(EguiPlugin);
    app.add_plugins(RuntimeRebindingPlugin);
    app.add_plugins(RebindingDefaultBindingsPlugin);

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
        ).add_binding(
            GamepadBindingDevice::Any,
            GamepadBinding::button(GamepadButtonType::Other(128)),
        ),
        KeyboardBindings::default().add_binding(KbB::new(KeyCode::Space).just_pressed()),
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
        &LocalizedActionName,
        Has<BoolActionValue>,
    )>,
    set_query: Query<(&LocalizedActionSetName, &ActionsInSet)>,
    waiting: Res<WaitingForInput>,
    request_keyboard: EventWriter<RequestKeyboardRebinding>,
    action_type_query: ActionStateQuery,
    reset_bindings: EventWriter<ResetToDefautlBindings>,
    mouse_rebind: EventWriter<RequestMouseRebinding>,
     gamepad_rebind: EventWriter<RequestGamepadRebinding>,
) {
    egui::Window::new("Schminput Rebinding Ui").show(ctxs.ctx_mut(), |ui| {
        // ui.label("hello wowld");
        schminput_rebinding_egui::egui::draw_rebinding_ui(
            ui,
            &mut action_query,
            &action_type_query,
            &set_query,
            &waiting,
            request_keyboard,
            mouse_rebind,
            gamepad_rebind,
            reset_bindings,
        );
    });
}
