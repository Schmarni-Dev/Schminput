use bevy::{color::palettes::css, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use schminput::{
    gamepad::{GamepadBinding, GamepadBindings},
    openxr::{AttachSpaceToEntity, OxrActionBlueprint, SpaceActionValue, OCULUS_TOUCH_PROFILE},
    prelude::*,
    ActionBundle, ActionSetBundle,
};
#[derive(Component, Clone, Copy)]
struct HandLeft;
#[derive(Component, Clone, Copy)]
struct HandRight;

#[derive(Component, Clone, Copy)]
struct MoveAction;
#[derive(Component, Clone, Copy)]
struct LookAction;
#[derive(Component, Clone, Copy)]
struct JumpAction;

fn main() {
    let mut app = App::new();
    app.add_plugins(bevy_mod_openxr::add_xr_plugins(DefaultPlugins));
    app.add_plugins(FrameTimeDiagnosticsPlugin);
    app.add_plugins(schminput::DefaultSchminputPlugins);
    app.add_systems(Startup, setup);
    app.add_systems(Startup, setup_env);
    app.add_systems(Update, run);

    app.run();
}
fn setup(mut cmds: Commands) {
    let player_set = cmds.spawn(ActionSetBundle::new("player", "Player")).id();
    let pose_set = cmds.spawn(ActionSetBundle::new("pose", "Poses")).id();
    cmds.spawn(ActionBundle::new("move", "Move", player_set))
        .insert((
            MoveAction,
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/left/input/thumbstick")
                .end(),
            Vec2ActionValue::default(),
        ));
    cmds.spawn(ActionBundle::new("look", "Look", player_set))
        .insert((
            LookAction,
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/thumbstick/x")
                .end(),
            F32ActionValue::default(),
        ));
    cmds.spawn(ActionBundle::new("jump", "Jump", player_set))
        .insert((
            JumpAction,
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/a/click")
                .end(),
            KeyboardBindings::default().add_binding(KeyboardBinding::new(KeyCode::Space)),
            GamepadBindings::default().add_binding(
                GamepadBinding::button(GamepadButtonType::South).button_just_pressed(),
            ),
            BoolActionValue::default(),
        ));
    let left_hand = cmds.spawn((SpatialBundle::default(), HandLeft)).id();

    let right_hand = cmds.spawn((SpatialBundle::default(), HandRight)).id();
    cmds.spawn(ActionBundle::new(
        "hand_left_pose",
        "Left Hand Pose",
        pose_set,
    ))
    .insert((
        MoveAction,
        OxrActionBlueprint::default()
            .interaction_profile(OCULUS_TOUCH_PROFILE)
            .binding("/user/hand/left/input/grip/pose")
            .end(),
        AttachSpaceToEntity(left_hand),
        SpaceActionValue::default(),
    ));
    cmds.spawn(ActionBundle::new(
        "hand_right_pose",
        "Right Hand Pose",
        pose_set,
    ))
    .insert((
        MoveAction,
        OxrActionBlueprint::default()
            .interaction_profile(OCULUS_TOUCH_PROFILE)
            .binding("/user/hand/right/input/aim/pose")
            .end(),
        AttachSpaceToEntity(right_hand),
        SpaceActionValue::default(),
    ));
}

fn run(
    move_action: Query<&Vec2ActionValue, With<MoveAction>>,
    look_action: Query<&F32ActionValue, With<LookAction>>,
    jump_action: Query<&BoolActionValue, With<JumpAction>>,
    left_hand: Query<&GlobalTransform, With<HandLeft>>,
    right_hand: Query<&GlobalTransform, With<HandRight>>,
    mut gizmos: Gizmos,
) {
    info!("move: {}", move_action.single().any);
    info!("look: {}", look_action.single().any);
    info!("jump: {}", jump_action.single().any);
    for hand in left_hand.into_iter() {
        let (_, rot, pos) = hand.to_scale_rotation_translation();
        gizmos.sphere(pos, rot, 0.1, css::ORANGE_RED);
    }
    for hand in right_hand.into_iter() {
        let (_, rot, pos) = hand.to_scale_rotation_translation();
        gizmos.sphere(pos, rot, 0.1, css::LIMEGREEN);
    }
}

fn setup_env(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(2.5)).mesh()),
        material: materials.add(StandardMaterial::from(Color::srgb(0.3, 0.5, 0.3))),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(0.1)))),
        material: materials.add(StandardMaterial::from(Color::srgb(0.8, 0.7, 0.6))),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
}
