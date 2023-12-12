use std::f32::consts::TAU;

use bevy::{prelude::*, diagnostic::FrameTimeDiagnosticsPlugin};
use bevy_oxr::xr_input::trackers::{OpenXRTrackingRoot, OpenXRLeftController, OpenXRController, OpenXRTracker, OpenXRRightController};
use schminput_schmanager::{
    keyboard_binding_provider::{
        KeyBinding, KeyboardBinding, KeyboardBindingProvider, KeyboardBindings,
    },
    new_action,
    oxr_binding_provider::{OXRBinding, OXRSetupBindings},
    SchminputApp, SchminputPlugin,
};

pub struct XrActionSet;
impl XrActionSet {
    pub fn name() -> String {
        "Xr Action Set".into()
    }
    pub fn key() -> &'static str {
        "xr_action_set"
    }
}

new_action!(
    PlayerMove,
    Vec2,
    "player_move",
    "Move".into(),
    XrActionSet::key(),
    XrActionSet::name()
);
new_action!(
    PlayerTurn,
    f32,
    "player_turn",
    "Turn".into(),
    XrActionSet::key(),
    XrActionSet::name()
);

fn main() {
    let mut app = App::new();
    app.register_action::<PlayerMove>();
    app.register_action::<PlayerTurn>();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(KeyboardBindingProvider);
    app.add_plugins(SchminputPlugin);
    app.add_plugins(FrameTimeDiagnosticsPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(Startup, setup_env);
    app.add_systems(Startup, spawn_controllers_example);
    app.add_systems(Update, run);

    app.run();
}


fn spawn_controllers_example(mut commands: Commands) {
    //left hand
    commands.spawn((
        OpenXRLeftController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
    ));
    //right hand
    commands.spawn((
        OpenXRRightController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
    ));
}

fn run(
    actions: Query<(&PlayerMove, &PlayerTurn)>,
    mut transform: Query<&mut Transform, With<OpenXRTrackingRoot>>,
    time: Res<Time>,
) {
    let (player_move, player_turn) = actions.get_single().unwrap();
    let mut transform = transform.get_single_mut().unwrap();
    transform.rotate_y(player_turn.data.clamp(-1.0, 1.0) * TAU * 0.5 * time.delta_seconds());
    let p_move = player_move.data.normalize_or_zero();
    let mut forward = (transform.forward() * p_move.x) + (transform.right() * p_move.y);
    forward.y = 0.0;
    let forward = forward.normalize_or_zero();
    transform.translation += forward * time.delta_seconds() * 5.0;
}

fn setup_env(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
}

fn setup(
    mut keyboard: ResMut<KeyboardBindings>,
    mut oxr: ResMut<OXRSetupBindings>,
    mut commands: Commands,
) {
    let player_move = PlayerMove::default();
    let player_turn = PlayerTurn::default();
    keyboard.add_binding(
        &player_move,
        KeyboardBinding::Dpad {
            up: KeyBinding::Held(KeyCode::W),
            down: KeyBinding::Held(KeyCode::S),
            left: KeyBinding::Held(KeyCode::A),
            right: KeyBinding::Held(KeyCode::D),
        },
    );
    keyboard.add_binding(
        &player_turn,
        KeyboardBinding::Number {
            positive: KeyBinding::Held(KeyCode::E),
            negative: KeyBinding::Held(KeyCode::Q),
        },
    );
    oxr.add_binding(
        &player_move,
        OXRBinding {
            device: "/interaction_profiles/oculus/touch_controller",
            binding: "/user/hand/left/input/thumbstick",
        },
    );
    oxr.add_binding(
        &player_turn,
        OXRBinding {
            device: "/interaction_profiles/oculus/touch_controller",
            binding: "/user/hand/right/input/thumbstick/y",
        },
    );
    commands.spawn((player_turn, player_move));
}
