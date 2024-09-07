use std::borrow::Cow;

use bevy::{
    input::{
        gamepad::{GamepadAxisChangedEvent, GamepadButtonInput},
        keyboard::KeyboardInput,
        mouse::MouseButtonInput,
        ButtonState,
    },
    prelude::*,
};
#[cfg(feature = "xr")]
use schminput::openxr::OxrActionBlueprint;
use schminput::{
    gamepad::{GamepadBinding, GamepadBindingSource, GamepadBindings},
    keyboard::KeyboardBindings,
    mouse::{MouseBindings, MouseButtonBinding, MouseMotionBinding},
};
#[cfg(feature = "xr")]
#[derive(Event)]
pub enum RequestOpenXrRebinding {
    DeleteBinding {
        binding_index: usize,
        profile: Cow<'static, str>,
        action: Entity,
    },
    DeleteProfile {
        profile: Cow<'static, str>,
        action: Entity,
    },
}

#[derive(Resource)]
enum PendingKeyboardRebinding {
    Rebind {
        binding_index: usize,
        action: Entity,
    },
    New {
        action: Entity,
    },
}

#[derive(Event, Clone, Copy)]
pub enum RequestKeyboardRebinding {
    RebindKey {
        binding_index: usize,
        action: Entity,
    },
    DeleteBinding {
        binding_index: usize,
        action: Entity,
    },
    NewBinding {
        action: Entity,
    },
}

#[derive(Resource)]
enum PendingGamepadRebinding {
    Rebind {
        binding_index: usize,
        action: Entity,
    },
    New {
        action: Entity,
    },
}

#[derive(Event, Clone, Copy)]
pub enum RequestGamepadRebinding {
    Rebind {
        binding_index: usize,
        action: Entity,
    },
    DeleteBinding {
        binding_index: usize,
        action: Entity,
    },
    NewBinding {
        action: Entity,
    },
}

#[derive(Resource)]
enum PendingMouseButtonRebinding {
    Rebind {
        binding_index: usize,
        action: Entity,
    },
    New {
        action: Entity,
    },
}
#[derive(Event, Clone, Copy)]
pub enum RequestMouseRebinding {
    RebindButton {
        binding_index: usize,
        action: Entity,
    },
    DeleteButtonBinding {
        binding_index: usize,
        action: Entity,
    },
    NewButtonBinding {
        action: Entity,
    },
    NewMotionBinding {
        action: Entity,
    },
    DeleteMotionBinding {
        action: Entity,
    },
}

#[derive(Clone, Copy, Resource)]
pub struct WaitingForInput(u8);
impl WaitingForInput {
    pub fn waiting(&self) -> bool {
        self.0 > 0
    }
}

pub struct RuntimeRebindingPlugin;
impl Plugin for RuntimeRebindingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaitingForInput(0));
        app.add_event::<RequestKeyboardRebinding>();
        app.add_event::<RequestMouseRebinding>();
        app.add_event::<RequestGamepadRebinding>();
        #[cfg(feature = "xr")]
        {
            app.add_event::<RequestOpenXrRebinding>();
        }
        app.add_systems(PostUpdate, handle_keyboard_request);
        app.add_systems(PostUpdate, handle_mouse_request);
        app.add_systems(PostUpdate, handle_gamepad_request);
        app.add_systems(
            PreUpdate,
            handle_keyboard_rebinding.run_if(resource_exists::<PendingKeyboardRebinding>),
        );
        app.add_systems(
            PreUpdate,
            handle_mouse_rebinding.run_if(resource_exists::<PendingMouseButtonRebinding>),
        );
        app.add_systems(
            PreUpdate,
            handle_gamepad_rebinding.run_if(resource_exists::<PendingGamepadRebinding>),
        );
        #[cfg(feature = "xr")]
        {
            app.add_systems(PostUpdate, handle_openxr_request);
        }
    }
}
#[cfg(feature = "xr")]
fn handle_openxr_request(
    mut event: EventReader<RequestOpenXrRebinding>,
    mut action_query: Query<&mut OxrActionBlueprint>,
) {
    match event.read().next() {
        Some(RequestOpenXrRebinding::DeleteBinding {
            profile,
            binding_index,
            action,
        }) => {
            let Ok(mut v) = action_query.get_mut(*action) else {
                return;
            };
            if let Some(bindings) = v.bindings.get_mut(profile) {
                bindings.remove(*binding_index);
            }
        }
        Some(RequestOpenXrRebinding::DeleteProfile { profile, action }) => {
            let Ok(mut v) = action_query.get_mut(*action) else {
                return;
            };
            v.bindings.remove(profile);
        }
        None => {}
    }
}

