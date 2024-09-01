use bevy::prelude::*;
use schminput::prelude::*;
fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultSchminputPlugins)
        .add_systems(Startup, setup_actions)
        .add_systems(Update, print_action)
        .add_systems(Update, update_action_set)
        .run()
}
#[derive(Resource)]
struct Actions {
    #[allow(dead_code)]
    core_set: Entity,
    movement_set: Entity,
    toggle_movement: Entity,
    move_action: Entity,
}

fn update_action_set(
    mut state: Local<bool>,
    query: Query<&BoolActionValue>,
    actions: Res<Actions>,
    mut cmds: Commands,
) {
    let b = query.get(actions.toggle_movement).unwrap();
    if b.any {
        *state = !*state;
        cmds.entity(actions.movement_set)
            .insert(ActionSetEnabled(!*state));
    }
}

fn print_action(action: Res<Actions>, query: Query<&Vec2ActionValue>) {
    let b = query.get(action.move_action).unwrap();
    info!("default: {}", b.any);
}

fn setup_actions(mut cmds: Commands) {
    let core = cmds.spawn(ActionSetBundle::new("core", "Core")).id();
    let player_set = cmds.spawn(ActionSetBundle::new("move", "Movement")).id();
    let toggle = cmds
        .spawn(ActionBundle::new(
            "toggle_movement",
            "Toggle Movement",
            core,
        ))
        .insert(BoolActionValue::default())
        .insert(
            KeyboardBindings::default()
                .add_binding(KeyboardBinding::new(KeyCode::Tab).just_pressed()),
        )
        .id();
    let move_action = cmds
        .spawn(ActionBundle::new("move", "Move", player_set))
        .insert(Vec2ActionValue::default())
        .insert(
            KeyboardBindings::default()
                .add_binding(KeyboardBinding::new(KeyCode::KeyW).y_axis())
                .add_binding(
                    KeyboardBinding::new(KeyCode::KeyS)
                        .y_axis()
                        .negative_axis_dir(),
                )
                .add_binding(KeyboardBinding::new(KeyCode::KeyD).x_axis())
                .add_binding(
                    KeyboardBinding::new(KeyCode::KeyA)
                        .x_axis()
                        .negative_axis_dir(),
                ),
        )
        .id();
    cmds.insert_resource(Actions {
        core_set: core,
        movement_set: player_set,
        toggle_movement: toggle,
        move_action,
    });
}
