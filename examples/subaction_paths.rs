use bevy::prelude::*;
use schminput::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultSchminputPlugins)
        .add_systems(Startup, setup_actions)
        .add_systems(Update, print_action)
        .run()
}

#[derive(Resource)]
struct ActionHandle(Entity);

fn print_action(
    action: Res<ActionHandle>,
    query: Query<&BoolActionValue>,
    paths: Res<SubactionPaths>,
) {
    let b = query.get(action.0).unwrap();
    info!("default: {}", b.any);

    info!("keyboard: {}", b.get_or_default(paths.get("/keyboard")));
    info!("mouse: {}", b.get_or_default(paths.get("/mouse/button")));
    info!("gamepad: {}", b.get_or_default(paths.get("/gamepad/*")));
    info!("dpad: {}", b.get_or_default(paths.get("/gamepad/*/dpad")));
    info!(
        "left trigger: {}",
        b.get_or_default(paths.get("/gamepad/*/trigger/left"))
    );
    info!(
        "trigger: {}",
        b.get_or_default(paths.get("/gamepad/*/trigger"))
    );
}

fn setup_actions(mut cmds: Commands, mut paths: ResMut<SubactionPaths>) {
    let set = cmds.spawn(ActionSet::new("core", "Core", 0)).id();
    let sub_paths = RequestedSubactionPaths::new()
        .mutate(&mut paths, cmds.reborrow())
        .push("/mouse/button")
        .push("/keyboard")
        .push("/gamepad/*")
        .push("/gamepad/*/dpad")
        .push("/gamepad/*/trigger/left")
        .push("/gamepad/*/trigger")
        .end();
    let action = cmds
        .spawn((
            Action::new("action", "Action", set),
            BoolActionValue::new(),
            KeyboardBindings::new().bind(KeyboardBinding::new(KeyCode::Space)),
            MouseBindings::new().bind(MouseButtonBinding::new(MouseButton::Left)),
            GamepadBindings::new()
                .bind(GamepadBinding::new(GamepadBindingSource::DPadDown))
                .bind(GamepadBinding::new(GamepadBindingSource::South))
                .bind(GamepadBinding::new(GamepadBindingSource::LeftTrigger))
                .bind(GamepadBinding::new(GamepadBindingSource::RightTrigger)),
            sub_paths,
        ))
        .id();
    cmds.insert_resource(ActionHandle(action));
}
