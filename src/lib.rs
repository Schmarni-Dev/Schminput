pub mod binding_modification;
pub mod gamepad;
pub mod impl_helpers;
pub mod keyboard;
pub mod mouse;
#[cfg(feature = "xr")]
pub mod openxr;
pub mod prelude;
pub mod subaction_paths;
#[cfg(feature = "xr")]
pub mod xr;

use std::{borrow::Cow, fmt::Display, hash::Hash, mem};

use bevy::{
    app::PluginGroupBuilder,
    ecs::{component::ComponentId, entity::EntityHashSet, world::DeferredWorld},
    prelude::*,
};
use binding_modification::BindingModifications;
use subaction_paths::{RequestedSubactionPaths, SubactionPathMap, SubactionPathPlugin};

#[derive(SystemSet, Clone, Copy, Debug, Reflect, Hash, PartialEq, Eq)]
pub enum SchminputSet {
    HandleNewSubactionPaths,
    ClearValues,
    SyncInputActions,
    SyncOutputActions,
}

pub struct SchminputPlugin;

impl Plugin for SchminputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InputAxis>();
        app.register_type::<InputAxisDirection>();
        app.configure_sets(
            PreUpdate,
            (
                SchminputSet::HandleNewSubactionPaths,
                SchminputSet::ClearValues,
                SchminputSet::SyncInputActions,
            )
                .chain(),
        );
        // Probably not needed, but for reference,
        app.configure_sets(PostUpdate, SchminputSet::SyncOutputActions);

        app.add_systems(PreUpdate, clean_bool.in_set(SchminputSet::ClearValues));
        app.add_systems(PreUpdate, clean_f32.in_set(SchminputSet::ClearValues));
        app.add_systems(PreUpdate, clean_vec2.in_set(SchminputSet::ClearValues));
    }
}

fn clean_bool(mut query: Query<&mut BoolActionValue>) {
    for mut val in &mut query {
        let _last = mem::take(val.as_mut());
    }
}
fn clean_f32(mut query: Query<&mut F32ActionValue>) {
    for mut val in &mut query {
        let _last = mem::take(val.as_mut());
    }
}
fn clean_vec2(mut query: Query<&mut Vec2ActionValue>) {
    for mut val in &mut query {
        let _last = mem::take(val.as_mut());
    }
}

pub struct DefaultSchminputPlugins;

impl PluginGroup for DefaultSchminputPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let g = PluginGroupBuilder::start::<DefaultSchminputPlugins>()
            .add(SchminputPlugin)
            .add(SubactionPathPlugin)
            .add(keyboard::KeyboardPlugin)
            .add(mouse::MousePlugin)
            .add(gamepad::GamepadPlugin);
        #[cfg(feature = "xr")]
        let g = g.add(xr::GenericXrInputPlugin);
        #[cfg(all(feature = "xr", not(target_family = "wasm")))]
        let g = g.add(openxr::OxrInputPlugin);
        g
    }
}

#[derive(Debug, Clone, Reflect, Component)]
#[component(on_add = on_action_add)]
#[component(on_remove = on_action_remove)]
#[require(RequestedSubactionPaths, BindingModifications)]
pub struct Action {
    pub set: Entity,
    pub localized_name: Cow<'static, str>,
    pub name: Cow<'static, str>,
}

fn on_action_add(mut world: DeferredWorld, entity: Entity, _component_id: ComponentId) {
    let Some(set_entity) = world.entity(entity).get::<Action>().map(|a| a.set) else {
        error!("action not on entity, this should be unreachable!");
        return;
    };
    let mut tmp_entity = world.entity_mut(set_entity);
    let Some(mut set) = tmp_entity.get_mut::<ActionsInSet>() else {
        error!("invalid action set {set_entity:?}");
        return;
    };
    set.0.insert(entity);
}

fn on_action_remove(mut world: DeferredWorld, entity: Entity, _component_id: ComponentId) {
    let Some(set_entity) = world.entity(entity).get::<Action>().map(|a| a.set) else {
        error!("action not on entity, this should be unreachable!");
        return;
    };
    let mut tmp_entity = world.entity_mut(set_entity);
    let Some(mut set) = tmp_entity.get_mut::<ActionsInSet>() else {
        error!("invalid action set {set_entity:?}");
        return;
    };
    set.0.remove(&entity);
}

