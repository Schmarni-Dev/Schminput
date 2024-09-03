use bevy::prelude::*;
use schminput::prelude::*;
use toml_edit::{value, DocumentMut, Item, TableLike, Value};

use crate::str_converstions::{
    button_behavior_to_str, gamepad_binding_source_to_cow_str, gamepad_haptics_type_to_str,
    input_axis_dir_to_str, input_axis_to_str, key_code_to_str, mouse_button_to_cow_str,
    str_to_button_behavior, str_to_gamepad_binding_source, str_to_gamepad_haptics_type,
    str_to_input_axis, str_to_input_axis_dir, str_to_key_code, str_to_mouse_button,
};

pub struct PersistentBindingsPlugin;

impl Plugin for PersistentBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, (serialize_v1, deserialize_v1).chain());
    }
}

fn implicit_table() -> toml_edit::Item {
    let mut w = toml_edit::Table::new();
    w.set_implicit(true);
    toml_edit::Item::Table(w)
}

#[derive(Resource)]
struct ConfigDocument(DocumentMut);

fn serialize_v1(
    mut action_query: Query<(
        Option<&KeyboardBindings>,
        Option<&MouseBindings>,
        Option<&GamepadBindings>,
        Option<&GamepadHapticOutputBindings>,
        &ActionName,
    )>,
    set_query: Query<(&ActionSetName, &ActionsInSet)>,
    mut cmds: Commands,
) {
    let mut owned_doc = toml_edit::DocumentMut::new();
    let doc = &mut owned_doc;
    doc.entry("version").or_insert(toml_edit::value(1i64));
    for (set_name, actions) in &set_query {
        let mut iter = action_query.iter_many_mut(actions.0.iter());
        while let Some((keyboard, mouse, gamepad, gamepad_haptics, name)) = iter.fetch_next() {
            let doc_bindings = doc
                .entry(&set_name.0)
                .or_insert(implicit_table())
                .as_table_mut()
                .unwrap()
                .entry(&name.0)
                .or_insert(toml_edit::table());
            if let Some(keyboard) = keyboard {
                let mut bindings_list = toml_edit::Array::new();
                for binding in keyboard.0.iter() {
                    let mut table = toml_edit::InlineTable::new();
                    table.insert("key", key_code_to_str(&binding.key).into());
                    table.insert("multiplier", (binding.multiplier as f64).into());
                    table.insert("axis_dir", input_axis_dir_to_str(binding.axis_dir).into());
                    table.insert("axis", input_axis_to_str(binding.axis).into());
                    table.insert(
                        "button_behavior",
                        button_behavior_to_str(binding.behavior).into(),
                    );
                    bindings_list.push(table);
                }
                bindings_list.fmt();
                doc_bindings["keyboard"] = toml_edit::value(bindings_list);
            }
            if let Some(mouse) = mouse {
                if let Some(motion) = mouse.movement {
                    let mut table = toml_edit::InlineTable::new();
                    table.insert("sensitivity", (motion.multiplier as f64).into());
                    doc_bindings
                        .as_table_mut()
                        .unwrap()
                        .insert("mouse_movement", value(table));
                }
                if !mouse.buttons.is_empty() {
                    let mut bindings_list = toml_edit::Array::new();
                    for binding in mouse.buttons.iter() {
                        let mut table = toml_edit::InlineTable::new();
                        table.insert("button", (&*mouse_button_to_cow_str(binding.button)).into());
                        table.insert("axis_dir", input_axis_dir_to_str(binding.axis_dir).into());
                        table.insert("axis", input_axis_to_str(binding.axis).into());
                        table.insert(
                            "button_behavior",
                            button_behavior_to_str(binding.behavior).into(),
                        );
                        bindings_list.push(table);
                    }
                    bindings_list.fmt();
                    doc_bindings["mouse_button"] = toml_edit::value(bindings_list);
                }
            }
            if let Some(gamepad) = gamepad {
                let mut bindings_list = toml_edit::Array::new();
                for binding in gamepad.bindings.iter() {
                    let mut table = toml_edit::InlineTable::new();
                    table.insert(
                        "key",
                        (&*gamepad_binding_source_to_cow_str(binding.source)).into(),
                    );
                    table.insert("axis_dir", input_axis_dir_to_str(binding.axis_dir).into());
                    table.insert("axis", input_axis_to_str(binding.axis).into());
                    table.insert(
                        "button_behavior",
                        button_behavior_to_str(binding.button_behavior).into(),
                    );
                    bindings_list.push(table);
                }
                bindings_list.fmt();
                doc_bindings["gamepad"] = toml_edit::value(bindings_list);
            }
            if let Some(gamepad_haptics) = gamepad_haptics {
                let mut bindings_list = toml_edit::Array::new();
                for binding in gamepad_haptics.bindings.iter() {
                    let mut table = toml_edit::InlineTable::new();
                    table.insert("haptic_type", gamepad_haptics_type_to_str(*binding).into());
                    bindings_list.push(table);
                }
                bindings_list.fmt();
                doc_bindings["gamepad_haptics"] = toml_edit::value(bindings_list);
            }
        }
    }
    info!("\n{}", owned_doc.to_string());
    cmds.insert_resource(ConfigDocument(owned_doc));
}

