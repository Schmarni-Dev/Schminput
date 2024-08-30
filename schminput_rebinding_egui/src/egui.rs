use std::ops::Deref;

use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, RichText, Ui};
use schminput::{prelude::*, ActionsInSet, InputAxis, InputAxisDirection, LocalizedActionSetName};

pub fn draw_rebinding_ui(
    ui: &mut Ui,
    action_query: &mut Query<(
        Entity,
        Option<&mut KeyboardBindings>,
        Option<&mut MouseBindings>,
        Option<&mut GamepadBindings>,
        &ActionName,
        &LocalizedActionName,
        Has<BoolActionValue>,
    )>,
    set_query: &Query<(&LocalizedActionSetName, &ActionsInSet)>,
    
) {
    for (localized_set_name, actions) in set_query.iter() {
        // idk what i am even derefing too
        ui.label(&***localized_set_name);

        ui.indent("set", |ui| {
            for (
                entity,
                mut keyboard,
                mut mouse,
                mut gamepad,
                name,
                localized_name,
                is_bool_action,
            ) in action_query.iter_mut()
            {
                // idk what i am even derefing too
                ui.label(&***localized_name);
                if let Some(mut keyboard) = keyboard {
                    ui.label("  Keyboard:");
                    ui.indent("binding", |ui| {
                        for (binding_index, binding) in keyboard.0.iter_mut().enumerate() {
                            draw_keyboard_binding(
                                ui,
                                binding,
                                is_bool_action,
                                binding_index,
                                entity,
                            );
                        }
                    });
                }
            }
        });
    }
    // ui.label("Hello, World!");
}

#[derive(Resource)]
struct PendingKeyboardRebinding {
    binding_index: usize,
    action: Entity,
}

#[derive(Hash, Clone, Copy)]
struct BindingIdHash {
    binding_index: usize,
    action: Entity,
    id: &'static str,
}

pub fn draw_keyboard_binding(
    ui: &mut Ui,
    mut binding: &mut KeyboardBinding,
    is_bool_action: bool,
    binding_index: usize,
    action: Entity,
) {
    ui.horizontal(|ui| {
        if !is_bool_action {
            let mut axis = binding.axis;
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
                    &mut axis,
                    InputAxis::X,
                    RichText::new(InputAxis::X.to_string()).monospace(),
                );
                ui.selectable_value(
                    &mut axis,
                    InputAxis::Y,
                    RichText::new(InputAxis::Y.to_string()).monospace(),
                );
            });

            // needed for correct change detection
            if axis != binding.axis {
                binding.axis = axis
            }

            let mut dir = binding.axis_dir;
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
                    &mut dir,
                    InputAxisDirection::Positive,
                    RichText::new(InputAxisDirection::Positive.to_string()).monospace(),
                );
                ui.selectable_value(
                    &mut dir,
                    InputAxisDirection::Negative,
                    RichText::new(InputAxisDirection::Negative.to_string()).monospace(),
                );
            });
            // needed for correct change detection
            if dir != binding.axis_dir {
                binding.axis_dir = dir
            }
        }
        let key_string = format!("{:?}", binding.key);
        if ui
            .button(
                RichText::new(key_string.strip_prefix("Key").unwrap_or(&key_string)).monospace(),
            )
            .clicked()
        {
            info!("TODO: accept input change request");
        }
        if ui
            .button(RichText::new('X').monospace())
            .clicked()
        {
            info!("TODO: Delete Binding");
        }
    });
}