fn handle_gamepad_request(
    mut event: EventReader<RequestGamepadRebinding>,
    mut cmds: Commands,
    pending: Option<Res<PendingGamepadRebinding>>,
    mut action_query: Query<&mut GamepadBindings>,
    mut waiting: ResMut<WaitingForInput>,
) {
    if pending.is_some() {
        return;
    }
    match event.read().next().copied() {
        Some(RequestGamepadRebinding::Rebind {
            binding_index,
            action,
        }) => {
            cmds.insert_resource(PendingGamepadRebinding::Rebind {
                binding_index,
                action,
            });
            waiting.0 += 1;
        }
        Some(RequestGamepadRebinding::DeleteBinding {
            binding_index,
            action,
        }) => {
            let Ok(mut v) = action_query.get_mut(action) else {
                return;
            };
            v.bindings.remove(binding_index);
        }
        Some(RequestGamepadRebinding::NewBinding { action }) => {
            cmds.insert_resource(PendingGamepadRebinding::New { action });
            waiting.0 += 1;
        }
        None => {}
    }
}

fn handle_gamepad_rebinding(
    rebinding: Res<PendingGamepadRebinding>,
    mut action_query: Query<Option<&mut GamepadBindings>>,
    mut button_input: EventReader<GamepadButtonInput>,
    mut axis_input: EventReader<GamepadAxisChangedEvent>,
    mut cmds: Commands,
    mut waiting: ResMut<WaitingForInput>,
) {
    for input in button_input.read() {
        if input.state == ButtonState::Released {
            continue;
        }
        match *rebinding {
            PendingGamepadRebinding::Rebind {
                binding_index,
                action,
            } => {
                let Ok(Some(mut bindings)) = action_query.get_mut(action) else {
                    error!("keyboard rebinding request with invalid action entity");
                    return;
                };
                let Some(binding) = bindings.bindings.get_mut(binding_index) else {
                    error!("keyboard rebinding request with invalid binding index");
                    return;
                };
                binding.source = GamepadBindingSource::from_button_type(&input.button.button_type);
            }
            PendingGamepadRebinding::New { action } => {
                let Ok(bindings) = action_query.get_mut(action) else {
                    error!("keyboard rebinding request with invalid action entity");
                    return;
                };
                match bindings {
                    Some(mut bindings) => bindings.bindings.push(GamepadBinding::new(
                        GamepadBindingSource::from_button_type(&input.button.button_type),
                    )),
                    None => {
                        cmds.entity(action)
                            .insert(GamepadBindings::default().add_binding(GamepadBinding::new(
                                GamepadBindingSource::from_button_type(&input.button.button_type),
                            )));
                    }
                }
            }
        }
        cmds.remove_resource::<PendingGamepadRebinding>();
        waiting.0 = waiting.0.saturating_sub(1);
        break;
    }
    for input in axis_input.read() {
        if input.value.abs() < 0.6 {
            continue;
        }
        match *rebinding {
            PendingGamepadRebinding::Rebind {
                binding_index,
                action,
            } => {
                let Ok(Some(mut bindings)) = action_query.get_mut(action) else {
                    error!("keyboard rebinding request with invalid action entity");
                    return;
                };
                let Some(binding) = bindings.bindings.get_mut(binding_index) else {
                    error!("keyboard rebinding request with invalid binding index");
                    return;
                };
                binding.source = GamepadBindingSource::from_axis_type(&input.axis_type);
            }
            PendingGamepadRebinding::New { action } => {
                let Ok(bindings) = action_query.get_mut(action) else {
                    error!("keyboard rebinding request with invalid action entity");
                    return;
                };
                match bindings {
                    Some(mut bindings) => bindings.bindings.push(GamepadBinding::new(
                        GamepadBindingSource::from_axis_type(&input.axis_type),
                    )),
                    None => {
                        cmds.entity(action)
                            .insert(GamepadBindings::default().add_binding(GamepadBinding::new(
                                GamepadBindingSource::from_axis_type(&input.axis_type),
                            )));
                    }
                }
            }
        }
        cmds.remove_resource::<PendingGamepadRebinding>();
        waiting.0 = waiting.0.saturating_sub(1);
        break;
    }
}

fn handle_mouse_request(
    mut event: EventReader<RequestMouseRebinding>,
    mut cmds: Commands,
    pending: Option<Res<PendingMouseButtonRebinding>>,
    mut action_query: Query<&mut MouseBindings>,
    mut waiting: ResMut<WaitingForInput>,
) {
    if pending.is_some() {
        return;
    }
    match event.read().next().copied() {
        Some(RequestMouseRebinding::RebindButton {
            binding_index,
            action,
        }) => {
            cmds.insert_resource(PendingMouseButtonRebinding::Rebind {
                binding_index,
                action,
            });
            waiting.0 += 1;
        }
        Some(RequestMouseRebinding::DeleteButtonBinding {
            binding_index,
            action,
        }) => {
            let Ok(mut v) = action_query.get_mut(action) else {
                return;
            };
            v.buttons.remove(binding_index);
        }
        Some(RequestMouseRebinding::DeleteMotionBinding { action }) => {
            let Ok(mut v) = action_query.get_mut(action) else {
                return;
            };
            v.movement = None;
        }
        Some(RequestMouseRebinding::NewButtonBinding { action }) => {
            cmds.insert_resource(PendingMouseButtonRebinding::New { action });
            waiting.0 += 1;
        }
        Some(RequestMouseRebinding::NewMotionBinding { action }) => {
            let Ok(mut v) = action_query.get_mut(action) else {
                return;
            };
            v.movement = Some(MouseMotionBinding::default());
        }
        None => {}
    }
}

