pub mod default_bindings;
pub mod egui;
pub mod egui_window;
pub mod persistent_bindings;
pub mod runtime_rebinding;
pub mod str_converstions;
pub mod config;

use bevy::{app::PluginGroupBuilder, prelude::*};
use config::SchminputConfigPlugin;
use default_bindings::RebindingDefaultBindingsPlugin;
use egui_window::RebindingEguiWindowPlugin;
use persistent_bindings::PersistentBindingsPlugin;
use runtime_rebinding::RuntimeRebindingPlugin;
pub struct DefaultSchminputRebindingPlugins;

impl PluginGroup for DefaultSchminputRebindingPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut plugins = PluginGroupBuilder::start::<DefaultSchminputRebindingPlugins>();
        plugins = plugins
            .add(RuntimeRebindingPlugin)
            .add(PersistentBindingsPlugin)
            .add(SchminputConfigPlugin)
            .add(RebindingEguiWindowPlugin)
            .add(RebindingDefaultBindingsPlugin);
        plugins
    }
}
