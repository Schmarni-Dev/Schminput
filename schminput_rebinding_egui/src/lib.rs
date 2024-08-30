pub mod egui;
pub mod runtime_rebinding;

use bevy::{app::PluginGroupBuilder, prelude::*};
pub struct DefaultSchminputRebindingPlugins;


impl PluginGroup for DefaultSchminputRebindingPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut plugins = PluginGroupBuilder::start::<DefaultSchminputRebindingPlugins>();
        plugins
    }
}
