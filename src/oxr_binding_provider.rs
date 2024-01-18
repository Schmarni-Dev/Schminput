// TODOS:
// Location/Velocity flags
// Rumble like bevy
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_oxr::{
    input::XrInput,
    resources::{XrFrameState, XrSession},
    xr_init::{xr_only, XrSetup},
    xr_input::{
        action_set_system,
        actions::{ActionHandednes, ActionType, SetupActionSets, XrActionSets, XrBinding},
        QuatConv, Vec2Conv, Vec3Conv,
    },
};
use openxr::{ActionState, AnyGraphics, Posef, Session};

use crate::ActionTrait;

pub struct OXRBindingProvider;

impl Plugin for OXRBindingProvider {
    fn build(&self, app: &mut App) {
        app.insert_resource(CachedXrActionToOXRActions(default()));
        app.insert_resource(OXRSetupBindings {
            bindings: default(),
        });
        app.add_systems(XrSetup, transfer_bindings);
        app.add_systems(
            PreUpdate,
            (
                sync_actions_bool,
                sync_actions_f32,
                sync_actions_vec2,
                sync_actions_transform,
                sync_actions_velocity,
            )
                .run_if(xr_only())
                .after(action_set_system),
        );
    }
}
#[derive(Resource)]
struct CachedXrActionToOXRActions(HashMap<&'static str, (&'static str, &'static str)>);

fn sync_actions_bool(
    mut actions: Query<&mut dyn ActionTrait<T = bool>>,
    action_sets: Res<XrActionSets>,
    session: Res<XrSession>,
) {
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            if let Ok(oxr_value) =
                action_sets.get_action_bool(action.action_set_key(), action.action_key())
            {
                if let Ok(ActionState { current_state, .. }) =
                    oxr_value.state(&session, openxr::Path::NULL)
                {
                    let mut v = *action.get_value();
                    v = v || current_state;
                    action.set_value(v);
                }
            }
        })
    });
}
fn sync_actions_vec2(
    mut actions: Query<&mut dyn ActionTrait<T = Vec2>>,
    action_sets: Res<XrActionSets>,
    session: Res<XrSession>,
) {
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            if let Ok(oxr_value) =
                action_sets.get_action_vec2(action.action_set_key(), action.action_key())
            {
                if let Ok(ActionState { current_state, .. }) =
                    oxr_value.state(&session, openxr::Path::NULL)
                {
                    let v = *action.get_value();
                    action.set_value(current_state.to_vec2() + v);
                }
            }
        })
    });
}

fn sync_actions_transform(
    mut actions: Query<&mut dyn ActionTrait<T = Transform>>,
    action_sets: Res<XrActionSets>,
    session: Res<XrSession>,
    xr_input: Res<XrInput>,
    frame_state: Res<XrFrameState>,
) {
    let s = Session::<AnyGraphics>::clone(&session);
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            if let Ok(oxr_value) =
                action_sets.get_action_posef(action.action_set_key(), action.action_key())
            {
                if let Ok(space) =
                    oxr_value.create_space(s.clone(), openxr::Path::NULL, Posef::IDENTITY)
                {
                    if let Ok((location, _velocity)) = space.relate(
                        &xr_input.stage,
                        frame_state.lock().unwrap().predicted_display_time,
                    ) {
                        let mut transform = Transform::IDENTITY;
                        transform.translation = location.pose.position.to_vec3();
                        transform.rotation = location.pose.orientation.to_quat();
                        action.set_value(transform);
                    }
                }
            }
        })
    });
}

