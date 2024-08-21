pub mod gamepad;
pub mod keyboard;
pub mod mouse;
#[cfg(feature = "xr")]
pub mod openxr;
pub mod prelude;
pub mod subaction_paths;

use std::{borrow::Cow, hash::Hash, mem};

use bevy::{app::PluginGroupBuilder, prelude::*};
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
        let p = PluginGroupBuilder::start::<DefaultSchminputPlugins>()
            .add(SchminputPlugin)
            .add(SubactionPathPlugin)
            .add(keyboard::KeyboardPlugin)
            .add(mouse::MousePlugin)
            .add(gamepad::GamepadPlugin);
        #[cfg(feature = "xr")]
        return p.add(openxr::OxrInputPlugin);
        #[cfg(not(feature = "xr"))]
        return p;
    }
}

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
#[derive(Debug, Clone, Component, Reflect, Deref, DerefMut, Default)]
pub struct Vec2ActionValue(pub SubactionPathMap<Vec2>);

#[derive(Debug, Clone, Component, Reflect, Deref, DerefMut, Default)]
pub struct F32ActionValue(pub SubactionPathMap<f32>);

#[derive(Debug, Clone, Component, Reflect, Deref, DerefMut, Default)]
pub struct BoolActionValue(pub SubactionPathMap<bool>);

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

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
pub struct ActionSetEnabled(pub bool);

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

#[derive(Bundle, Clone, Debug)]
pub struct ActionSetBundle {
    pub id: ActionSetName,
    pub name: LocalizedActionSetName,
    pub enabled: ActionSetEnabled,
}

impl ActionSetBundle {
    pub fn new(
        id: impl Into<Cow<'static, str>>,
        name: impl Into<Cow<'static, str>>,
    ) -> ActionSetBundle {
        ActionSetBundle {
            id: ActionSetName(id.into()),
            name: LocalizedActionSetName(name.into()),
            enabled: ActionSetEnabled(true),
        }
    }
}

#[derive(Bundle, Clone, Debug)]
pub struct ActionBundle {
    pub id: ActionName,
    pub name: LocalizedActionName,
    pub set: ActionSet,
    pub paths: RequestedSubactionPaths,
}

impl ActionBundle {
    pub fn new(
        id: impl Into<Cow<'static, str>>,
        name: impl Into<Cow<'static, str>>,
        set: Entity,
    ) -> ActionBundle {
        ActionBundle {
            id: ActionName(id.into()),
            name: LocalizedActionName(name.into()),
            set: ActionSet(set),
            paths: RequestedSubactionPaths::default(),
        }
    }
}
