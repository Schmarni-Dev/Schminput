// Register extra fn on action trait with base action pass empty block for impl macro
use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::ActionTrait;

use super::MouseBindings;

pub struct MouseMotionBindingProvider;
impl Plugin for MouseMotionBindingProvider {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, sync_mouse_moitions);
    }
}

fn sync_mouse_moitions(
    mut actions: Query<&mut dyn MouseMotionAction>,
    bindings: Res<MouseBindings>,
    mut mouse_events: EventReader<MouseMotion>,
) {
    let mut delta = Vec2::ZERO;
    for ev in mouse_events.read() {
        delta += ev.delta;
    }
    for mut action in &mut actions.iter_mut().flatten() {
        if bindings
            .motion_bindings
            .contains(&(action.action_key(), action.action_set_key()))
        {
            let mut v = delta.yx() + action.get_value();
            v.x *= -action.mouse_sens_x();
            v.y *= action.mouse_sens_y();
            action.set_value(v);
        }
    }
}

#[bevy_trait_query::queryable]
pub trait MouseMotionAction: ActionTrait<T = Vec2> {
    fn mouse_sens_x(&self) -> f32;
    fn mouse_sens_y(&self) -> f32;
}

#[macro_export]
/// new_action!(
/// |   action_ident: ident,
/// |   action_key: &'static str,
/// |   action_name: expression -> String,
/// |   action_set_key: expression -> &'static str,
/// |   action_set_name: expression -> String
/// )
macro_rules! mouse_action {
    ($ident:ident, $key:literal, $name:expr, $set_key:expr, $set_name:expr) => {
        #[derive(bevy::prelude::Component)]
        pub struct $ident {
            data: bevy::prelude::Vec2,
            previous_data: bevy::prelude::Vec2,
            mouse_sens_x: f32,
            mouse_sens_y: f32,
        }

        impl Default for $ident {
            fn default() -> Self {
                Self {
                    mouse_sens_x: 1.0,
                    mouse_sens_y: 1.0,
                    data: bevy::prelude::Vec2::ZERO,
                    previous_data: bevy::prelude::Vec2::ZERO,
                }
            }
        }

        bevy_schminput::gen_action_trait_impl!(
            $ident,
            bevy::prelude::Vec2,
            $key,
            $name,
            $set_key,
            $set_name
        );
        bevy_schminput::gen_mouse_action_trait_impl!($ident);
        impl bevy_schminput::ActionExtensionTrait for $ident {
            fn register_other_trait(app: &mut App)
            where
                Self: Sized,
            {
                use bevy_schminput::prelude::RegisterExt;
                app.register_component_as::<dyn bevy_schminput::mouse::motion::MouseMotionAction, Self>();
            }
        }
    };
}

#[macro_export]
/// new_action!(action_ident: ident)
macro_rules! gen_mouse_action_trait_impl {
    ($ident:ident) => {
        impl bevy_schminput::mouse::motion::MouseMotionAction for $ident {
            fn mouse_sens_x(&self) -> f32 {
                self.mouse_sens_x
            }

            fn mouse_sens_y(&self) -> f32 {
                self.mouse_sens_y
            }
        }
    };
}
