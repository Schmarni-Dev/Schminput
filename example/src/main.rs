use bevy::prelude::*;
use schminput_schmanager::{
    keyboard_binding_provider::{
        KeyboardActivationType, KeyboardBinding, KeyboardBindingProvider, KeyboardBingings,
    },
    new_action, ErasedAction, SchminputApp,
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

new_action!(
    ExampleAction,
    Vec2,
    "example_action",
    "Example Action".into(),
    ExampleActionSet::key(),
    ExampleActionSet::name()
);
new_action!(
    BoolAction,
    bool,
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
    app.add_systems(Startup, setup);
    app.add_systems(Update, run);
    app.add_systems(PostUpdate, reset);

    app.run();
}

/// Will be put into lib later
fn reset(mut input: Query<&mut dyn ErasedAction>) {
    input
        .par_iter_mut()
        .for_each(|mut e| e.iter_mut().for_each(|mut a| a.reset_value()));
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

fn setup(mut keyboard: ResMut<KeyboardBingings>, mut commands: Commands) {
    let example = ExampleAction::default();
    let bool = BoolAction::default();
    keyboard.add_binding(
        &bool,
        KeyboardBinding::Simple(KeyboardActivationType::JustPressed, KeyCode::C),
    );
    keyboard.add_binding(
        &example,
        KeyboardBinding::Dpad {
            up: KeyCode::W,
            down: KeyCode::S,
            left: KeyCode::A,
            right: KeyCode::D,
        },
    );
    commands.spawn((example, bool));
}
