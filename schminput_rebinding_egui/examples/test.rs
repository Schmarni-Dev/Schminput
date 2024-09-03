use bevy::prelude::*;
use schminput::prelude::*;
use schminput_rebinding_egui::persistent_bindings::PersistentBindingsPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultSchminputPlugins)
        .add_plugins(PersistentBindingsPlugin)
        .add_systems(PreUpdate, || panic!("Stop"))
        .run()
}
