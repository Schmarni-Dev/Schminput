use atomicow::CowArc;
use bevy::{ecs::entity::EntityHash, prelude::*, utils::hashbrown::HashMap};

use crate::SchminputSet;

pub struct SubactionPathPlugin;

impl Plugin for SubactionPathPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SubactionPaths>();
        app.add_event::<SubactionPathCreated>();
        app.add_systems(
            PreUpdate,
            emit_new_path_events.before(SchminputSet::HandleNewSubactionPaths),
        );
    }
}

fn emit_new_path_events(
    mut paths: ResMut<SubactionPaths>,
    mut e: EventWriter<SubactionPathCreated>,
) {
    e.send_batch(paths.new_paths.iter().copied().map(SubactionPathCreated));
    paths.new_paths.clear();
}

#[derive(Clone, Copy, Debug, Event)]
pub struct SubactionPathCreated(pub SubactionPath);

#[derive(Resource, Debug, Default)]
pub struct SubactionPaths {
    map: HashMap<CowArc<'static, str>, SubactionPath>,
    new_paths: Vec<SubactionPath>,
}

#[derive(Clone, Component, Debug)]
pub struct SubactionPathStr(pub CowArc<'static, str>);

impl SubactionPaths {
    pub fn get_or_create_path<P: Into<CowArc<'static, str>>>(
        &mut self,
        path: P,
        cmds: &mut Commands,
    ) -> SubactionPath {
        *self.map.entry(path.into()).or_insert_with_key(|p| {
            let path = SubactionPath(cmds.spawn(SubactionPathStr(p.clone())).id());
            self.new_paths.push(path);
            path
        })
    }
    pub fn get<P: Into<CowArc<'static, str>>>(&self, path: P) -> Option<SubactionPath> {
        self.map.get(&path.into()).copied()
    }
}

/// Subaction Path for action
#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Deref, Hash)]
#[repr(transparent)]
pub struct SubactionPath(pub Entity);

#[derive(Debug, Clone, Component, Reflect, PartialEq, Eq, Deref, Default, DerefMut)]
pub struct RequestedSubactionPaths(pub Vec<SubactionPath>);

#[derive(Debug, Clone, Component, Reflect, PartialEq, Eq, Deref, DerefMut, Default)]
pub struct SubactionPathMap<T: Sized + Default> {
    pub paths: HashMap<SubactionPath, T, EntityHash>,
    #[deref]
    pub any: T,
}
impl<T: Default> SubactionPathMap<T> {
    pub fn get_with_path(&self, path: &SubactionPath) -> Option<&T> {
        self.paths.get(path)
    }
    pub fn entry_with_path(
        &mut self,
        path: SubactionPath,
    ) -> bevy::utils::hashbrown::hash_map::Entry<'_, SubactionPath, T, EntityHash> {
        self.paths.entry(path)
    }

    pub fn set_value_for_path(&mut self, path: SubactionPath, value: T) {
        self.paths.entry(path).insert(value);
    }
    pub fn new() -> SubactionPathMap<T> {
        default()
    }
}
