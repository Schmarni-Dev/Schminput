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
    let player_set = cmds.spawn(ActionSet::new("player", "Player")).id();
    let pose_set = cmds.spawn(ActionSet::new("pose", "Poses")).id();
    let move_action = cmds
        .spawn((
            Action::new("move", "Move", player_set),
            OxrBindings::new().bindngs(OCULUS_TOUCH_PROFILE, ["/user/hand/left/input/thumbstick"]),
            Vec2ActionValue::new(),
        ))
        .id();
    let look = cmds
        .spawn((
            Action::new("look", "Look", player_set),
            OxrBindings::new().bindngs(
                OCULUS_TOUCH_PROFILE,
                ["/user/hand/right/input/thumbstick/x"],
            ),
            F32ActionValue::new(),
        ))
        .id();
    let jump = cmds
        .spawn((
            Action::new("jump", "Jump", player_set),
            OxrBindings::new().bindngs(OCULUS_TOUCH_PROFILE, ["/user/hand/right/input/a/click"]),
            KeyboardBindings::new().bind(KeyboardBinding::new(KeyCode::Space)),
            GamepadBindings::new().bind(
                GamepadBinding::new(GamepadBindingSource::South).button_just_pressed(),
            ),
            BoolActionValue::new(),
        ))
        .id();
    let left_hand = cmds.spawn(HandLeft).id();

    let right_hand = cmds.spawn(HandRight).id();
    let left_pose = cmds
        .spawn((
            Action::new("hand_left_pose", "Left Hand Pose", pose_set),
            OxrBindings::new().bindngs(OCULUS_TOUCH_PROFILE, ["/user/hand/left/input/grip/pose"]),
            AttachSpaceToEntity(left_hand),
            SpaceActionValue::new(),
        ))
        .id();
    let right_pose = cmds
        .spawn((
            Action::new("hand_right_pose", "Right Hand Pose", pose_set),
            OxrBindings::new().bindngs(OCULUS_TOUCH_PROFILE, ["/user/hand/right/input/aim/pose"]),
            AttachSpaceToEntity(right_hand),
            SpaceActionValue::new(),
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
        let pose = hand.to_isometry();
        gizmos.sphere(pose, 0.1, css::ORANGE_RED);
    }
    for hand in right_hand.into_iter() {
        let pose = hand.to_isometry();
        gizmos.sphere(pose, 0.1, css::LIMEGREEN);
    }
}

fn setup_env(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(2.5)).mesh())),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.3, 0.5, 0.3)))),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(0.1))))),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.8, 0.7, 0.6)))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}
