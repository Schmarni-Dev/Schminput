use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_openxr::init::OxrTrackingRoot;
use bevy_schminput::{
    gamepad::{GamepadBinding, GamepadBindingDevice, GamepadBindings},
    prelude::*,
    openxr::{OxrActionBlueprint, PoseActionValue, SetPoseOfEntity, OCULUS_TOUCH_PROFILE},
    ActionHeaderBuilder, ActionSetHeaderBuilder,
};
#[derive(Component, Clone, Copy)]
struct HandLeft;
#[derive(Component, Clone, Copy)]
struct HandRightAction;

#[derive(Component, Clone, Copy)]
struct MoveAction;
#[derive(Component, Clone, Copy)]
struct LookAction;
#[derive(Component, Clone, Copy)]
struct JumpAction;

fn main() {
    let mut app = App::new();
    app.add_plugins(bevy_openxr::add_xr_plugins(DefaultPlugins));
    app.add_plugins(FrameTimeDiagnosticsPlugin);
    app.add_plugins(bevy_schminput::DefaultSchmugins);
    app.add_plugins(bevy_schminput::openxr::OxrInputPlugin);
    // app.add_systems(XrPostSetup, xr_add_forward_ref);
    app.add_systems(Startup, setup);
    app.add_systems(Startup, setup_env);
    app.add_systems(Update, run);
    // app.add_systems(Startup, spawn_controllers_example);
    // app.add_systems(
    //     Update,
    //     (apply_turning, apply_forward, apply_movement).chain(),
    // );

    app.run();
}
fn setup(mut cmds: Commands, root: Query<Entity, With<OxrTrackingRoot>>) {
    let player_set = ActionSetHeaderBuilder::new("player")
        .with_name("Player")
        .build(&mut cmds)
        .id();
    let pose_set = ActionSetHeaderBuilder::new("pose")
        .with_name("Poses")
        .build(&mut cmds)
        .id();
    ActionHeaderBuilder::new("move")
        .with_name("Move")
        .with_set(player_set)
        .build(&mut cmds)
        .insert((
            MoveAction,
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/left/input/thumbstick")
                .end(),
            Vec2ActionValue::default(),
        ));
    ActionHeaderBuilder::new("look")
        .with_name("Look")
        .with_set(player_set)
        .build(&mut cmds)
        .insert((
            LookAction,
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/thumbstick/x")
                .end(),
            F32ActionValue::default(),
        ));
    ActionHeaderBuilder::new("jump")
        .with_name("Jump")
        .with_set(player_set)
        .build(&mut cmds)
        .insert((
            JumpAction,
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/a/click")
                .end(),
            KeyboardBindings::default().add_binding(KeyboardBinding::new(KeyCode::Space)),
            GamepadBindings::default().add_binding(
                GamepadBindingDevice::Any,
                GamepadBinding::button(GamepadButtonType::South).button_just_pressed(),
            ),
            BoolActionValue::default(),
        ));
    let left_hand = cmds.spawn((SpatialBundle::default(), HandLeft)).id();
    ActionHeaderBuilder::new("hand_left_pose")
        .with_name("Left Hand Pose")
        .with_set(pose_set)
        .build(&mut cmds)
        .insert((
            MoveAction,
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/left/input/grip/pose")
                .end(),
            SetPoseOfEntity(left_hand),
        ));
    ActionHeaderBuilder::new("hand_right_pose")
        .with_name("Right Hand Pose")
        .with_set(pose_set)
        .build(&mut cmds)
        .insert((
            MoveAction,
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/grip/pose")
                .end(),
            PoseActionValue(Transform::IDENTITY),
            HandRightAction,
        ));
    // cmds.entity(root.single()).add_child(left_hand);
}

// fn apply_forward(mut forward_ref: Query<(&Transform, &mut ForwardRef)>) {
//     for (transform, mut forward) in forward_ref.iter_mut() {
//         forward.forward = *transform.forward();
//         forward.right = *transform.right();
//     }
// }
//
// fn apply_movement(
//     mut actions: Query<(&PlayerMove, &mut Transform)>,
//     forward_ref: Query<&ForwardRef>,
//     time: Res<Time>,
// ) {
//     let forward_ref = match forward_ref.get_single() {
//         Ok(v) => v,
//         Err(_) => return,
//     };
//     for (player_move, mut transform) in actions.iter_mut() {
//         let p_move = player_move.data.normalize_or_zero();
//         let mut forward = (forward_ref.forward * p_move.y) + (forward_ref.right * p_move.x);
//         forward.y = 0.0;
//         let forward = forward.normalize_or_zero();
//         transform.translation += forward * time.delta_seconds() * 3.0;
//     }
// }

