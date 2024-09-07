use bevy::prelude::*;
use bevy_egui::egui::{
    self, collapsing_header::CollapsingState, CollapsingHeader, Color32, DragValue, Id, RichText,
    Ui,
};
use schminput::{
    gamepad::GamepadHapticType, prelude::*, ActionsInSet, ButtonInputBeheavior, InputAxis,
    InputAxisDirection, LocalizedActionSetName,
};

#[cfg(feature = "xr")]
use crate::xr_utils::RestartXrSession;
use crate::{
    config::{LoadSchminputConfig, SaveSchminputConfig},
    default_bindings::ResetToDefautlBindings,
    egui::macros::collapsable,
    runtime_rebinding::{
        RequestGamepadRebinding, RequestKeyboardRebinding, RequestMouseRebinding, WaitingForInput,
    },
};
#[cfg(feature = "xr")]
use bevy_mod_openxr::resources::OxrInstance;

const DELETE_CHAR: char = '🗙';
fn get_delete_text() -> RichText {
    RichText::new(DELETE_CHAR)
        .monospace()
        .color(Color32::LIGHT_RED)
        .strong()
}

mod macros {
    macro_rules! collapsable {
        ($ui:expr,$entity:expr,$name:literal, $on_add:block, $body:expr) => {
            { CollapsingState::load_with_default_open(
            $ui.ctx(),
            Id::new(BindingIdHash {
                binding_index: 0,
                action: $entity,
                id: $name,
            }),
            false,
        )
        .show_header($ui, |ui: &mut Ui| {
            ui.horizontal(|ui| {
                ui.label($name);
                if ui.button(RichText::new('+').monospace()).clicked() {
                    $on_add
                };
            });
        })
        .body($body);}
        };
    }
    pub(crate) use collapsable;
}
/// Used when `xr` feature is not enabled, can be ignored
#[derive(Component)]
pub struct PlaceholderComponent;

#[cfg(not(feature = "xr"))]
pub type ActionQueryData<'a> = (
    Entity,
    Option<&'a mut KeyboardBindings>,
    Option<&'a mut MouseBindings>,
    Option<&'a mut GamepadBindings>,
    Option<&'a mut GamepadHapticOutputBindings>,
    Option<&'a mut PlaceholderComponent>,
    &'a LocalizedActionName,
    Has<BoolActionValue>,
);
#[cfg(feature = "xr")]
pub type ActionQueryData<'a> = (
    Entity,
    Option<&'a mut KeyboardBindings>,
    Option<&'a mut MouseBindings>,
    Option<&'a mut GamepadBindings>,
    Option<&'a mut GamepadHapticOutputBindings>,
    Option<&'a mut OxrActionBlueprint>,
    &'a LocalizedActionName,
    Has<BoolActionValue>,
);

