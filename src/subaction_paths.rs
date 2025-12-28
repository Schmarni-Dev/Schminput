use atomicow::CowArc;
use bevy::{ecs::entity::EntityHash, platform::collections::{hash_map::Entry, HashMap}, prelude::*};

use crate::SchminputSystems;

pub struct SubactionPathPlugin;

impl Plugin for SubactionPathPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SubactionPaths>();
        app.add_message::<SubactionPathCreated>();
        app.add_systems(
            PreUpdate,
            emit_new_path_events.before(SchminputSystems::HandleNewSubactionPaths),
        );
    }
}

fn emit_new_path_events(
    mut paths: ResMut<SubactionPaths>,
    mut e: MessageWriter<SubactionPathCreated>,
) {
    e.write_batch(paths.new_paths.iter().copied().map(SubactionPathCreated));
    paths.new_paths.clear();
}

#[derive(Clone, Copy, Debug, Message)]
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

impl RequestedSubactionPaths {
    pub fn with_path(
        mut self,
        path: impl Into<CowArc<'static, str>>,
        paths: &mut SubactionPaths,
        cmds: &mut Commands,
    ) -> Self {
        self.push_path(path, paths, cmds);
        self
    }
    pub fn push_path(
        &mut self,
        path: impl Into<CowArc<'static, str>>,
        paths: &mut SubactionPaths,
        cmds: &mut Commands,
    ) {
        self.push(paths.get_or_create_path(path, cmds));
    }
    pub fn new() -> Self {
        Self::default()
    }
    /// Helper for pushing paths
    pub fn mutate<'s, 'w, 'p>(
        self,
        paths: &'p mut SubactionPaths,
        cmds: Commands<'s, 'w>,
    ) -> RequestedSubactionPathsMutator<'s, 'w, 'p> {
        RequestedSubactionPathsMutator(self, cmds, paths)
    }
}
/// Helper for pushing paths
pub struct RequestedSubactionPathsMutator<'s, 'w, 'p>(
    RequestedSubactionPaths,
    Commands<'s, 'w>,
    &'p mut SubactionPaths,
);
impl RequestedSubactionPathsMutator<'_, '_, '_> {
    pub fn push(mut self, path: impl Into<CowArc<'static, str>>) -> Self {
        self.0.push_path(path, self.2, &mut self.1);
        self
    }
    pub fn push_many(
        mut self,
        paths: impl IntoIterator<Item = impl Into<CowArc<'static, str>>>,
    ) -> Self {
        for path in paths.into_iter() {
            self.0.push_path(path, self.2, &mut self.1);
        }
        self
    }
    pub fn end(self) -> RequestedSubactionPaths {
        self.0
    }
}

#[derive(Debug, Clone, Component, Reflect, PartialEq, Eq, Deref, DerefMut, Default)]
pub struct SubactionPathMap<T: Sized + Default> {
    pub paths: HashMap<SubactionPath, T, EntityHash>,
    #[deref]
    pub any: T,
}
impl<T: Default + Clone> SubactionPathMap<T> {
    pub fn get_with_path_or_default(&self, path: &SubactionPath) -> T {
        self.paths.get(path).cloned().unwrap_or_default()
    }
    pub fn get_or_default(&self, path: Option<SubactionPath>) -> T {
        if path.is_none() {
            warn!("path does not exist!");
        }
        path.and_then(|p| self.paths.get(&p))
            .cloned()
            .unwrap_or_default()
    }
}
impl<T: Default> SubactionPathMap<T> {
    pub fn get_with_path(&self, path: &SubactionPath) -> Option<&T> {
        self.paths.get(path)
    }
    pub fn entry_with_path(
        &mut self,
        path: SubactionPath,
    ) -> Entry<'_, SubactionPath, T, EntityHash> {
        self.paths.entry(path)
    }

    pub fn set_value_for_path(&mut self, path: SubactionPath, value: T) {
        self.paths.entry(path).insert(value);
    }
    pub fn new() -> SubactionPathMap<T> {
        default()
    }
}
