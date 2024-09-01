use bevy::prelude::*;
use schminput::{
    binding_modification::{
        BindingModifiaction, BindingModifiactions, PremultiplyDeltaTimeSecondsModification,
    },
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
    let set = cmds.spawn(ActionSetBundle::new("core", "Core")).id();
    let thumbstick_path = paths.get_or_create_path("/gamepad/*/thumbstick", &mut cmds);
    let mut modifications = BindingModifiactions::default();
    let modification_entity = cmds.spawn(PremultiplyDeltaTimeSecondsModification).id();
    modifications
        .per_path
        .push((thumbstick_path, BindingModifiaction(modification_entity)));
    let action_1 = cmds
        .spawn(ActionBundle::new("action_1", "Test Action 1", set))
        .insert(
            GamepadBindings::default()
                .add_binding(GamepadBinding::new(GamepadBindingSource::LeftStickX).x_axis())
                .add_binding(GamepadBinding::new(GamepadBindingSource::LeftStickY).y_axis()),
        )
        .insert(modifications)
        .insert(Vec2ActionValue::default())
        .id();
    let action_2 = cmds
        .spawn(ActionBundle::new("action_2", "Test Action 2", set))
        .insert(
            GamepadBindings::default()
                .add_binding(GamepadBinding::new(GamepadBindingSource::LeftStickX).x_axis())
                .add_binding(GamepadBinding::new(GamepadBindingSource::LeftStickY).y_axis()),
        )
        .insert(Vec2ActionValue::default())
        .id();
    cmds.insert_resource(Actions { action_1, action_2 })
}