pub fn draw_rebinding_ui(
    ui: &mut Ui,
    action_query: &mut Query<ActionQueryData>,
    action_type_query: &ActionStateQuery,
    set_query: &Query<(&LocalizedActionSetName, &ActionsInSet)>,
    waiting: &WaitingForInput,
    mut request_keyboard: EventWriter<RequestKeyboardRebinding>,
    mut mouse_rebind: EventWriter<RequestMouseRebinding>,
    mut gamepad_rebind: EventWriter<RequestGamepadRebinding>,
    mut reset_bindings: EventWriter<ResetToDefautlBindings>,
    mut request_save: EventWriter<SaveSchminputConfig>,
    mut request_load: EventWriter<LoadSchminputConfig>,
    #[cfg(feature = "xr")] mut request_session_restart: EventWriter<RestartXrSession>,
    #[cfg(feature = "xr")] instance: Res<OxrInstance>,
) {
    if waiting.waiting() {
        ui.heading("Waiting for input");
        return;
    }
    for (localized_set_name, actions) in set_query.iter() {
        CollapsingHeader::new(&***localized_set_name)
            .default_open(true)
            .show(ui, |ui| {
                let mut iter = action_query.iter_many_mut(actions.0.iter());
                while let Some((
                    entity,
                    keyboard,
                    mut mouse,
                    gamepad,
                    gamepad_haptics,
                    xr_blueprint,
                    localized_name,
                    is_bool_action,
                )) = iter.fetch_next()
                {
                    let action_type = ActionType::from_query(action_type_query, entity);
                    // idk what i am even derefing too
                    ui.collapsing(&***localized_name, |ui| {
                        if action_type != ActionType::GamepadHaptic {
                            collapsable!(
                                ui,
                                entity,
                                "Keyboard:",
                                {
                                    request_keyboard.send(RequestKeyboardRebinding::NewBinding {
                                        action: entity,
                                    });
                                },
                                |ui| {
                                    if let Some(mut keyboard) = keyboard {
                                        for (binding_index, binding) in
                                            keyboard.0.iter_mut().enumerate()
                                        {
                                            draw_keyboard_binding(
                                                ui,
                                                binding,
                                                is_bool_action,
                                                binding_index,
                                                entity,
                                                &mut request_keyboard,
                                            );
                                        }
                                    }
                                }
                            )
                        }
                        if action_type != ActionType::GamepadHaptic {
                            collapsable!(
                                ui,
                                entity,
                                "Mouse Motion:",
                                {
                                    mouse_rebind.send(RequestMouseRebinding::NewMotionBinding {
                                        action: entity,
                                    });
                                },
                                |ui| {
                                    // always triggers change detection
                                    if let Some(mouse) = mouse.as_mut() {
                                        if let Some(movement) = mouse.movement.as_mut() {
                                            draw_mouse_moiton_binding(
                                                ui,
                                                movement,
                                                entity,
                                                &mut mouse_rebind,
                                            );
                                        }
                                    }
                                }
                            );
                            collapsable!(
                                ui,
                                entity,
                                "Mouse Button:",
                                {
                                    mouse_rebind.send(RequestMouseRebinding::NewButtonBinding {
                                        action: entity,
                                    });
                                },
                                |ui| {
                                    // always triggers change detection
                                    if let Some(mouse) = mouse.as_mut() {
                                        for (binding_index, binding) in
                                            mouse.buttons.iter_mut().enumerate()
                                        {
                                            draw_mouse_button_binding(
                                                ui,
                                                binding,
                                                is_bool_action,
                                                binding_index,
                                                entity,
                                                &mut mouse_rebind,
                                            )
                                        }
                                    }
                                }
                            );
                        }
                        if action_type != ActionType::GamepadHaptic {
                            collapsable!(
                                ui,
                                entity,
                                "Gamepad:",
                                {
                                    gamepad_rebind.send(RequestGamepadRebinding::NewBinding {
                                        action: entity,
                                    });
                                },
                                |ui| {
                                    if let Some(mut gamepad) = gamepad {
                                        for (binding_index, binding) in
                                            gamepad.bindings.iter_mut().enumerate()
                                        {
                                            draw_gamepad_binding(
                                                ui,
                                                binding,
                                                is_bool_action,
                                                binding_index,
                                                entity,
                                                &mut gamepad_rebind,
                                            )
                                        }
                                    }
                                }
                            );
                        }
                        if action_type == ActionType::GamepadHaptic {
                            if let Some(mut gamepad_haptics) = gamepad_haptics {
                                collapsable!(
                                    ui,
                                    entity,
                                    "Gamepad Haptics:",
                                    {
                                        gamepad_haptics.bindings.push(GamepadHapticType::Weak);
                                    },
                                    |ui| {
                                        let mut delete = None;
                                        for (binding_index, binding) in
                                            gamepad_haptics.bindings.iter_mut().enumerate()
                                        {
                                            ui.horizontal(|ui| {
                                                egui::ComboBox::new(
                                                    BindingIdHash {
                                                        binding_index,
                                                        action: entity,
                                                        id: "gamepad_haptics",
                                                    },
                                                    "Haptics Type",
                                                )
                                                .width(0.0)
                                                .selected_text(
                                                    RichText::new(binding.to_string()).monospace(),
                                                )
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        binding,
                                                        GamepadHapticType::Weak,
                                                        RichText::new(
                                                            GamepadHapticType::Weak.to_string(),
                                                        )
                                                        .monospace(),
                                                    );
                                                    ui.selectable_value(
                                                        binding,
                                                        GamepadHapticType::Strong,
                                                        RichText::new(
                                                            GamepadHapticType::Strong.to_string(),
                                                        )
                                                        .monospace(),
                                                    );
                                                });
                                                if ui.button(get_delete_text()).clicked() {
                                                    delete = Some(binding_index);
                                                }
                                            });
                                        }
                                        if let Some(index) = delete {
                                            gamepad_haptics.bindings.remove(index);
                                        }
                                    }
                                );
                            }
                        }
                        if let Some(blueprint) = xr_blueprint {
                            collapsable!(ui, entity, "OpenXR Bindings:", {}, |ui| {})
                        }
                    })
                    .header_response
                    .on_hover_text(format!("Action Type: {}", action_type));
                }
            });
    }
    if ui.button("Reset All Bindings").clicked() {
        reset_bindings.send(ResetToDefautlBindings::All);
    }
    if ui.button("Save All Bindings").clicked() {
        request_save.send_default();
    }
    if ui.button("Load All Bindings").clicked() {
        request_load.send_default();
    }
    #[cfg(feature = "xr")]
    if ui.button("Restart Xr Session").clicked() {
        request_session_restart.send_default();
    }
}

