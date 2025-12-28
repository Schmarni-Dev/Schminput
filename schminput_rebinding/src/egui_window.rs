#[cfg(feature = "xr")]
use crate::runtime_rebinding::RequestOpenXrRebinding;
#[cfg(feature = "xr")]
use crate::xr_utils::RestartXrSession;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use schminput::{ActionsInSet, prelude::*};

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
            EguiPrimaryContextPass,
            draw_ui.run_if(resource_equals(ShowEguiRebindingWindow(true))),
        );
    }
}
fn draw_ui(
    mut ctxs: EguiContexts,
    mut action_query: Query<ActionQueryData>,
    set_query: Query<(&ActionSet, &ActionsInSet)>,
    waiting: Res<WaitingForInput>,
    request_keyboard: MessageWriter<RequestKeyboardRebinding>,
    action_type_query: ActionStateQuery,
    reset_bindings: MessageWriter<ResetToDefautlBindings>,
    mouse_rebind: MessageWriter<RequestMouseRebinding>,
    gamepad_rebind: MessageWriter<RequestGamepadRebinding>,
    request_save: MessageWriter<SaveSchminputConfig>,
    request_load: MessageWriter<LoadSchminputConfig>,
    #[cfg(feature = "xr")] request_session_restart: MessageWriter<RestartXrSession>,
    #[cfg(feature = "xr")] openxr_rebind: MessageWriter<RequestOpenXrRebinding>,
) {
    let Ok(ctx) = ctxs.ctx_mut() else {
        return;
    };
    egui::Window::new("Schminput Rebinding Ui").show(ctx, |ui| {
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
            openxr_rebind,
        );
    });
}
