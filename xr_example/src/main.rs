use std::f32::consts::TAU;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_oxr::{
    xr_init::{XrEnableStatus, XrPostSetup},
    xr_input::trackers::{
        OpenXRController, OpenXRHMD, OpenXRLeftController, OpenXRRightController, OpenXRTracker,
        OpenXRTrackingRoot,
    },
    DefaultXrPlugins,
};
use bevy_schminput::{
    mouse::{motion::MouseMotionBindingProvider, MouseBindings},
    mouse_action,
    prelude::*,
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

basic_action!(
    PlayerMove,
    Vec2,
    "player_move",
    "Move".into(),
    XrActionSet::key(),
    XrActionSet::name()
);
mouse_action!(
    PlayerLook,
    "player_look",
    "Look".into(),
    XrActionSet::key(),
    XrActionSet::name()
);

fn main() {
    let mut app = App::new();
    app.register_action::<PlayerMove>();
    info!("Test");
    app.register_action::<PlayerLook>();
    info!("HMMMMM");
    app.add_plugins(DefaultXrPlugins::default());
    app.add_plugins(KeyboardBindingProvider);
    app.add_plugins(OXRBindingProvider);
    app.add_plugins(MouseBindingProvider);
    app.add_plugins(MouseMotionBindingProvider);
    app.add_plugins(SchminputPlugin);
    app.add_plugins(FrameTimeDiagnosticsPlugin);
    app.add_systems(XrPostSetup, xr_add_forward_ref);
    app.add_systems(Startup, setup);
    app.add_systems(Startup, setup_env);
    app.add_systems(Startup, spawn_controllers_example);
    app.add_systems(
        Update,
        (apply_turning, apply_forward, apply_movement).chain(),
    );

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
fn apply_turning(mut actions: Query<(&PlayerLook, &mut Transform)>, time: Res<Time>) {
    for (player_turn, mut transform) in actions.iter_mut() {
        transform.rotate_y(-player_turn.data.x * TAU * 0.5 * time.delta_seconds());
        transform.rotate_local_x(-player_turn.data.y * TAU * 0.5 * time.delta_seconds());
    }
}

fn apply_forward(mut forward_ref: Query<(&Transform, &mut ForwardRef)>) {
    for (transform, mut forward) in forward_ref.iter_mut() {
        forward.forward = transform.forward();
        forward.right = transform.right();
    }
}

fn apply_movement(
    mut actions: Query<(&PlayerMove, &mut Transform)>,
    forward_ref: Query<&ForwardRef>,
    time: Res<Time>,
) {
    let forward_ref = match forward_ref.get_single() {
        Ok(v) => v,
        Err(_) => return,
    };
    for (player_move, mut transform) in actions.iter_mut() {
        let p_move = player_move.data.normalize_or_zero();
        let mut forward = (forward_ref.forward * p_move.y) + (forward_ref.right * p_move.x);
        forward.y = 0.0;
        let forward = forward.normalize_or_zero();
        transform.translation += forward * time.delta_seconds() * 3.0;
    }
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
    mut mouse: ResMut<MouseBindings>,
    xr_enabled: Option<Res<XrEnableStatus>>,
) {
    let player_move = PlayerMove::default();
    let mut player_turn = PlayerLook::default();
    player_turn.mouse_sens_x = 0.1;
    player_turn.mouse_sens_y = 0.1;
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
            positive: KeyBinding::Held(KeyCode::Q),
            negative: KeyBinding::Held(KeyCode::E),
        },
    );
    mouse.add_motion_binding(&player_turn);
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

    if xr_enabled.is_none() || xr_enabled.is_some_and(|v| *v == XrEnableStatus::Disabled) {
        info!("Non Xr Mode");
        let mut t = Transform::from_xyz(0.0, 1.8, 0.0);
        t.rotate_x(TAU * -0.05);
        let cam = commands
            .spawn((
                Camera3dBundle::default(),
                player_turn,
                ForwardRef::default(),
            ))
            .insert(t)
            .id();

        commands
            .spawn((SpatialBundle::default(), OpenXRTrackingRoot, player_move))
            .push_children(&[cam]);
    } else {
        info!("Xr Mode");
        commands.spawn((
            SpatialBundle::default(),
            OpenXRTrackingRoot,
            player_turn,
            player_move,
        ));
    }
}

fn xr_add_forward_ref(mut commands: Commands, hmd: Query<Entity, With<OpenXRTrackingRoot>>) {
    commands
        .entity(hmd.get_single().unwrap())
        .insert(ForwardRef::default());
}

#[derive(Component, Default)]
struct ForwardRef {
    forward: Vec3,
    right: Vec3,
}