pub fn draw_gamepad_binding(
    ui: &mut Ui,
    binding: &mut GamepadBinding,
    is_bool_action: bool,
    binding_index: usize,
    action: Entity,
    gamepad_rebind: &mut EventWriter<RequestGamepadRebinding>,
) {
    CollapsingState::load_with_default_open(
        ui.ctx(),
        Id::new(BindingIdHash {
            binding_index,
            action,
            id: "gamepad advanced",
        }),
        false,
    )
    .show_header(ui, |ui: &mut Ui| {
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new(format!("{}", binding.source)).monospace())
                .clicked()
            {
                gamepad_rebind.send(RequestGamepadRebinding::Rebind {
                    binding_index,
                    action,
                });
            }
            if ui.button(get_delete_text()).clicked() {
                gamepad_rebind.send(RequestGamepadRebinding::DeleteBinding {
                    binding_index,
                    action,
                });
            }
        });
    })
    .body(|ui| {
        if !is_bool_action {
            draw_input_axis(ui, &mut binding.axis, binding_index, action);
            draw_input_axis_dir(ui, &mut binding.axis_dir, binding_index, action);
        }
        if binding.source.as_button_type().is_some() {
            draw_button_behavior(ui, &mut binding.button_behavior, binding_index, action)
        }
    });
}

pub fn draw_mouse_button_binding(
    ui: &mut Ui,
    binding: &mut MouseButtonBinding,
    is_bool_action: bool,
    binding_index: usize,
    action: Entity,
    mouse_rebind: &mut EventWriter<RequestMouseRebinding>,
) {
    CollapsingState::load_with_default_open(
        ui.ctx(),
        Id::new(BindingIdHash {
            binding_index,
            action,
            id: "mouse button advanced",
        }),
        false,
    )
    .show_header(ui, |ui: &mut Ui| {
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new(format!("{:?}", binding.button)).monospace())
                .clicked()
            {
                mouse_rebind.send(RequestMouseRebinding::RebindButton {
                    binding_index,
                    action,
                });
            }
            if ui.button(get_delete_text()).clicked() {
                mouse_rebind.send(RequestMouseRebinding::DeleteButtonBinding {
                    binding_index,
                    action,
                });
            }
        });
    })
    .body(|ui| {
        if !is_bool_action {
            draw_input_axis(ui, &mut binding.axis, binding_index, action);
            draw_input_axis_dir(ui, &mut binding.axis_dir, binding_index, action);
        }

        // Will not pull out
        draw_button_behavior(ui, &mut binding.behavior, binding_index, action)
    });
}
pub fn draw_mouse_moiton_binding(
    ui: &mut Ui,
    binding: &mut MouseMotionBinding,
    action: Entity,
    mouse_rebind: &mut EventWriter<RequestMouseRebinding>,
) {
    ui.horizontal(|ui| {
        ui.label("sensitivity: ");
        ui.add(
            DragValue::new(&mut binding.multiplier)
                .speed(0.05)
                .update_while_editing(false),
        );
        if ui.button(get_delete_text()).clicked() {
            mouse_rebind.send(RequestMouseRebinding::DeleteMotionBinding { action });
        }
    });
}

pub fn draw_keyboard_binding(
    ui: &mut Ui,
    binding: &mut KeyboardBinding,
    is_bool_action: bool,
    binding_index: usize,
    action: Entity,
    request_keyboard: &mut EventWriter<RequestKeyboardRebinding>,
) {
    CollapsingState::load_with_default_open(
        ui.ctx(),
        Id::new(BindingIdHash {
            binding_index,
            action,
            id: "advanced",
        }),
        false,
    )
    .show_header(ui, |ui: &mut Ui| {
        ui.horizontal(|ui| {
            let key_string = format!("{:?}", binding.key);
            if ui
                .button(
                    RichText::new(key_string.strip_prefix("Key").unwrap_or(&key_string))
                        .monospace(),
                )
                .clicked()
            {
                request_keyboard.send(RequestKeyboardRebinding::RebindKey {
                    binding_index,
                    action,
                });
            }
            if ui.button(get_delete_text()).clicked() {
                request_keyboard.send(RequestKeyboardRebinding::DeleteBinding {
                    binding_index,
                    action,
                });
            }
        });
    })
    .body(|ui| {
        if !is_bool_action {
            draw_input_axis(ui, &mut binding.axis, binding_index, action);
            draw_input_axis_dir(ui, &mut binding.axis_dir, binding_index, action);
        }

        // Will not pull out
        draw_button_behavior(ui, &mut binding.behavior, binding_index, action)
    });
}

