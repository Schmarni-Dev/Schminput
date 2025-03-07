use bevy::prelude::*;
use schminput::{prelude::*, ActionsInSet};
use toml_edit::{value, DocumentMut, Item, TableLike, Value};

use crate::str_converstions::*;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PersistentBindingsSet {
    Serialize,
    Deserialize,
}

pub struct PersistentBindingsPlugin;

impl Plugin for PersistentBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DeserializeSchminputConfig>();
        app.add_event::<SerializeSchminputConfig>();
        app.add_event::<FinnishedSchminputConfigSerialization>();
        app.add_event::<FinnishedSchminputConfigDeserialization>();
        app.add_systems(
            PostUpdate,
            serialize_v1
                .run_if(on_event::<SerializeSchminputConfig>)
                .in_set(PersistentBindingsSet::Serialize),
        );
        app.add_systems(
            PostUpdate,
            deserialize_v1
                .run_if(on_event::<DeserializeSchminputConfig>)
                .in_set(PersistentBindingsSet::Deserialize),
        );
    }
}

fn implicit_table() -> toml_edit::Item {
    let mut w = toml_edit::Table::new();
    w.set_implicit(true);
    toml_edit::Item::Table(w)
}

#[derive(Event, Clone)]
pub struct DeserializeSchminputConfig {
    pub config: String,
}
#[derive(Event, Clone)]
pub struct FinnishedSchminputConfigDeserialization;

#[derive(Event, Clone)]
pub struct SerializeSchminputConfig {
    pub base_config: String,
}
#[derive(Event, Clone)]
pub struct FinnishedSchminputConfigSerialization {
    pub output: String,
}

#[cfg(feature = "xr")]
type XrBindings<'a> = &'a OxrBindings;
#[cfg(not(feature = "xr"))]
type XrBindings = ();

fn serialize_v1(
    mut request: EventReader<SerializeSchminputConfig>,
    mut respone: EventWriter<FinnishedSchminputConfigSerialization>,
    mut action_query: Query<(
        Option<&KeyboardBindings>,
        Option<&MouseBindings>,
        Option<&GamepadBindings>,
        Option<&GamepadHapticOutputBindings>,
        Option<XrBindings>,
        &Action,
    )>,
    set_query: Query<(&ActionSet, &ActionsInSet)>,
) {
    for request in request.read() {
        let mut owned_doc = match request.base_config.parse::<DocumentMut>() {
            Ok(v) => v,
            Err(err) => {
                error!("unable to parse base config toml: {}", err);
                continue;
            }
        };
        let doc = &mut owned_doc;
        doc.entry("version").or_insert(toml_edit::value(1i64));
        for (action_set, actions) in &set_query {
            let mut iter = action_query.iter_many_mut(actions.0.iter());
            #[cfg_attr(not(feature = "xr"), allow(unused_variables))]
            while let Some((keyboard, mouse, gamepad, gamepad_haptics, openxr, action)) =
                iter.fetch_next()
            {
                let doc_bindings = doc
                    .entry(&action_set.name)
                    .or_insert(implicit_table())
                    .as_table_mut()
                    .unwrap()
                    .entry(&action.name)
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
                            table.insert(
                                "button",
                                (&*mouse_button_to_cow_str(binding.button)).into(),
                            );
                            table
                                .insert("axis_dir", input_axis_dir_to_str(binding.axis_dir).into());
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
                #[cfg(feature = "xr")]
                if let Some(openxr) = openxr {
                    let mut table = toml_edit::Table::new();
                    for (interaction_profile, bindings) in openxr.bindings.iter() {
                        let mut bindings_list = toml_edit::Array::new();
                        for binding in bindings {
                            bindings_list.push(binding.to_string());
                        }
                        bindings_list.fmt();
                        table.insert(interaction_profile, value(bindings_list));
                    }
                    doc_bindings["openxr"] = toml_edit::Item::Table(table);
                }
            }
        }
        respone.send(FinnishedSchminputConfigSerialization {
            output: owned_doc.to_string(),
        });
    }
}

