use bevy::{color::palettes::css, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use schminput::prelude::*;
#[derive(Component, Clone, Copy)]
struct HandLeft;
#[derive(Component, Clone, Copy)]
struct HandRight;

#[allow(dead_code)]
#[derive(Resource, Clone, Copy)]
struct CoreActions {
    set: Entity,
    left_pose: Entity,
    right_pose: Entity,
}

#[allow(dead_code)]
#[derive(Resource, Clone, Copy)]
struct MoveActions {
    set: Entity,
    move_action: Entity,
    look: Entity,
    jump: Entity,
}

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
    let move_action = cmds
        .spawn(ActionBundle::new("move", "Move", player_set))
        .insert((
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/left/input/thumbstick")
                .end(),
            Vec2ActionValue::default(),
        ))
        .id();
    let look = cmds
        .spawn(ActionBundle::new("look", "Look", player_set))
        .insert((
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/thumbstick/x")
                .end(),
            F32ActionValue::default(),
        ))
        .id();
    let jump = cmds
        .spawn(ActionBundle::new("jump", "Jump", player_set))
        .insert((
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/a/click")
                .end(),
            KeyboardBindings::default().add_binding(KeyboardBinding::new(KeyCode::Space)),
            GamepadBindings::default().add_binding(
                GamepadBinding::new(GamepadBindingSource::South).button_just_pressed(),
            ),
            BoolActionValue::default(),
        ))
        .id();
    let left_hand = cmds.spawn((SpatialBundle::default(), HandLeft)).id();

    let right_hand = cmds.spawn((SpatialBundle::default(), HandRight)).id();
    let left_pose = cmds
        .spawn(ActionBundle::new(
            "hand_left_pose",
            "Left Hand Pose",
            pose_set,
        ))
        .insert((
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/left/input/grip/pose")
                .end(),
            AttachSpaceToEntity(left_hand),
            SpaceActionValue::default(),
        ))
        .id();
    let right_pose = cmds
        .spawn(ActionBundle::new(
            "hand_right_pose",
            "Right Hand Pose",
            pose_set,
        ))
        .insert((
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/aim/pose")
                .end(),
            AttachSpaceToEntity(right_hand),
            SpaceActionValue::default(),
        ))
        .id();
    cmds.insert_resource(MoveActions {
        set: player_set,
        move_action,
        look,
        jump,
    });
    cmds.insert_resource(CoreActions {
        set: pose_set,
        left_pose,
        right_pose,
    });
}

fn run(
    move_actions: Res<MoveActions>,
    vec2_value: Query<&Vec2ActionValue>,
    f32_value: Query<&F32ActionValue>,
    bool_value: Query<&BoolActionValue>,
    left_hand: Query<&GlobalTransform, With<HandLeft>>,
    right_hand: Query<&GlobalTransform, With<HandRight>>,
    mut gizmos: Gizmos,
) {
    info!(
        "move: {}",
        vec2_value.get(move_actions.move_action).unwrap().any
    );
    info!("look: {}", f32_value.get(move_actions.look).unwrap().any);
    info!("jump: {}", bool_value.get(move_actions.jump).unwrap().any);
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
