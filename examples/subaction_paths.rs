use bevy::{input::gamepad::{GamepadAxisChangedEvent, GamepadButtonChangedEvent}, prelude::*};
use schminput::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultSchminputPlugins)
        .add_systems(Startup, setup_actions)
        // .add_systems(Update, print_action)
        .add_systems(Update, test)
        .run()
}

fn test(
    mut event: EventReader<GamepadAxisChangedEvent>,
    mut event2: EventReader<GamepadButtonChangedEvent>,
) {
    for e in event.read() {
        info!("axis: {:#?}", e);
    }
    for e in event2.read() {
        info!("button: {:#?}", e);
    }
}

#[derive(Resource)]
struct Action(Entity);

fn print_action(action: Res<Action>, query: Query<&BoolActionValue>, paths: Res<SubactionPaths>) {
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
        "dpad: {}",
        b.get_with_path(&paths.get("/gamepad/*/dpad").unwrap())
            .copied()
            .unwrap_or_default()
    );
    info!(
        "left trigger: {}",
        b.get_with_path(&paths.get("/gamepad/*/secondary_trigger/left").unwrap())
            .copied()
            .unwrap_or_default()
    );
    info!(
        "trigger: {}",
        b.get_with_path(&paths.get("/gamepad/*/secondary_trigger").unwrap())
            .copied()
            .unwrap_or_default()
    );
}

fn setup_actions(mut cmds: Commands, mut paths: ResMut<SubactionPaths>) {
    let set = cmds.spawn(ActionSetBundle::new("core", "Core")).id();
    let mut sub_paths = RequestedSubactionPaths::default();
    sub_paths.push(paths.get_or_create_path("/mouse/button", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/keyboard", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/gamepad/*", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/gamepad/*/dpad", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/gamepad/*/secondary_trigger/left", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/gamepad/*/secondary_trigger", &mut cmds));
    let action = cmds
        .spawn(ActionBundle::new("action", "Action", set))
        .insert(BoolActionValue::default())
        .insert(KeyboardBindings::default().add_binding(KeyboardBinding::new(KeyCode::Space)))
        .insert(MouseBindings::default().add_binding(MouseButtonBinding::new(MouseButton::Left)))
        .insert(
            GamepadBindings::default()
                .add_binding(GamepadBinding::button(GamepadButtonType::DPadDown))
                .add_binding(GamepadBinding::button(GamepadButtonType::South))
                .add_binding(GamepadBinding::button(GamepadButtonType::LeftTrigger2))
                .add_binding(GamepadBinding::button(GamepadButtonType::RightTrigger2)), // .add_binding(GamepadBinding::axis(GamepadAxisType::LeftZ))
                                                                                        // .add_binding(GamepadBinding::axis(GamepadAxisType::RightZ)),
        )
        .insert(sub_paths)
        .id();
    cmds.insert_resource(Action(action));
}
