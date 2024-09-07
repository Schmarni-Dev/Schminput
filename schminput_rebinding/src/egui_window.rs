#[cfg(feature = "xr")]
use crate::xr_utils::RestartXrSession;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
#[cfg(feature = "xr")]
use bevy_mod_openxr::resources::OxrInstance;
use schminput::prelude::*;

use crate::{
    config::{LoadSchminputConfig, SaveSchminputConfig},
    default_bindings::ResetToDefautlBindings,
    egui::{ActionQueryData, ActionStateQuery},
    runtime_rebinding::{
        RequestGamepadRebinding, RequestKeyboardRebinding, RequestMouseRebinding, WaitingForInput,
    },
};

#[derive(Clone, Copy, Resource, Debug, Default, PartialEq, Eq)]
pub struct ShowEguiRebindingWindow(pub bool);

pub struct RebindingEguiWindowPlugin;

impl Plugin for RebindingEguiWindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShowEguiRebindingWindow>();
        app.add_systems(
            Update,
            draw_ui.run_if(resource_equals(ShowEguiRebindingWindow(true))),
        );
    }
}
fn draw_ui(
    mut ctxs: EguiContexts,
    mut action_query: Query<ActionQueryData>,
    set_query: Query<(&LocalizedActionSetName, &ActionsInSet)>,
    waiting: Res<WaitingForInput>,
    request_keyboard: EventWriter<RequestKeyboardRebinding>,
    action_type_query: ActionStateQuery,
    reset_bindings: EventWriter<ResetToDefautlBindings>,
    mouse_rebind: EventWriter<RequestMouseRebinding>,
    gamepad_rebind: EventWriter<RequestGamepadRebinding>,
    request_save: EventWriter<SaveSchminputConfig>,
    request_load: EventWriter<LoadSchminputConfig>,
    #[cfg(feature = "xr")] request_session_restart: EventWriter<RestartXrSession>,
    #[cfg(feature = "xr")] instance: Res<OxrInstance>,
) {
    egui::Window::new("Schminput Rebinding Ui").show(ctxs.ctx_mut(), |ui| {
        crate::egui::draw_rebinding_ui(
            ui,
            &mut action_query,
            &action_type_query,
            &set_query,
            &waiting,
            request_keyboard,
            mouse_rebind,
            gamepad_rebind,
            reset_bindings,
            request_save,
            request_load,
            #[cfg(feature = "xr")]
            request_session_restart,
            #[cfg(feature = "xr")]
            instance,
        );
    });
}