fn draw_button_behavior(
    ui: &mut Ui,
    behavior: &mut ButtonInputBeheavior,
    binding_index: usize,
    action: Entity,
) {
    let mut b = *behavior;
    egui::ComboBox::new(
        BindingIdHash {
            binding_index,
            action,
            id: "behavior",
        },
        "behavior",
    )
    .width(0.0)
    .selected_text(RichText::new(b.to_string()).monospace())
    .show_ui(ui, |ui| {
        ui.selectable_value(
            &mut b,
            ButtonInputBeheavior::Pressed,
            RichText::new(ButtonInputBeheavior::Pressed.to_string()).monospace(),
        );
        ui.selectable_value(
            &mut b,
            ButtonInputBeheavior::JustPressed,
            RichText::new(ButtonInputBeheavior::JustPressed.to_string()).monospace(),
        );
        ui.selectable_value(
            &mut b,
            ButtonInputBeheavior::JustReleased,
            RichText::new(ButtonInputBeheavior::JustReleased.to_string()).monospace(),
        );
    });
    // needed for correct change detection
    if b != *behavior {
        *behavior = b
    }
}
fn draw_input_axis(ui: &mut Ui, axis: &mut InputAxis, binding_index: usize, action: Entity) {
    egui::ComboBox::new(
        BindingIdHash {
            binding_index,
            action,
            id: "axis",
        },
        "axis",
    )
    .width(0.0)
    .selected_text(RichText::new(axis.to_string()).monospace())
    .show_ui(ui, |ui| {
        ui.selectable_value(
            axis,
            InputAxis::X,
            RichText::new(InputAxis::X.to_string()).monospace(),
        );
        ui.selectable_value(
            axis,
            InputAxis::Y,
            RichText::new(InputAxis::Y.to_string()).monospace(),
        );
    });
}
fn draw_input_axis_dir(
    ui: &mut Ui,
    dir: &mut InputAxisDirection,
    binding_index: usize,
    action: Entity,
) {
    egui::ComboBox::new(
        BindingIdHash {
            binding_index,
            action,
            id: "direction",
        },
        "direction",
    )
    .width(0.0)
    .selected_text(RichText::new(dir.to_string()).monospace())
    .show_ui(ui, |ui| {
        ui.selectable_value(
            dir,
            InputAxisDirection::Positive,
            RichText::new(InputAxisDirection::Positive.to_string()).monospace(),
        );
        ui.selectable_value(
            dir,
            InputAxisDirection::Negative,
            RichText::new(InputAxisDirection::Negative.to_string()).monospace(),
        );
    });
}

pub type ActionStateQuery<'world, 'state> = Query<
    'world,
    'state,
    (
        Has<BoolActionValue>,
        Has<F32ActionValue>,
        Has<Vec2ActionValue>,
        Has<GamepadHapticOutput>,
    ),
>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionType {
    Unkown,
    Bool,
    F32,
    Vec2,
    GamepadHaptic,
    // xr
    // XrSpace
    // XrHaptic
}
impl ActionType {
    fn from_query(query: &ActionStateQuery, entity: Entity) -> ActionType {
        let Ok((has_bool, has_f32, has_vec2, has_gamepad_haptic)) = query.get(entity) else {
            return ActionType::Unkown;
        };
        if has_bool {
            return ActionType::Bool;
        }
        if has_f32 {
            return ActionType::F32;
        }
        if has_vec2 {
            return ActionType::Vec2;
        }
        if has_gamepad_haptic {
            return ActionType::GamepadHaptic;
        }
        ActionType::Unkown
    }
}
impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ActionType::Unkown => "Unknown",
            ActionType::Bool => "Boolean",
            ActionType::F32 => "1D Axis",
            ActionType::Vec2 => "2D Axis",
            ActionType::GamepadHaptic => "Gamepad Haptics",
        })
    }
}

#[derive(Hash, Clone, Copy)]
struct BindingIdHash<'a> {
    binding_index: usize,
    action: Entity,
    id: &'a str,
}
