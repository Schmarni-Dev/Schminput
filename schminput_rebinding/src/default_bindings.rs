use bevy::prelude::*;
use schminput::prelude::*;

#[derive(SystemSet, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum DefaultBindingsSet {
    CopyDefaultBindngs,
    LoadCustomBindings,
}

#[derive(Event, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum ResetToDefautlBindings {
    All,
    Action(Entity),
}

pub struct RebindingDefaultBindingsPlugin;
impl Plugin for RebindingDefaultBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResetToDefautlBindings>();
        app.add_systems(
            PostStartup,
            copy_default_bindings.in_set(DefaultBindingsSet::CopyDefaultBindngs),
        );
        app.add_systems(PostUpdate, reset_bindings);
    }
}

fn reset_bindings(
    mut event: EventReader<ResetToDefautlBindings>,
    mut cmds: Commands,
    query: Query<(Entity, &DefaultBindings)>,
    #[cfg(not(feature = "xr"))] default_bindings_query: Query<(
        Option<&KeyboardBindings>,
        Option<&GamepadBindings>,
        Option<&MouseBindings>,
        Option<()>,
    )>,
    #[cfg(feature = "xr")] default_bindings_query: Query<(
        Option<&KeyboardBindings>,
        Option<&GamepadBindings>,
        Option<&MouseBindings>,
        Option<&OxrActionBlueprint>,
    )>,
) {
    for event in event.read().copied() {
        match event {
            ResetToDefautlBindings::All => {
                for (action, bindings) in &query {
                    #[cfg_attr(not(feature = "xr"), allow(unused_variables))]
                    let Ok((keyboard, gamepad, mouse, xr)) = default_bindings_query.get(bindings.0) else {
                        continue;
                    };
                    let mut w = cmds.entity(action);
                    if let Some(v) = keyboard {
                        w.insert(v.clone());
                    } else {
                        w.remove::<KeyboardBindings>();
                    }
                    if let Some(v) = gamepad {
                        w.insert(v.clone());
                    } else {
                        w.remove::<GamepadBindings>();
                    }

                    if let Some(v) = mouse {
                        w.insert(v.clone());
                    } else {
                        w.remove::<MouseBindings>();
                    }

                    #[cfg(feature = "xr")]
                    if let Some(v) = xr {
                        w.insert(v.clone());
                    } else {
                        w.remove::<OxrActionBlueprint>();
                    }
                }
            }

            ResetToDefautlBindings::Action(action) => {
                let Ok((action, bindings)) = query.get(action) else {
                    continue;
                };
                #[cfg_attr(not(feature = "xr"), allow(unused_variables))]
                let Ok((keyboard, gamepad, mouse, xr)) = default_bindings_query.get(bindings.0) else {
                    continue;
                };
                let mut w = cmds.entity(action);
                if let Some(v) = keyboard {
                    w.insert(v.clone());
                } else {
                    w.remove::<KeyboardBindings>();
                }
                if let Some(v) = gamepad {
                    w.insert(v.clone());
                } else {
                    w.remove::<GamepadBindings>();
                }

                if let Some(v) = mouse {
                    w.insert(v.clone());
                } else {
                    w.remove::<MouseBindings>();
                }

                #[cfg(feature = "xr")]
                if let Some(v) = xr {
                    w.insert(v.clone());
                } else {
                    w.remove::<OxrActionBlueprint>();
                }
            }
        }
    }
}

fn copy_default_bindings(
    #[cfg(not(feature = "xr"))] query: Query<(
        Entity,
        Option<&KeyboardBindings>,
        Option<&GamepadBindings>,
        Option<&MouseBindings>,
        Option<()>,
    )>,
    #[cfg(feature = "xr")] query: Query<(
        Entity,
        Option<&KeyboardBindings>,
        Option<&GamepadBindings>,
        Option<&MouseBindings>,
        Option<&OxrActionBlueprint>,
    )>,
    mut cmds: Commands,
) {
    #[cfg_attr(not(feature = "xr"), allow(unused_variables))]
    for (action, keyboard, gamepad, mouse, xr) in &query {
        let mut w = cmds.spawn_empty();
        if let Some(v) = keyboard {
            w.insert(v.clone());
        }
        if let Some(v) = gamepad {
            w.insert(v.clone());
        }
        if let Some(v) = mouse {
            w.insert(v.clone());
        }
        #[cfg(feature = "xr")]
        if let Some(v) = xr {
            #[allow(clippy::unit_arg, clippy::clone_on_copy)]
            w.insert(v.clone());
        }
        let w = w.id();
        cmds.entity(action).insert(DefaultBindings(w));
    }
}

#[derive(Clone, Copy, Component)]
struct DefaultBindings(Entity);