fn deserialize_v1(
    mut request: EventReader<DeserializeSchminputConfig>,
    mut respone: EventWriter<FinnishedSchminputConfigDeserialization>,
    mut action_query: Query<(Entity, &Action)>,
    set_query: Query<(&ActionSet, &ActionsInSet)>,
    mut cmds: Commands,
) {
    for request in request.read() {
        let doc = match request.config.parse::<DocumentMut>() {
            Ok(v) => v,
            Err(err) => {
                error!("unable to parse base config toml: {}", err);
                continue;
            }
        };
        match doc.get("version") {
            // WHY?!
            Some(toml_edit::Item::Value(toml_edit::Value::Integer(i))) if *i.value() == 1 => {}
            v => {
                error!("invalid version in config file, not loading: {:?}", v);
                return;
            }
        }
        for (name, item) in doc.iter() {
            if name == "version" {
                continue;
            }

            let Some((_set_name, actions)) =
                set_query.iter().find(|(action_set, _)| action_set.name == name)
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
                    .find(|(_, action)| action.name == action_name)
                    .map(|(e, _)| e)
                else {
                    error!("unable to find action with name: {}", action_name);
                    continue;
                };
                let mut keyboard_bindings = KeyboardBindings::new();
                let mut mouse_bindings = MouseBindings::new();
                let mut gamepad_bindings = GamepadBindings::new();
                let mut gamepad_haptics_bindings = GamepadHapticOutputBindings::new();
                #[cfg_attr(not(feature = "xr"), allow(unused_variables), allow(unused_mut))]
                let mut xr_bindings;
                #[cfg(feature = "xr")]
                {
                    xr_bindings = OxrBindings::new();
                }
                #[allow(unused_assignments)]
                #[cfg(not(feature = "xr"))]
                {
                    xr_bindings = ();
                }

                keyboard_bindings = parse_keyboard(bindings, name, action_name, keyboard_bindings);
                mouse_bindings = parse_mouse(bindings, name, action_name, mouse_bindings);

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
                                    error!(
                                        "cannot get string for {name}.{action_name}.gamepad.key"
                                    );
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
                                    error!(
                                        "cannot get string for {name}.{action_name}.gamepad.axis"
                                    );
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
                #[cfg(feature = "xr")]
                {
                    xr_bindings = parse_openxr(bindings, name, action_name, xr_bindings);
                }
                let mut e_cmds = cmds.entity(action_entity);
                e_cmds.insert((
                    keyboard_bindings,
                    mouse_bindings,
                    gamepad_bindings,
                    gamepad_haptics_bindings,
                ));
                #[cfg(feature = "xr")]
                {
                    e_cmds.insert(xr_bindings);
                }
            }
        }
        respone.send(FinnishedSchminputConfigDeserialization);
    }
}
#[cfg(feature = "xr")]
fn parse_openxr(
    bindings: &toml_edit::Table,
    name: &str,
    action_name: &str,
    mut xr_bindings: OxrBindings,
) -> OxrBindings {
    if let Some(openxr) = bindings.get("openxr") {
        let Some(openxr) = openxr.as_table() else {
            error!("{name}.{action_name}.openxr is not a table");
            return xr_bindings;
        };
        for (interaction_profile, bindings) in openxr.iter() {
            let Some(bindings) = bindings.as_array() else {
                error!("{name}.{action_name}.openxr.\"{interaction_profile}\" is not an array");
                continue;
            };
            let mut binding_builder =
                xr_bindings.interaction_profile(interaction_profile.to_string());
            for binding_value in bindings.iter() {
                let Some(binding) = binding_value.as_str() else {
                    error!(
                        "binding for {interaction_profile} on {name}.{action_name} is not a string"
                    );
                    continue;
                };
                binding_builder = binding_builder.binding(binding.to_string());
            }

            xr_bindings = binding_builder.end();
        }
    }
    xr_bindings
}

fn parse_mouse(
    bindings: &toml_edit::Table,
    name: &str,
    action_name: &str,
    mut mouse_bindings: MouseBindings,
) -> MouseBindings {
    if let Some(movement) = bindings.get("mouse_movement") {
        let Some(binding_table) = movement.as_inline_table() else {
            error!("mouse_movement field on {name}.{action_name} is not a table");
            return mouse_bindings;
        };
        let Some(sensitivity) = f32_from_table(binding_table, "sensitivity") else {
            error!("cannot get number for {name}.{action_name}.keyboard.sensitivity");
            return mouse_bindings;
        };
        mouse_bindings.movement = Some(MouseMotionBinding {
            motion_type: MouseMotionType::DeltaMotion,
            multiplier: sensitivity,
        });
    }
    if let Some(mouse_button) = bindings.get("mouse_button") {
        let Some(mouse_button) = mouse_button.as_array() else {
            error!("mouse_button field on {name}.{action_name} is not an array");
            return mouse_bindings;
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
                    error!("cannot get string for {name}.{action_name}.mouse_button.axis_dir");
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
                let Some(val) = str_from_table(binding_table, "button_behavior") else {
                    error!(
                        "cannot get string for {name}.{action_name}.mouse_button.button_behavior"
                    );
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
    mouse_bindings
}
fn parse_keyboard(
    bindings: &toml_edit::Table,
    set_name: &str,
    action_name: &str,
    mut keyboard_bindings: KeyboardBindings,
) -> KeyboardBindings {
    if let Some(keyboard) = bindings.get("keyboard") {
        let Some(keyboard) = keyboard.as_array() else {
            error!("keyboard field on {set_name}.{action_name} is not an array");
            return keyboard_bindings;
        };
        for binding_table in keyboard.iter() {
            let Some(binding_table) = binding_table.as_inline_table() else {
                error!("keyboard binding array doesn't contain inline tables");
                continue;
            };
            let key = {
                let Some(val) = str_from_table(binding_table, "key") else {
                    error!("cannot get string for {set_name}.{action_name}.keyboard.key");
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
                    error!("cannot get string for {set_name}.{action_name}.keyboard.axis_dir");
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
                    error!("cannot get string for {set_name}.{action_name}.keyboard.axis");
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
                    error!(
                        "cannot get string for {set_name}.{action_name}.keyboard.button_behavior"
                    );
                    continue;
                };
                let Some(w) = str_to_button_behavior(val) else {
                    error!("unable to parse {val} as button behavior");
                    continue;
                };
                w
            };
            let Some(multiplier) = f32_from_table(binding_table, "multiplier") else {
                error!("cannot get number for {set_name}.{action_name}.keyboard.multiplier");
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
    keyboard_bindings
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
