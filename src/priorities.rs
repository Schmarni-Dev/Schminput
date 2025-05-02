use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::{
    ecs::{entity::EntityHashMap, system::SystemId}, platform::collections::{HashMap, HashSet}, prelude::*
};

use crate::{ActionSet, ActionsInSet, SchminputSet};

pub struct PrioritiesPlugin;
impl Plugin for PrioritiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BindingIdSystems>();
        app.add_systems(
            PreUpdate,
            system.in_set(SchminputSet::CalculateBindingCollisions),
        );
    }
}

fn system(world: &mut World) {
    let mut bindings = EntityHashMap::<HashMap<u64, Vec<u64>>>::default();
    let binding_id_systems = world.remove_resource::<BindingIdSystems>().unwrap();
    let query = world
        .query::<(Entity, &ActionSet, &ActionsInSet)>()
        .iter(world)
        .filter(|(_, set, _)| set.enabled && !set.transparent)
        .map(|(e, _, a)| (e, a.0.iter().cloned().collect::<Vec<_>>()))
        .collect::<Vec<_>>();
    for (entity, actions) in query.into_iter() {
        for action in actions.into_iter() {
            for (binding_type_id, system) in binding_id_systems.0.iter() {
                let id = match world.run_system_with(*system, action) {
                    Ok(id) => id,
                    Err(err) => {
                        error!("error while running binding id system: {err}");
                        continue;
                    }
                };
                bindings
                    .entry(entity)
                    .or_default()
                    .entry(*binding_type_id)
                    .or_default()
                    .extend(id);
            }
        }
    }
    let priorities = world
        .query::<(Entity, &ActionSet)>()
        .iter(world)
        .filter(|(_, s)| s.enabled && !s.transparent)
        .map(|(e, s)| (e, s.priority))
        .collect::<Vec<_>>();
    let mut priority_sets = HashMap::<u32, Vec<Entity>>::new();
    for (e, p) in priorities.iter().cloned() {
        priority_sets.entry(p).or_default().push(e);
    }
    let mut sets_set = priority_sets.into_iter().collect::<Vec<_>>();
    sets_set.sort_by_key(|&(priority, _)| priority);
    sets_set.reverse();
    let mut last: HashMap<u64, HashSet<u64>> = default();
    for (_, sets) in sets_set {
        let blocked = BlockedInputs(last.clone());
        for set in sets {
            world.entity_mut(set).insert(blocked.clone());
            let Some(data) = bindings.get(&set) else {
                continue;
            };
            for (binding_type_id, ids) in data {
                last.entry(*binding_type_id).or_default().extend(ids);
            }
        }
    }

    world.insert_resource(binding_id_systems);
}

#[derive(Clone, Component, Debug)]
pub struct BlockedInputs(pub HashMap<u64, HashSet<u64>>);

#[derive(Resource, Default)]
struct BindingIdSystems(HashMap<u64, SystemId<In<Entity>, Vec<u64>>>);

pub trait PriorityAppExt {
    fn add_binding_id_system<M>(
        &mut self,
        label: &str,
        system: impl IntoSystem<In<Entity>, Vec<u64>, M> + 'static,
    ) -> &mut Self;
}
impl PriorityAppExt for App {
    fn add_binding_id_system<M>(
        &mut self,
        label: &str,
        system: impl IntoSystem<In<Entity>, Vec<u64>, M> + 'static,
    ) -> &mut Self {
        self.init_resource::<BindingIdSystems>();
        let mut hasher = DefaultHasher::new();
        label.hash(&mut hasher);
        let system = self.register_system(system);
        self.world_mut()
            .resource_mut::<BindingIdSystems>()
            .0
            .insert(hasher.finish(), system);
        self
    }
}