fn sync_actions_velocity(
    mut actions: Query<&mut dyn ActionTrait<T = Velocity>>,
    action_sets: Res<XrActionSets>,
    session: Res<XrSession>,
    xr_input: Res<XrInput>,
    frame_state: Res<XrFrameState>,
) {
    let s = Session::<AnyGraphics>::clone(&session);
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            if let Ok(oxr_value) =
                action_sets.get_action_posef(action.action_set_key(), action.action_key())
            {
                if let Ok(space) =
                    oxr_value.create_space(s.clone(), openxr::Path::NULL, Posef::IDENTITY)
                {
                    if let Ok((_location, velocity)) = space.relate(
                        &xr_input.stage,
                        frame_state.lock().unwrap().predicted_display_time,
                    ) {
                        action.set_value(Velocity {
                            linear: velocity.linear_velocity.to_vec3(),
                            angular: velocity.angular_velocity.to_vec3(),
                        });
                    }
                }
            }
        })
    });
}
fn sync_actions_f32(
    mut actions: Query<&mut dyn ActionTrait<T = f32>>,
    action_sets: Res<XrActionSets>,
    session: Res<XrSession>,
) {
    actions.par_iter_mut().for_each(|mut e| {
        e.iter_mut().for_each(|mut action| {
            if let Ok(oxr_value) =
                action_sets.get_action_f32(action.action_set_key(), action.action_key())
            {
                if let Ok(ActionState { current_state, .. }) =
                    oxr_value.state(&session, openxr::Path::NULL)
                {
                    let mut v = *action.get_value();
                    v += current_state;
                    action.set_value(v);
                }
            }
        })
    });
}

struct CachedXrActions(HashSet<&'static str>);
impl CachedXrActions {
    fn get(&self, binding: &'static str) -> bool {
        self.0.contains(binding)
    }
    fn set(&mut self, binding: &'static str) {
        self.0.insert(binding);
    }
}

#[allow(unreachable_code, clippy::diverging_sub_expression)]
fn transfer_bindings(
    schminput_bindings: Res<OXRSetupBindings>,
    mut oxr_bindings: ResMut<SetupActionSets>,
    mut commands: Commands,
) {
    commands.remove_resource::<OXRSetupBindings>();
    let mut cached_xr_actions = CachedXrActions(default());
    for (set_key, (set_name, actions)) in schminput_bindings.bindings.iter() {
        let set = oxr_bindings.add_action_set(set_key, set_name.to_string(), 0);
        for (action_key, (action_name, action_type, action_bindings)) in actions {
            for binding in action_bindings {
                let OXRBinding { device, binding } = binding;
                if !cached_xr_actions.get(action_key) {
                    set.new_action(
                        action_key,
                        action_name.to_string(),
                        *action_type,
                        ActionHandednes::Single,
                    );
                    cached_xr_actions.set(action_key);
                }
                let bindings = &[XrBinding::new(action_key, binding)];
                set.suggest_binding(device, bindings);
            }
        }
    }
}

pub struct OXRBinding {
    pub device: &'static str,
    pub binding: &'static str,
}

#[derive(Resource, Default)]
pub struct OXRSetupBindings {
    #[allow(clippy::type_complexity)]
    bindings: HashMap<
        &'static str,
        (
            String,
            HashMap<&'static str, (String, ActionType, Vec<OXRBinding>)>,
        ),
    >,
}

impl OXRSetupBindings {
    // I don't understand why this 'static is needed or why it works
    pub fn add_binding<T: 'static + ActionTypeFromActionT>(
        &mut self,
        action: &dyn ActionTrait<T = T>,
        binding: OXRBinding,
    ) {
        self.bindings
            .entry(action.action_set_key())
            .or_insert((action.action_set_name(), HashMap::new()))
            .1
            .entry(action.action_key())
            .or_insert((action.action_name(), T::get_action_type(), Vec::new()))
            .2
            .push(binding);
    }
}

pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl ActionTypeFromActionT for Velocity {
    fn get_action_type() -> ActionType {
        ActionType::PoseF
    }
}
impl ActionTypeFromActionT for Transform {
    fn get_action_type() -> ActionType {
        ActionType::PoseF
    }
}
impl ActionTypeFromActionT for f32 {
    fn get_action_type() -> ActionType {
        ActionType::F32
    }
}
impl ActionTypeFromActionT for Vec2 {
    fn get_action_type() -> ActionType {
        ActionType::Vec2
    }
}
impl ActionTypeFromActionT for bool {
    fn get_action_type() -> ActionType {
        ActionType::Bool
    }
}

pub trait ActionTypeFromActionT {
    fn get_action_type() -> ActionType;
}