fn deserialize_v1(
    doc: Res<ConfigDocument>,
    mut action_query: Query<(Entity, &ActionName)>,
    set_query: Query<(&ActionSetName, &ActionsInSet)>,
    mut cmds: Commands,
) {
    match doc.0.get("version") {
        // WHY?!
        Some(toml_edit::Item::Value(toml_edit::Value::Integer(i))) if *i.value() == 1 => {}
        v => {
            error!("invalid version in config file, not loading: {:?}", v);
            return;
        }
    }
    for (name, item) in doc.0.iter() {
        if name == "version" {
            continue;
        }

        let Some((_set_name, actions)) = set_query.iter().find(|(set_name, _)| set_name.0 == name)
        else {
            error!("unable to find actionset with name: {}", name);
            continue;
        };
        let Some(table) = item.as_table() else {
            error!("action set {} not a table", name);
            continue;
        };
        for (action_name, action_bindings) in table.iter() {
            let Some(bindings) = action_bindings.as_table() else {
                error!("action {} not a table", action_name);
                continue;
            };
            let Some(action_entity) = action_query
                .iter_many_mut(actions.0.iter())
                .find(|(_, n)| n.0 == action_name)
                .map(|(e, _)| e)
            else {
                error!("unable to find action with name: {}", action_name);
                continue;
            };
            let mut keyboard_bindings = KeyboardBindings::default();
            let mut mouse_bindings = MouseBindings::default();
            let mut gamepad_bindings = GamepadBindings::default();
            let mut gamepad_haptics_bindings = GamepadHapticOutputBindings::default();
            'keyboard: {
                if let Some(keyboard) = bindings.get("keyboard") {
                    let Some(keyboard) = keyboard.as_array() else {
                        error!("keyboard field on {name}.{action_name} is not an array");
                        break 'keyboard;
                    };
                    for binding_table in keyboard.iter() {
                        let Some(binding_table) = binding_table.as_inline_table() else {
                            error!("keyboard binding array doesn't contain inline tables");
                            continue;
                        };
                        let key = {
                            let Some(val) = str_from_table(binding_table, "key") else {
                                error!("cannot get string for {name}.{action_name}.keyboard.key");
                                continue;
                            };
                            let Some(w) = str_to_key_code(val) else {
                                error!("unable to parse {val} as keycode");
                                continue;
                            };
                            w
                        };
                        let axis_dir = {
                            let Some(val) = str_from_table(binding_table, "axis_dir") else {
                                error!(
                                    "cannot get string for {name}.{action_name}.keyboard.axis_dir"
                                );
                                continue;
                            };
                            let Some(w) = str_to_input_axis_dir(val) else {
                                error!("unable to parse {val} as axis direction");
                                continue;
                            };
                            w
                        };
                        let axis = {
                            let Some(val) = str_from_table(binding_table, "axis") else {
                                error!("cannot get string for {name}.{action_name}.keyboard.axis");
                                continue;
                            };
                            let Some(w) = str_to_input_axis(val) else {
                                error!("unable to parse {val} as axis");
                                continue;
                            };
                            w
                        };
                        let behavior = {
                            let Some(val) = str_from_table(binding_table, "button_behavior") else {
                                error!("cannot get string for {name}.{action_name}.keyboard.button_behavior");
                                continue;
                            };
                            let Some(w) = str_to_button_behavior(val) else {
                                error!("unable to parse {val} as button behavior");
                                continue;
                            };
                            w
                        };
                        let Some(multiplier) = f32_from_table(binding_table, "multiplier") else {
                            error!(
                                "cannot get number for {name}.{action_name}.keyboard.multiplier"
                            );
                            continue;
                        };
                        keyboard_bindings = keyboard_bindings.add_binding(KeyboardBinding {
                            key,
                            axis,
                            axis_dir,
                            behavior,
                            multiplier,
                        });
                    }
                }
                'mouse: {
                    if let Some(movement) = bindings.get("mouse_movement") {
                        let Some(binding_table) = movement.as_inline_table() else {
                            error!("mouse_movement field on {name}.{action_name} is not a table");
                            break 'mouse;
                        };
                        let Some(sensitivity) = f32_from_table(binding_table, "sensitivity") else {
                            error!(
                                "cannot get number for {name}.{action_name}.keyboard.sensitivity"
                            );
                            break 'mouse;
                        };
                        mouse_bindings.movement = Some(MouseMotionBinding {
                            motion_type: MouseMotionType::DeltaMotion,
                            multiplier: sensitivity,
                        });
                    }
                    if let Some(mouse_button) = bindings.get("mouse_button") {
                        let Some(mouse_button) = mouse_button.as_array() else {
                            error!("mouse_button field on {name}.{action_name} is not an array");
                            break 'mouse;
                        };
                        for binding_table in mouse_button.iter() {
                            let Some(binding_table) = binding_table.as_inline_table() else {
                                error!("mouse binding array doesn't contain inline tables");
                                continue;
                            };
                            let button = {
                                let Some(val) = str_from_table(binding_table, "button") else {
                                    error!("cannot get string for {name}.{action_name}.mouse_button.button");
                                    continue;
                                };
                                let Some(w) = str_to_mouse_button(val) else {
                                    error!("unable to parse {val} as mouse button");
                                    continue;
                                };
                                w
                            };
                            let axis_dir = {
                                let Some(val) = str_from_table(binding_table, "axis_dir") else {
                                    error!(
                                    "cannot get string for {name}.{action_name}.mouse_button.axis_dir"
                                );
                                    continue;
                                };
                                let Some(w) = str_to_input_axis_dir(val) else {
                                    error!("unable to parse {val} as axis direction");
                                    continue;
                                };
                                w
                            };
                            let axis = {
                                let Some(val) = str_from_table(binding_table, "axis") else {
                                    error!("cannot get string for {name}.{action_name}.mouse_button.axis");

                                    continue;
                                };
                                let Some(w) = str_to_input_axis(val) else {
                                    error!("unable to parse {val} as axis");
                                    continue;
                                };
                                w
                            };
                            let behavior = {
                                let Some(val) = str_from_table(binding_table, "button_behavior")
                                else {
                                    error!("cannot get string for {name}.{action_name}.mouse_button.button_behavior");
                                    continue;
                                };
                                let Some(w) = str_to_button_behavior(val) else {
                                    error!("unable to parse {val} as button behavior");
                                    continue;
                                };
                                w
                            };
                            mouse_bindings = mouse_bindings.add_binding(MouseButtonBinding {
                                button,
                                axis,
                                axis_dir,
                                behavior,
                            });
                        }
                    }
                }
            }

            'gamepad: {
                if let Some(gamepad) = bindings.get("gamepad") {
                    let Some(gamepad) = gamepad.as_array() else {
                        error!("gamepad field on {name}.{action_name} is not an array");
                        break 'gamepad;
                    };
                    for binding_table in gamepad.iter() {
                        let Some(binding_table) = binding_table.as_inline_table() else {
                            error!("gamepad binding array doesn't contain inline tables");
                            continue;
                        };
                        let source = {
                            let Some(val) = str_from_table(binding_table, "key") else {
                                error!("cannot get string for {name}.{action_name}.gamepad.key");
                                continue;
                            };
                            let Some(w) = str_to_gamepad_binding_source(val) else {
                                error!("unable to parse {val} as gamepad binding source");
                                continue;
                            };
                            w
                        };
                        let axis_dir = {
                            let Some(val) = str_from_table(binding_table, "axis_dir") else {
                                error!(
                                    "cannot get string for {name}.{action_name}.gamepad.axis_dir"
                                );
                                continue;
                            };
                            let Some(w) = str_to_input_axis_dir(val) else {
                                error!("unable to parse {val} as axis direction");
                                continue;
                            };
                            w
                        };
                        let axis = {
                            let Some(val) = str_from_table(binding_table, "axis") else {
                                error!("cannot get string for {name}.{action_name}.gamepad.axis");
                                continue;
                            };
                            let Some(w) = str_to_input_axis(val) else {
                                error!("unable to parse {val} as axis");
                                continue;
                            };
                            w
                        };
                        let behavior = {
                            let Some(val) = str_from_table(binding_table, "button_behavior") else {
                                error!("cannot get string for {name}.{action_name}.gamepad.button_behavior");
                                continue;
                            };
                            let Some(w) = str_to_button_behavior(val) else {
                                error!("unable to parse {val} as button behavior");
                                continue;
                            };
                            w
                        };
                        gamepad_bindings = gamepad_bindings.add_binding(GamepadBinding {
                            source,
                            button_behavior: behavior,
                            axis,
                            axis_dir,
                        });
                    }
                }
            }
            'gamepad_haptics: {
                if let Some(gamepad_haptics) = bindings.get("gamepad_haptics") {
                    let Some(gamepad_haptics) = gamepad_haptics.as_array() else {
                        error!("gamepad_haptics field on {name}.{action_name} is not an array");
                        break 'gamepad_haptics;
                    };
                    for binding_table in gamepad_haptics.iter() {
                        let Some(binding_table) = binding_table.as_inline_table() else {
                            error!("gamepad binding array doesn't contain inline tables");
                            continue;
                        };
                        let haptic_type = {
                            let Some(val) = str_from_table(binding_table, "haptic_type") else {
                                error!("cannot get string for {name}.{action_name}.gamepad_haptics.haptic_type");
                                continue;
                            };
                            let Some(w) = str_to_gamepad_haptics_type(val) else {
                                error!("unable to parse {val} as gamepad haptic type");
                                continue;
                            };
                            w
                        };
                        gamepad_haptics_bindings.bindings.push(haptic_type);
                    }
                }
            }
            cmds.entity(action_entity).insert((
                keyboard_bindings,
                mouse_bindings,
                gamepad_bindings,
                gamepad_haptics_bindings,
            ));
        }
    }
}

