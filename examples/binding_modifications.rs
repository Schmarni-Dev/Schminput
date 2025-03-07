use bevy::prelude::*;
use schminput::{
    binding_modification::{BindingModifiactions, PremultiplyDeltaTimeSecondsModification},
    prelude::*,
};
fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultSchminputPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, print_output)
        .run()
}

fn print_output(actions: Res<Actions>, query: Query<&Vec2ActionValue>) {
    let a1 = query.get(actions.action_1).unwrap();
    let a2 = query.get(actions.action_2).unwrap();
    info!("action 1: {}", a1.any);
    info!("action 2: {}", a2.any);
}

#[derive(Resource)]
struct Actions {
    action_1: Entity,
    action_2: Entity,
}

fn setup(mut cmds: Commands, mut paths: ResMut<SubactionPaths>) {
    let set = cmds.spawn(ActionSet::new("core", "Core")).id();
    let thumbstick_path = paths.get_or_create_path("/gamepad/*/thumbstick", &mut cmds);
    let modification_entity = cmds.spawn(PremultiplyDeltaTimeSecondsModification).id();
    let action_1 = cmds
        .spawn((
            Action::new("action_1", "Test Action 1", set),
            GamepadBindings::new()
                .add_binding(GamepadBinding::new(GamepadBindingSource::LeftStickX).x_axis())
                .add_binding(GamepadBinding::new(GamepadBindingSource::LeftStickY).y_axis()),
            BindingModifiactions::new()
                .with_path_modification(thumbstick_path, modification_entity),
            Vec2ActionValue::new(),
        ))
        .id();
    let action_2 = cmds
        .spawn((
            Action::new("action_2", "Test Action 2", set),
            GamepadBindings::new()
                .add_binding(GamepadBinding::new(GamepadBindingSource::LeftStickX).x_axis())
                .add_binding(GamepadBinding::new(GamepadBindingSource::LeftStickY).y_axis()),
            Vec2ActionValue::new(),
        ))
        .id();
    cmds.insert_resource(Actions { action_1, action_2 })
}
