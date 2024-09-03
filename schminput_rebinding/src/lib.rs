pub mod config;
pub mod default_bindings;
#[cfg(feature = "egui")]
pub mod egui;
#[cfg(feature = "egui")]
pub mod egui_window;
pub mod persistent_bindings;
pub mod runtime_rebinding;
pub mod str_converstions;

use bevy::{app::PluginGroupBuilder, prelude::*};
use config::SchminputConfigPlugin;
use default_bindings::RebindingDefaultBindingsPlugin;
#[cfg(feature = "egui")]
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
            .add(RebindingDefaultBindingsPlugin);

        #[cfg(feature = "egui")]
        #[allow(clippy::unnecessary_operation)]
        {
            plugins = plugins.add(RebindingEguiWindowPlugin)
        };

        plugins
    }
}