fn handle_mouse_rebinding(
    rebinding: Res<PendingMouseButtonRebinding>,
    mut action_query: Query<Option<&mut MouseBindings>>,
    mut input: EventReader<MouseButtonInput>,
    mut cmds: Commands,
    mut waiting: ResMut<WaitingForInput>,
) {
    for input in input.read() {
        if input.state == ButtonState::Released {
            continue;
        }
        match *rebinding {
            PendingMouseButtonRebinding::Rebind {
                binding_index,
                action,
            } => {
                let Ok(Some(mut bindings)) = action_query.get_mut(action) else {
                    error!("keyboard rebinding request with invalid action entity");
                    return;
                };
                let Some(binding) = bindings.buttons.get_mut(binding_index) else {
                    error!("keyboard rebinding request with invalid binding index");
                    return;
                };
                binding.button = input.button;
            }
            PendingMouseButtonRebinding::New { action } => {
                let Ok(bindings) = action_query.get_mut(action) else {
                    error!("keyboard rebinding request with invalid action entity");
                    return;
                };
                match bindings {
                    Some(mut bindings) => {
                        bindings.buttons.push(MouseButtonBinding::new(input.button))
                    }
                    None => {
                        let mut bindings = MouseBindings::default();
                        bindings.buttons.push(MouseButtonBinding::new(input.button));
                        cmds.entity(action).insert(bindings);
                    }
                }
            }
        }
        cmds.remove_resource::<PendingMouseButtonRebinding>();
        waiting.0 = waiting.0.saturating_sub(1);
        break;
    }
}

fn handle_keyboard_request(
    mut event: EventReader<RequestKeyboardRebinding>,
    mut cmds: Commands,
    pending: Option<Res<PendingKeyboardRebinding>>,
    mut action_query: Query<&mut KeyboardBindings>,
    mut waiting: ResMut<WaitingForInput>,
) {
    if pending.is_some() {
        return;
    }
    match event.read().next().copied() {
        Some(RequestKeyboardRebinding::RebindKey {
            binding_index,
            action,
        }) => {
            cmds.insert_resource(PendingKeyboardRebinding::Rebind {
                binding_index,
                action,
            });
            waiting.0 += 1;
        }
        Some(RequestKeyboardRebinding::DeleteBinding {
            binding_index,
            action,
        }) => {
            let Ok(mut v) = action_query.get_mut(action) else {
                return;
            };
            v.0.remove(binding_index);
        }
        Some(RequestKeyboardRebinding::NewBinding { action }) => {
            cmds.insert_resource(PendingKeyboardRebinding::New { action });
            waiting.0 += 1;
        }
        None => {}
    }
}

fn handle_keyboard_rebinding(
    rebinding: Res<PendingKeyboardRebinding>,
    mut action_query: Query<Option<&mut KeyboardBindings>>,
    mut input: EventReader<KeyboardInput>,
    mut cmds: Commands,
    mut waiting: ResMut<WaitingForInput>,
) {
    for input in input.read() {
        if input.state == ButtonState::Released {
            continue;
        }
        match *rebinding {
            PendingKeyboardRebinding::Rebind {
                binding_index,
                action,
            } => {
                let Ok(Some(mut bindings)) = action_query.get_mut(action) else {
                    error!("keyboard rebinding request with invalid action entity");
                    return;
                };
                let Some(binding) = bindings.0.get_mut(binding_index) else {
                    error!("keyboard rebinding request with invalid binding index");
                    return;
                };
                binding.key = input.key_code;
            }
            PendingKeyboardRebinding::New { action } => {
                let Ok(bindings) = action_query.get_mut(action) else {
                    error!("keyboard rebinding request with invalid action entity");
                    return;
                };
                match bindings {
                    Some(mut bindings) => bindings
                        .0
                        .push(schminput::keyboard::KeyboardBinding::new(input.key_code)),
                    None => {
                        let mut bindings = KeyboardBindings::default();
                        bindings
                            .0
                            .push(schminput::keyboard::KeyboardBinding::new(input.key_code));
                        cmds.entity(action).insert(bindings);
                    }
                }
            }
        }
        cmds.remove_resource::<PendingKeyboardRebinding>();
        waiting.0 = waiting.0.saturating_sub(1);
        break;
    }
}