impl Action {
    pub fn new(
        id: impl Into<Cow<'static, str>>,
        name: impl Into<Cow<'static, str>>,
        set: Entity,
    ) -> Action {
        Action {
            name: id.into(),
            localized_name: name.into(),
            set,
        }
    }
}

#[derive(Debug, Clone, Reflect, Component)]
#[component(on_remove = on_action_remove)]
#[require(ActionsInSet)]
pub struct ActionSet {
    pub name: Cow<'static, str>,
    pub localized_name: Cow<'static, str>,
    pub enabled: bool,
    pub priority: u32,
    pub blocks_input: bool,
}

impl ActionSet {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        localized_name: impl Into<Cow<'static, str>>,
        priority: u32,
    ) -> ActionSet {
        ActionSet {
            name: name.into(),
            localized_name: localized_name.into(),
            enabled: true,
            priority,
            blocks_input: true,
        }
    }
    pub fn dont_block_input(mut self) -> Self {
        self.blocks_input = false;
        self
    }
}

#[derive(Debug, Clone, Component, Reflect, Deref, Default)]
pub struct ActionsInSet(pub EntityHashSet);

/// +X: Right, +Y: Up
#[derive(Debug, Clone, Component, Reflect, Deref, DerefMut, Default)]
pub struct Vec2ActionValue(pub SubactionPathMap<Vec2>);

#[derive(Debug, Clone, Component, Reflect, Deref, DerefMut, Default)]
pub struct F32ActionValue(pub SubactionPathMap<f32>);

#[derive(Debug, Clone, Component, Reflect, Deref, DerefMut, Default)]
pub struct BoolActionValue(pub SubactionPathMap<bool>);

impl Vec2ActionValue {
    pub fn new() -> Self {
        Self::default()
    }
}
impl F32ActionValue {
    pub fn new() -> Self {
        Self::default()
    }
}
impl BoolActionValue {
    pub fn new() -> Self {
        Self::default()
    }
}

// there might be a better name for this
/// +X = Right, +Y = Up
#[derive(Clone, Copy, Debug, Reflect, Default, PartialEq, Eq, Hash)]
pub enum InputAxis {
    X,
    #[default]
    Y,
}

impl Display for InputAxis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputAxis::X => f.write_str("X Axis"),
            InputAxis::Y => f.write_str("Y Axis"),
        }
    }
}

impl InputAxis {
    pub fn vec_axis(&self, vec: Vec2) -> f32 {
        match self {
            InputAxis::X => vec.x,
            InputAxis::Y => vec.y,
        }
    }
    pub fn new_vec(&self, value: f32) -> Vec2 {
        match self {
            InputAxis::X => Vec2::new(value, 0.0),
            InputAxis::Y => Vec2::new(0.0, value),
        }
    }
    pub fn vec_axis_mut<'a>(&self, vec: &'a mut Vec2) -> &'a mut f32 {
        match self {
            InputAxis::X => &mut vec.x,
            InputAxis::Y => &mut vec.y,
        }
    }
}

// there might be a better name for this
#[derive(Clone, Copy, Debug, Reflect, Default, PartialEq, Eq, Hash)]
pub enum InputAxisDirection {
    #[default]
    Positive,
    Negative,
}

impl Display for InputAxisDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputAxisDirection::Positive => f.write_str("+"),
            InputAxisDirection::Negative => f.write_str("-"),
        }
    }
}

impl InputAxisDirection {
    #[inline(always)]
    pub fn as_multipier(&self) -> f32 {
        match self {
            InputAxisDirection::Positive => 1f32,
            InputAxisDirection::Negative => -1f32,
        }
    }
}

// TODO: add released?
#[derive(Clone, Copy, Debug, Reflect, Default, PartialEq, Eq, Hash)]
pub enum ButtonInputBeheavior {
    JustPressed,
    #[default]
    Pressed,
    JustReleased,
}

impl ButtonInputBeheavior {
    pub fn apply<T: Copy + Eq + Hash + Send + Sync>(
        &self,
        input: &ButtonInput<T>,
        value: T,
    ) -> bool {
        match self {
            ButtonInputBeheavior::JustPressed => input.just_pressed(value),
            ButtonInputBeheavior::Pressed => input.pressed(value),
            ButtonInputBeheavior::JustReleased => input.just_released(value),
        }
    }
}
impl Display for ButtonInputBeheavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ButtonInputBeheavior::JustPressed => "On Press",
            ButtonInputBeheavior::Pressed => "Pressed",
            ButtonInputBeheavior::JustReleased => "On Release",
        })
    }
}
