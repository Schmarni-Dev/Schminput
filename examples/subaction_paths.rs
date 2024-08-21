use bevy::prelude::*;
use schminput::{
    keyboard::KeyboardBindings,
    mouse::MouseBindings,
    subaction_paths::{RequestedSubactionPaths, SubactionPaths},
    ActionHeaderBuilder, ActionSetHeaderBuilder, BoolActionValue, ButtonInputBeheavior,
    DefaultSchminputPlugins, InputAxis, InputAxisDirection,
};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultSchminputPlugins)
        .add_systems(Startup, setup_actions)
        .add_systems(Update, print_action)
        .run()
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
}

fn setup_actions(mut cmds: Commands, mut paths: ResMut<SubactionPaths>) {
    let set = ActionSetHeaderBuilder::new("core")
        .with_name("Core")
        .build(&mut cmds)
        .id();
    let mut sub_paths = RequestedSubactionPaths::default();
    sub_paths.push(paths.get_or_create_path("/mouse/button", &mut cmds));
    sub_paths.push(paths.get_or_create_path("/keyboard", &mut cmds));
    let action = ActionHeaderBuilder::new("action")
        .with_name("Action")
        .with_set(set)
        .build(&mut cmds)
        .insert(BoolActionValue::default())
        .insert(
            KeyboardBindings::default()
                .add_binding(schminput::keyboard::KeyboardBinding::new(KeyCode::Space)),
        )
        .insert(
            MouseBindings::default().add_binding(schminput::mouse::MouseButtonBinding {
                axis: InputAxis::X,
                axis_dir: InputAxisDirection::Positive,
                button: MouseButton::Left,
                premultipy_delta_time: false,
                behavior: ButtonInputBeheavior::Pressed,
            }),
        )
        .insert(sub_paths)
        .id();
    cmds.insert_resource(Action(action));
}
