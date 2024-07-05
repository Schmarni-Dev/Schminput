pub mod gamepad;
pub mod keyboard;
pub mod mouse;
#[cfg(feature = "xr")]
pub mod openxr;
pub mod prelude;

use std::{borrow::Cow, hash::Hash};

use bevy::{app::PluginGroupBuilder, ecs::system::EntityCommands, prelude::*};

#[derive(SystemSet, Clone, Copy, Debug, Reflect, Hash, PartialEq, Eq)]
pub enum SchminputSet {
    ClearValues,
    SyncInputActions,
    SyncOutputActions,
}

pub struct SchminputPlugin;

impl Plugin for SchminputPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PreUpdate,
            (SchminputSet::ClearValues, SchminputSet::SyncInputActions).chain(),
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
        val.0 = false;
    }
}
fn clean_f32(mut query: Query<&mut F32ActionValue>) {
    for mut val in &mut query {
        val.0 = 0.0;
    }
}
fn clean_vec2(mut query: Query<&mut Vec2ActionValue>) {
    for mut val in &mut query {
        val.0 = Vec2::ZERO;
    }
}

pub struct DefaultSchmugins;

impl PluginGroup for DefaultSchmugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let p = PluginGroupBuilder::start::<DefaultSchmugins>()
            .add(SchminputPlugin)
            .add(keyboard::KeyboardPlugin)
            .add(mouse::MousePlugin)
            .add(gamepad::GamepadPlugin);
        #[cfg(feature = "xr")]
        return p.add(openxr::OxrInputPlugin);
        #[cfg(not(feature = "xr"))]
        return p;
    }
}

// TODO: figure out a nice way of doing subaction paths, preferably across input methods

/// The ActionSet This Action belongs to.
#[derive(Debug, Clone, Copy, Component, Reflect, Deref)]
pub struct ActionSet(pub Entity);

/// The Display name of the Action Set.
#[derive(Debug, Clone, Component, Reflect, Deref)]
pub struct LocalizedActionSetName(pub Cow<'static, str>);

/// This needs to be a unique identifier that describes the Action Set.
#[derive(Debug, Clone, Component, Reflect, Deref)]
pub struct ActionSetName(pub Cow<'static, str>);

/// The Display name of the Action.
#[derive(Debug, Clone, Component, Reflect, Deref)]
pub struct LocalizedActionName(pub Cow<'static, str>);

/// This needs to be a unique identifier that describes the action.
/// If using an ActionSet this only needs to be unique in that Set.
#[derive(Debug, Clone, Component, Reflect, Deref)]
pub struct ActionName(pub Cow<'static, str>);

/// +X: Right, +Y: Up
#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
pub struct Vec2ActionValue(pub Vec2);

impl Vec2ActionValue {
    const ZERO: Self = Vec2ActionValue(Vec2::ZERO);
}

impl Default for Vec2ActionValue {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
pub struct F32ActionValue(pub f32);

impl F32ActionValue {
    const ZERO: Self = F32ActionValue(0f32);
}

impl Default for F32ActionValue {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Default, Deref, DerefMut)]
pub struct BoolActionValue(pub bool);

// there might be a better name for this
/// +X = Right, +Y = Up
#[derive(Clone, Copy, Debug, Reflect, Default, PartialEq, Eq, Hash)]
pub enum InputAxis {
    X,
    #[default]
    Y,
}

impl InputAxis {
    pub fn vec_axis(&self, vec: Vec2) -> f32 {
        match self {
            InputAxis::X => vec.x,
            InputAxis::Y => vec.y,
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

impl InputAxisDirection {
    #[inline(always)]
    pub fn as_multipier(&self) -> f32 {
        match self {
            InputAxisDirection::Positive => 1f32,
            InputAxisDirection::Negative => -1f32,
        }
    }
}

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

pub struct ActionSetHeaderBuilder {
    id: ActionSetName,
    name: Option<LocalizedActionSetName>,
}

impl ActionSetHeaderBuilder {
    pub fn new(id: impl Into<Cow<'static, str>>) -> ActionSetHeaderBuilder {
        ActionSetHeaderBuilder {
            id: ActionSetName(id.into()),
            name: None,
        }
    }
    pub fn with_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = Some(LocalizedActionSetName(name.into()));
        self
    }
    pub fn build<'a>(self, cmds: &'a mut Commands) -> EntityCommands<'a> {
        let mut e_cmds = cmds.spawn(self.id);
        if let Some(name) = self.name {
            e_cmds.insert(name);
        }

        e_cmds
    }
}

pub struct ActionHeaderBuilder {
    id: ActionName,
    name: Option<LocalizedActionName>,
    set: Option<ActionSet>,
}

impl ActionHeaderBuilder {
    pub fn new(id: impl Into<Cow<'static, str>>) -> ActionHeaderBuilder {
        ActionHeaderBuilder {
            id: ActionName(id.into()),
            name: None,
            set: None,
        }
    }
    pub fn with_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = Some(LocalizedActionName(name.into()));
        self
    }
    pub fn with_set(mut self, set: Entity) -> Self {
        self.set = Some(ActionSet(set));
        self
    }
    pub fn build<'a>(self, cmds: &'a mut Commands) -> EntityCommands<'a> {
        let mut e_cmds = cmds.spawn(self.id);
        if let Some(name) = self.name {
            e_cmds.insert(name);
        }
        if let Some(set) = self.set {
            e_cmds.insert(set);
        }

        e_cmds
    }
}
