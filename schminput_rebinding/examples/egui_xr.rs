use bevy::{color::palettes::css, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_egui::EguiPlugin;
use schminput::prelude::*;
use schminput_rebinding::{
    config::{ConfigFilePath, LoadSchminputConfig},
    egui_window::ShowEguiRebindingWindow,
    DefaultSchminputRebindingPlugins,
};
use std::path::PathBuf;

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
    app.insert_resource(ShowEguiRebindingWindow(true));
    app.insert_resource(ConfigFilePath::Path(PathBuf::from("./config/egui_xr.toml")));
    app.add_plugins(bevy_mod_openxr::add_xr_plugins(DefaultPlugins));
    app.add_plugins(FrameTimeDiagnosticsPlugin);
    app.add_plugins(schminput::DefaultSchminputPlugins);
    app.add_plugins(DefaultSchminputRebindingPlugins);
    app.world_mut().send_event(LoadSchminputConfig);

    app.add_plugins(EguiPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(Startup, setup_env);
    app.add_systems(Update, run);

    app.run();
}
fn setup(mut cmds: Commands) {
    cmds.spawn(Camera3d::default());
    let player_set = cmds.spawn(ActionSet::new("movement", "Movement")).id();
    let pose_set = cmds.spawn(ActionSet::new("core", "Core")).id();
    cmds.spawn((
        Action::new("move", "Move", player_set),
        MoveAction,
        OxrBindings::new()
            .interaction_profile(OCULUS_TOUCH_PROFILE)
            .binding("/user/hand/left/input/thumbstick")
            .end(),
        Vec2ActionValue::new(),
    ));
    cmds.spawn((
        Action::new("look", "Look", player_set),
        LookAction,
        OxrBindings::new()
            .interaction_profile(OCULUS_TOUCH_PROFILE)
            .binding("/user/hand/right/input/thumbstick/x")
            .end(),
        F32ActionValue::new(),
    ));
    cmds.spawn((
        Action::new("jump", "Jump", player_set),
        JumpAction,
        OxrBindings::new()
            .interaction_profile(OCULUS_TOUCH_PROFILE)
            .binding("/user/hand/right/input/a/click")
            .end(),
        KeyboardBindings::new().bind(KeyboardBinding::new(KeyCode::Space)),
        GamepadBindings::new()
            .bind(GamepadBinding::new(GamepadBindingSource::South).button_just_pressed()),
        BoolActionValue::new(),
    ));
    let left_hand = cmds.spawn(HandLeft).id();

    let right_hand = cmds.spawn(HandRight).id();
    cmds.spawn((
        Action::new("hand_left_pose", "Left Hand Pose", pose_set),
        MoveAction,
        OxrBindings::new()
            .interaction_profile(OCULUS_TOUCH_PROFILE)
            .binding("/user/hand/left/input/grip/pose")
            .end(),
        AttachSpaceToEntity(left_hand),
        SpaceActionValue::new(),
    ));
    cmds.spawn((
        Action::new("hand_right_pose", "Right Hand Pose", pose_set),
        MoveAction,
        OxrBindings::new()
            .interaction_profile(OCULUS_TOUCH_PROFILE)
            .binding("/user/hand/right/input/aim/pose")
            .end(),
        AttachSpaceToEntity(right_hand),
        SpaceActionValue::new(),
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
    // info!("move: {}", move_action.single().any);
    // info!("look: {}", look_action.single().any);
    // info!("jump: {}", jump_action.single().any);
    for hand in left_hand.into_iter() {
        let pose = hand.to_isometry();
        gizmos.sphere(pose, 0.01, css::ORANGE_RED);
    }
    for hand in right_hand.into_iter() {
        let pose = hand.to_isometry();
        gizmos.sphere(pose, 0.01, css::LIMEGREEN);
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
