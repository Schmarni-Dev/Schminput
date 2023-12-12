pub mod keyboard_binding_provider;
pub mod mouse_binding_provider;

#[cfg(feature = "xr")]
pub mod oxr_binding_provider;

use bevy::prelude::*;
use bevy_trait_query::RegisterExt;

pub struct SchminputPlugin;

impl Plugin for SchminputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, reset_actions);
    }
}
fn reset_actions(mut input: Query<&mut dyn ErasedAction>) {
    input
        .par_iter_mut()
        .for_each(|mut e| e.iter_mut().for_each(|mut a| a.reset_value()));
}

#[macro_export]
/// new_action!(
/// |   action_ident: ident, 
/// |   action_type: Type,
/// |   action_key: &'static str, 
/// |   action_name: expression -> String,
/// |   action_set_key: expression -> &'static str, 
/// |   action_set_name: expression -> String
/// )
macro_rules! new_action {
    ($ident:ident, $type:ty, $key:literal, $name:expr, $set_key:expr, $set_name:expr) => {
        #[derive(Component, Default)]
        pub struct $ident {
            data: $type,
            previous_data: $type,
        }

        impl schminput_schmanager::ActionTrait for $ident {
            type T = $type;
            fn reset_value(&mut self) {
                self.previous_data = self.data;
                self.data = default();
            }
            fn set_value(&mut self, value: Self::T) {
                self.previous_data = self.data;
                self.data = value;
            }

            fn get_value(&self) -> &Self::T {
                &self.data
            }

            fn get_previous_value(&self) -> &Self::T {
                &self.previous_data
            }

            fn action_set_name(&self) -> String {
                $set_name
            }

            fn action_set_key(&self) -> &'static str {
                $set_key
            }

            fn action_name(&self) -> String {
                $name
            }

            fn action_key(&self) -> &'static str {
                $key
            }
        }
    };
}

#[bevy_trait_query::queryable]
pub trait ActionTrait {
    type T;
    fn reset_value(&mut self);
    fn set_value(&mut self, value: Self::T);
    fn get_value(&self) -> &Self::T;
    fn get_previous_value(&self) -> &Self::T;
    fn action_set_name(&self) -> String;
    fn action_set_key(&self) -> &'static str;
    fn action_name(&self) -> String;
    fn action_key(&self) -> &'static str;
    fn is_action(&self, other: &str) -> bool {
        other == self.action_key()
    }
}

#[bevy_trait_query::queryable]
pub trait ErasedAction {
    fn reset_value(&mut self);
}
impl<T: ActionTrait> ErasedAction for T {
    fn reset_value(&mut self) {
        self.reset_value();
    }
}

pub trait SchminputApp {
    fn register_action<T: ActionTrait + Component>(&mut self) -> &mut Self;
}

impl SchminputApp for App {
    fn register_action<T: ActionTrait + Component>(&mut self) -> &mut Self {
        self.register_component_as::<dyn ActionTrait<T = T::T>, T>();
        self.register_component_as::<dyn ErasedAction, T>();
        self
    }
}
