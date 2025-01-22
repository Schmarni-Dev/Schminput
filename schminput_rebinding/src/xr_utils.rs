use bevy::prelude::*;
use bevy_mod_xr::session::{XrCreateSessionEvent, XrPreDestroySession, XrRequestExitEvent};

pub struct SchmebindingXrUtilsPlugin;
impl Plugin for SchmebindingXrUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RestartXrSession>();
        app.add_systems(
            PostUpdate,
            on_restart_event.run_if(on_event::<RestartXrSession>),
        );
        app.add_systems(XrPreDestroySession, |v: Option<ResMut<ShouldRestart>>| {
            if let Some(mut v) = v {
                v.after_destroy = true;
            }
        });
        app.add_systems(
            Update,
            (|mut v: ResMut<ShouldRestart>| v.frame_counter += 1)
                .run_if(|v: Option<Res<ShouldRestart>>| v.is_some_and(|v| v.after_destroy)),
        );
        app.add_systems(
            PostUpdate,
            start_on_destroy.run_if(|v: Option<Res<ShouldRestart>>| {
                v.is_some_and(|v| v.after_destroy && v.frame_counter >= 5)
            }),
        );
    }
}

fn start_on_destroy(mut event: EventWriter<XrCreateSessionEvent>, mut cmds: Commands) {
    event.send_default();
    cmds.remove_resource::<ShouldRestart>();
}

fn on_restart_event(mut cmds: Commands, mut event: EventWriter<XrRequestExitEvent>) {
    event.send_default();
    cmds.insert_resource(ShouldRestart::default());
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct ShouldRestart {
    after_destroy: bool,
    frame_counter: u8,
}

#[derive(Event, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RestartXrSession;