fn run(
    move_action: Query<&Vec2ActionValue, With<MoveAction>>,
    look_action: Query<&F32ActionValue, With<LookAction>>,
    jump_action: Query<&BoolActionValue, With<JumpAction>>,
    left_hand: Query<&Transform, With<HandLeft>>,
    right_hand_action: Query<&PoseActionValue, With<HandRightAction>>,
    mut gizmos: Gizmos,
    root: Query<&GlobalTransform, With<OxrTrackingRoot>>,
) {
    info!("move: {}", **move_action.single());
    info!("look: {}", **look_action.single());
    info!("jump: {}", **jump_action.single());
    for hand in left_hand.into_iter() {
        gizmos.sphere(hand.translation, hand.rotation, 0.1, Color::ORANGE_RED);
    }
    let pose = right_hand_action.single();
    // let pose = root.single().compute_transform().mul_transform(**pose);
    gizmos.sphere(pose.translation, pose.rotation, 0.1, Color::LIME_GREEN);
}

fn setup_env(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::new(Vec3::Y).mesh().size(5.0, 5.0)),
        material: materials.add(StandardMaterial::from(Color::rgb(0.3, 0.5, 0.3))),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(0.1)))),
        material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.7, 0.6))),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
}

// fn setup(
//     mut keyboard: ResMut<KeyboardBindings>,
//     mut oxr: ResMut<OXRSetupBindings>,
//     mut commands: Commands,
//     mut mouse: ResMut<MouseBindings>,
//     xr_enabled: Option<Res<XrStatus>>,
// ) {
//     let player_move = PlayerMove::default();
//     let player_turn = PlayerLook {
//         mouse_sens_x: 0.1,
//         mouse_sens_y: 0.1,
//         ..Default::default()
//     };
//     keyboard.add_binding(
//         &player_move,
//         KeyboardBinding::Dpad {
//             up: KeyBinding::Held(KeyCode::KeyW),
//             down: KeyBinding::Held(KeyCode::KeyS),
//             left: KeyBinding::Held(KeyCode::KeyA),
//             right: KeyBinding::Held(KeyCode::KeyD),
//         },
//     );
//     keyboard.add_binding(
//         &player_turn,
//         KeyboardBinding::Number {
//             positive: KeyBinding::Held(KeyCode::KeyQ),
//             negative: KeyBinding::Held(KeyCode::KeyE),
//         },
//     );
//     mouse.add_motion_binding(&player_turn);
//     oxr.add_binding(
//         &player_move,
//         OXRBinding {
//             device: "/interaction_profiles/oculus/touch_controller",
//             binding: "/user/hand/left/input/thumbstick",
//         },
//     );
//     oxr.add_binding(
//         &player_turn,
//         OXRBinding {
//             device: "/interaction_profiles/oculus/touch_controller",
//             binding: "/user/hand/right/input/thumbstick/y",
//         },
//     );
//
//     if xr_enabled.is_none() || xr_enabled.is_some_and(|v| *v == XrStatus::Disabled) {
//         info!("Non Xr Mode");
//         let mut t = Transform::from_xyz(0.0, 1.8, 0.0);
//         t.rotate_x(TAU * -0.05);
//         let cam = commands
//             .spawn((
//                 Camera3dBundle::default(),
//                 player_turn,
//                 ForwardRef::default(),
//             ))
//             .insert(t)
//             .id();
//
//         commands
//             .spawn((SpatialBundle::default(), OpenXRTrackingRoot, player_move))
//             .push_children(&[cam]);
//     } else {
//         info!("Xr Mode");
//         commands.spawn((
//             SpatialBundle::default(),
//             OpenXRTrackingRoot,
//             player_turn,
//             player_move,
//         ));
//     }
// }
//
// fn xr_add_forward_ref(mut commands: Commands, hmd: Query<Entity, With<OpenXRTrackingRoot>>) {
//     commands
//         .entity(hmd.get_single().unwrap())
//         .insert(ForwardRef::default());
// }

#[derive(Component, Default)]
struct ForwardRef {
    forward: Vec3,
    right: Vec3,
}
