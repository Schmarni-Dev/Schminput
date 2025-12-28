use bevy::prelude::*;
use bevy_mod_xr::session::{
    XrCreateSessionMessage, XrRequestExitMessage, XrSessionDestroyedMessage, XrSessionPlugin,
};

pub struct SchmebindingXrUtilsPlugin;
impl Plugin for SchmebindingXrUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RestartXrSession>();
        if app.is_plugin_added::<XrSessionPlugin>() {
            app.add_systems(Last, on_restart_message.run_if(on_message::<RestartXrSession>));
            app.add_systems(
                PostUpdate,
                restart_session.run_if(
                    on_message::<XrSessionDestroyedMessage>.and(resource_exists::<ShouldRestart>),
                ),
            );
        }
    }
}

fn restart_session(mut message: MessageWriter<XrCreateSessionMessage>, mut cmds: Commands) {
    message.write_default();
    cmds.remove_resource::<ShouldRestart>();
    info!("restarting session");
}

fn on_restart_message(mut cmds: Commands, mut message: MessageWriter<XrRequestExitMessage>) {
    message.write_default();
    cmds.insert_resource(ShouldRestart(false));
    info!("on restart message");
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct ShouldRestart(bool);

#[derive(Message, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RestartXrSession;
