use bevy::{
    input::gamepad::{GamepadAxisChangedEvent, GamepadButtonChangedEvent},
    prelude::*,
};
use schminput::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultSchminputPlugins)
        .add_systems(Startup, setup_actions)
        .add_systems(Update, print_action)
        // .add_systems(Update, test)
        .run()
}

#[derive(Resource)]
struct Action(Entity);

fn print_action(action: Res<Action>, query: Query<&BoolActionValue>, paths: Res<SubactionPaths>) {
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
    let set = cmds.spawn(ActionSetBundle::new("core", "Core")).id();
    let mut sub_paths = RequestedSubactionPaths::default();
    sub_paths.push(paths.get_or_create_path("/mouse/button", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/keyboard", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/gamepad/*", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/gamepad/*/dpad", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/gamepad/*/trigger/left", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/gamepad/*/trigger", &mut cmds));
    let action = cmds
        .spawn(ActionBundle::new("action", "Action", set))
        .insert(BoolActionValue::default())
        .insert(KeyboardBindings::default().add_binding(KeyboardBinding::new(KeyCode::Space)))
        .insert(MouseBindings::default().add_binding(MouseButtonBinding::new(MouseButton::Left)))
        .insert(
            GamepadBindings::default()
                .add_binding(GamepadBinding::new(GamepadBindingSource::DPadDown))
                .add_binding(GamepadBinding::new(GamepadBindingSource::South))
                .add_binding(GamepadBinding::new(GamepadBindingSource::LeftTrigger))
                .add_binding(GamepadBinding::new(GamepadBindingSource::RightTrigger)),
        )
        .insert(sub_paths)
        .id();
    cmds.insert_resource(Action(action));
}