fn str_from_table<'a>(table: &'a dyn TableLike, key: &str) -> Option<&'a str> {
    match table.get(key) {
        Some(Item::Value(Value::String(v))) => Some(v.value()),
        _ => None,
    }
}
fn f32_from_table(table: &dyn TableLike, key: &str) -> Option<f32> {
    match table.get(key) {
        Some(Item::Value(Value::Float(v))) => Some(*v.value() as f32),
        Some(Item::Value(Value::Integer(v))) => Some(*v.value() as f32),
        _ => None,
    }
}

// #[derive(Reflect)]
// struct Test {
//     hi: KeyCode,
// }
//
// #[derive(Reflect)]
// struct ConfigActionSet(HashMap<String, ConfigAction>);
// #[derive(Reflect)]
// struct ConfigAction(Vec<ConfigBinding>);
//
// #[derive(Reflect)]
// enum ConfigBinding {
//     Keyboard {
//         key: KeyCode,
//         multiplier: f32,
//         dir: InputAxisDirection,
//         axis: InputAxis,
//     },
//     MouseMotion {
//         sensitivity: f32,
//     },
//     Mouse {
//         button: MouseButton,
//         multiplier: f32,
//         dir: InputAxisDirection,
//         axis: InputAxis,
//     },
// }
//
// fn build_config_v1() {
//     let mut table = Table::new();
//     table.insert("version".to_string(), toml::Value::Integer(1));
//
//     table.insert(
//         "core".to_string(),
//         toml::Value::Table({
//             let mut table = Table::new();
//             table.insert(
//                 "move".to_string(),
//                 toml::Value::Table({
//                     let mut table = Table::new();
//                     table.insert(
//                         "keyboard_bindings".to_string(),
//                         toml::Value::Array(vec![toml::Value::Table({
//                             let mut table = Table::new();
//                             table
//                                 .insert("key".to_string(), toml::Value::String("KeyW".to_string()));
//
//                             table
//                         })]),
//                     );
//                     table
//                 }),
//             );
//             table
//         }),
//     );
// }
