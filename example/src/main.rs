use bevy::prelude::*;
use bevy::prelude::Camera3dBundle;
use bevy_schminput::{
    keyboard::{KeyboardBinding, KeyboardBindings},
    mouse::MouseBindings,
    ActionHeaderBuilder, DefaultSchmugins, Vec2ActionValue,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultSchmugins);
    app.add_systems(Startup, setup);
    app.add_systems(Update, run);

    app.run();
}

#[derive(Component, Clone, Copy)]
struct MoveAction;
#[derive(Component, Clone, Copy)]
struct LookAction;

fn setup(mut cmds: Commands) {
    let mut move_e = ActionHeaderBuilder::new("move")
        .with_name("Move")
        .build(&mut cmds);
    use bevy_schminput::keyboard::KeyboardBinding as KbB;
    move_e.insert((
        Vec2ActionValue::default(),
        KeyboardBindings::default()
            .add_binding(KbB::new(KeyCode::KeyW).y_axis().positive_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyS).y_axis().negative_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyA).x_axis().negative_axis_dir())
            .add_binding(KbB::new(KeyCode::KeyD).x_axis().positive_axis_dir()),
        MoveAction,
    ));
    let mut look_e = ActionHeaderBuilder::new("look")
        .with_name("Look")
        .build(&mut cmds);
    look_e.insert((
        Vec2ActionValue::default(),
        MouseBindings::default().delta_motion(),
        LookAction,
    ));
    cmds.spawn(Camera3dBundle::default());
}

fn run(
    input: Query<&Vec2ActionValue, With<MoveAction>>,
    bools: Query<&Vec2ActionValue, With<LookAction>>,
) {
    for action in input.into_iter() {
        info!("move: {:?}", action);
    }
    for action in bools.into_iter() {
        info!("look: {:?}", action);
    }
}

// fn setup(
//     mut commands: Commands,
// ) {
//     let example = ExampleAction::default();
//     let bool = BoolAction::default();
//     keyboard.add_binding(
//         &bool,
//         KeyboardBinding::Simple(KeyBinding::JustPressed(KeyCode::KeyC)),
//     );
//     keyboard.add_binding(
//         &example,
//         KeyboardBinding::Dpad {
//             up: KeyBinding::Held(KeyCode::KeyW),
//             down: KeyBinding::Held(KeyCode::KeyS),
//             left: KeyBinding::Held(KeyCode::KeyA),
//             right: KeyBinding::Held(KeyCode::KeyD),
//         },
//     );
//     mouse.add_binding(&bool, MouseBinding::JustPressed(MouseButton::Left));
//     mouse.add_binding(&bool, MouseBinding::JustReleased(MouseButton::Left));
//     mouse.add_binding(&bool, MouseBinding::Held(MouseButton::Right));
//     commands.spawn((example, bool));
// }
