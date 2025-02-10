use bevy::prelude::*;
use bevy_mod_xr::session::{
    XrCreateSessionEvent, XrRequestExitEvent, XrSessionDestroyedEvent, XrSessionPlugin,
};

pub struct SchmebindingXrUtilsPlugin;
impl Plugin for SchmebindingXrUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RestartXrSession>();
        if app.is_plugin_added::<XrSessionPlugin>() {
            app.add_systems(Last, on_restart_event.run_if(on_event::<RestartXrSession>));
            app.add_systems(
                PostUpdate,
                restart_session.run_if(
                    on_event::<XrSessionDestroyedEvent>.and(resource_exists::<ShouldRestart>),
                ),
            );
        }
    }
}

fn restart_session(mut event: EventWriter<XrCreateSessionEvent>, mut cmds: Commands) {
    event.send_default();
    cmds.remove_resource::<ShouldRestart>();
    info!("restarting session");
}

fn on_restart_event(mut cmds: Commands, mut event: EventWriter<XrRequestExitEvent>) {
    event.send_default();
    cmds.insert_resource(ShouldRestart(false));
    info!("on restart event");
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct ShouldRestart(bool);

#[derive(Event, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RestartXrSession;
