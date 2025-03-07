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
    // panic!("STOP");
    let b = query.get(action.0).unwrap();
    info!("default: {}", b.any);

    info!(
        "keyboard: {}",
        b.get_with_path(&paths.get("/keyboard").unwrap())
            .copied()
            .unwrap_or_default()
    );
    info!(
        "mouse: {}",
        b.get_with_path(&paths.get("/mouse/button").unwrap())
            .copied()
            .unwrap_or_default()
    );
    info!(
        "gamepad: {}",
        b.get_with_path(&paths.get("/gamepad/*").unwrap())
            .copied()
            .unwrap_or_default()
    );
    info!(
        "dpad: {}",
        b.get_with_path(&paths.get("/gamepad/*/dpad").unwrap())
            .copied()
            .unwrap_or_default()
    );
    info!(
        "left trigger: {}",
        b.get_with_path(&paths.get("/gamepad/*/trigger/left").unwrap())
            .copied()
            .unwrap_or_default()
    );
    info!(
        "trigger: {}",
        b.get_with_path(&paths.get("/gamepad/*/trigger").unwrap())
            .copied()
            .unwrap_or_default()
    );
}

fn setup_actions(mut cmds: Commands, mut paths: ResMut<SubactionPaths>) {
    let set = cmds.spawn(ActionSet::new("core", "Core")).id();
    let mut sub_paths = RequestedSubactionPaths::new();
    sub_paths.push_path("/mouse/button", &mut paths, &mut cmds);
    sub_paths.push_path("/keyboard", &mut paths, &mut cmds);
    sub_paths.push_path("/gamepad/*", &mut paths, &mut cmds);
    sub_paths.push_path("/gamepad/*/dpad", &mut paths, &mut cmds);
    sub_paths.push_path("/gamepad/*/trigger/left", &mut paths, &mut cmds);
    sub_paths.push_path("/gamepad/*/trigger", &mut paths, &mut cmds);
    let action = cmds
        .spawn((
            Action::new("action", "Action", set),
            BoolActionValue::new(),
            KeyboardBindings::new().add_binding(KeyboardBinding::new(KeyCode::Space)),
            MouseBindings::new().add_binding(MouseButtonBinding::new(MouseButton::Left)),
            GamepadBindings::new()
                .add_binding(GamepadBinding::new(GamepadBindingSource::DPadDown))
                .add_binding(GamepadBinding::new(GamepadBindingSource::South))
                .add_binding(GamepadBinding::new(GamepadBindingSource::LeftTrigger))
                .add_binding(GamepadBinding::new(GamepadBindingSource::RightTrigger)),
            sub_paths,
        ))
        .id();
    cmds.insert_resource(ActionHandle(action));
}
