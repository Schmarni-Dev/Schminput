use bevy::prelude::*;
use bevy_schminput::{
    basic_action,
    keyboard_binding_provider::{
        KeyBinding, KeyboardBinding, KeyboardBindingProvider, KeyboardBindings,
    },
    mouse::{
        mouse_binding_provider::{MouseBinding, MouseBindingProvider},
        MouseBindings,
    },
    mouse_action, SchminputApp, SchminputPlugin,
};

pub struct ExampleActionSet;
impl ExampleActionSet {
    pub fn name() -> String {
        "Example Action Set".into()
    }
    pub fn key() -> &'static str {
        "example_action_set"
    }
}

mouse_action!(
    MouseAction,
    "mouse_action",
    "Mouse Action".into(),
    ExampleActionSet::key(),
    ExampleActionSet::name()
);

basic_action!(
    ExampleAction,
    Vec2,
    "example_action",
    "Example Action".into(),
    ExampleActionSet::key(),
    ExampleActionSet::name()
);
basic_action!(
    BoolAction,
    bool,
    "bool_action",
    "Bool Action".into(),
    ExampleActionSet::key(),
    ExampleActionSet::name()
);
basic_action!(
    TransformAction,
    Transform,
    "bool_action",
    "Bool Action".into(),
    ExampleActionSet::key(),
    ExampleActionSet::name()
);

fn main() {
    let mut app = App::new();
    app.register_action::<ExampleAction>();
    app.register_action::<BoolAction>();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(KeyboardBindingProvider);
    app.add_plugins(MouseBindingProvider);
    app.add_plugins(SchminputPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(Update, run);

    app.run();
}

fn run(input: Query<&ExampleAction>, bools: Query<&BoolAction>) {
    for action in input.into_iter() {
        info!("{:?}", action.data);
    }
    for action in bools.into_iter() {
        if action.data {
            info!("Hello World!");
        }
    }
}

fn setup(
    mut keyboard: ResMut<KeyboardBindings>,
    mut mouse: ResMut<MouseBindings>,
    mut commands: Commands,
) {
    let example = ExampleAction::default();
    let bool = BoolAction::default();
    keyboard.add_binding(
        &bool,
        KeyboardBinding::Simple(KeyBinding::JustPressed(KeyCode::KeyC)),
    );
    keyboard.add_binding(
        &example,
        KeyboardBinding::Dpad {
            up: KeyBinding::Held(KeyCode::KeyW),
            down: KeyBinding::Held(KeyCode::KeyS),
            left: KeyBinding::Held(KeyCode::KeyA),
            right: KeyBinding::Held(KeyCode::KeyD),
        },
    );
    mouse.add_binding(&bool, MouseBinding::JustPressed(MouseButton::Left));
    mouse.add_binding(&bool, MouseBinding::JustReleased(MouseButton::Left));
    mouse.add_binding(&bool, MouseBinding::Held(MouseButton::Right));
    commands.spawn((example, bool));
}
