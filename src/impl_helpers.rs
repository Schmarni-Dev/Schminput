use std::hash::{DefaultHasher, Hash as _, Hasher};

use crate::{
    binding_modification::{
        BindingModifications, PremultiplyDeltaSecsModification, UnboundedModification,
    },
    prelude::*,
    priorities::BlockedInputs,
    subaction_paths::SubactionPath,
};
use bevy::{
    ecs::{
        query::{QueryData, QueryFilter},
        system::SystemParam,
    },
    prelude::*,
};
pub struct GenericBindingData<'s> {
    pub action: &'s Action,
    pub subaction_path: Option<SubactionPath>,
    pub modifications: Modifications<'s>,
    pub is_bool: bool,
    pub is_f32: bool,
    pub is_vec2: bool,
}

pub struct Modifications<'s> {
    pub inner: &'s BindingModifications,
    pub premul_delta_time: bool,
    pub unbounded: bool,
}

#[derive(Default, Clone, Copy)]
pub struct BindingValue {
    pub vec2: Option<Vec2>,
    pub bool: Option<bool>,
    pub f32: Option<f32>,
}

#[derive(SystemParam)]
pub struct ProviderParam<
    'w,
    's,
    ActionData: QueryData + 'static,
    PathData: QueryData + 'static,
    PathFilter: QueryFilter + 'static = (),
> {
    pub action_query: Query<
        'w,
        's,
        (
            ActionData,
            &'static Action,
            &'static RequestedSubactionPaths,
            &'static BindingModifications,
            Option<&'static mut BoolActionValue>,
            Option<&'static mut F32ActionValue>,
            Option<&'static mut Vec2ActionValue>,
        ),
    >,
    pub action_set_query: Query<'w, 's, (&'static ActionSet, Option<&'static BlockedInputs>)>,
    pub binding_modification_query: Query<
        'w,
        's,
        (
            Has<PremultiplyDeltaSecsModification>,
            Has<UnboundedModification>,
        ),
    >,
    pub path_query: Query<'w, 's, PathData, PathFilter>,
}
impl<
        ActionData: QueryData + 'static,
        PathData: QueryData + 'static,
        PathFilter: QueryFilter + 'static,
    > ProviderParam<'_, '_, ActionData, PathData, PathFilter>
{
    pub fn run<BindingData>(
        &mut self,
        label: &str,
        binding_id: impl Fn(&BindingData) -> u64,
        path_matches: impl Fn(
            &BindingData,
            &<<PathData as QueryData>::ReadOnly as QueryData>::Item<'_>,
        ) -> bool,
        bindings: impl Fn(&<ActionData as QueryData>::Item<'_>) -> Vec<BindingData>,
        mut update_for_binding: impl FnMut(
            &BindingData,
            &mut <ActionData as QueryData>::Item<'_>,

            Option<&<<PathData as QueryData>::ReadOnly as QueryData>::Item<'_>>,
            &GenericBindingData,
        ) -> Vec<BindingValue>,
    ) {
        let label_id = {
            let mut hasher = DefaultHasher::new();
            label.hash(&mut hasher);
            hasher.finish()
        };
        let _span = debug_span!("ProviderHelper::run").entered();
        for (mut data, action, req_sub_paths, modifications, mut bool, mut f32, mut vec2) in
            self.action_query.iter_mut()
        {
            let Ok((set, input)) = self.action_set_query.get(action.set) else {
                continue;
            };
            if !set.enabled {
                continue;
            };
            let (pre_mul_delta_time_all, unbounded_all) = modifications
                .all_paths
                .as_ref()
                .and_then(|v| self.binding_modification_query.get(v.0).ok())
                .unwrap_or_default();

            let binding_iter = bindings(&data);

            let all_binding_values = binding_iter
                .iter()
                .flat_map(|binding_data| {
                    if let Some(input) = input.as_ref() {
                        if input
                            .0
                            .get(&label_id)
                            .is_some_and(|v| v.contains(&binding_id(binding_data)))
                        {
                            return Vec::new();
                        }
                    }
                    let mut binding_modifications = Modifications {
                        inner: modifications,
                        premul_delta_time: pre_mul_delta_time_all,
                        unbounded: unbounded_all,
                    };
                    for (mod_sub_path, modification) in modifications.per_path.iter().copied() {
                        let Ok(path_data) = self.path_query.get(*mod_sub_path) else {
                            continue;
                        };
                        if path_matches(binding_data, &path_data) {
                            let Ok((pre_mul_delta_time, unbounded)) =
                                self.binding_modification_query.get(modification.0)
                            else {
                                continue;
                            };
                            binding_modifications.premul_delta_time |= pre_mul_delta_time;
                            binding_modifications.unbounded |= unbounded;
                        }
                    }

                    update_for_binding(
                        binding_data,
                        &mut data,
                        None,
                        &GenericBindingData {
                            action,
                            subaction_path: None,
                            modifications: binding_modifications,
                            is_bool: bool.is_some(),
                            is_f32: f32.is_some(),
                            is_vec2: vec2.is_some(),
                        },
                    )
                })
                .collect::<Vec<_>>();
            apply_values(
                None,
                all_binding_values,
                vec2.as_mut(),
                f32.as_mut(),
                bool.as_mut(),
            );

            for sub_path in req_sub_paths.iter() {
                let Ok(path_data) = self.path_query.get(**sub_path) else {
                    continue;
                };
                let mut out = Vec::<BindingValue>::new();
                for binding in binding_iter.iter() {
                    if let Some(input) = input {
                        if input
                            .0
                            .get(&label_id)
                            .is_some_and(|v| v.contains(&binding_id(binding)))
                        {
                            continue;
                        }
                    }
                    if !path_matches(binding, &path_data) {
                        continue;
                    }
                    // TODO: precompute this
                    let mut binding_modifications = Modifications {
                        inner: modifications,
                        premul_delta_time: pre_mul_delta_time_all,
                        unbounded: unbounded_all,
                    };
                    for (mod_sub_path, modification) in modifications.per_path.iter().copied() {
                        let Ok(path_data) = self.path_query.get(*mod_sub_path) else {
                            continue;
                        };
                        if path_matches(binding, &path_data) {
                            let Ok((pre_mul_delta_time, unbounded)) =
                                self.binding_modification_query.get(modification.0)
                            else {
                                continue;
                            };
                            binding_modifications.premul_delta_time |= pre_mul_delta_time;
                            binding_modifications.unbounded |= unbounded;
                        }
                    }
                    out.extend(update_for_binding(
                        binding,
                        &mut data,
                        Some(&path_data),
                        &GenericBindingData {
                            action,
                            subaction_path: Some(*sub_path),
                            modifications: binding_modifications,
                            is_bool: bool.is_some(),
                            is_f32: f32.is_some(),
                            is_vec2: vec2.is_some(),
                        },
                    ))
                }
                apply_values(
                    Some(*sub_path),
                    out,
                    vec2.as_mut(),
                    f32.as_mut(),
                    bool.as_mut(),
                );
            }
        }
    }
}

fn apply_values(
    sub_path: Option<SubactionPath>,
    iter: impl IntoIterator<Item = BindingValue>,
    vec2: Option<&mut Mut<'_, Vec2ActionValue>>,
    f32: Option<&mut Mut<'_, F32ActionValue>>,
    bool: Option<&mut Mut<'_, BoolActionValue>>,
) {
    let mut out_vec2 = Vec2::ZERO;
    let mut out_bool = false;
    let mut out_f32 = 0f32;
    for data in iter {
        // this is incompatible with the OpenXR spec
        // the spec states that the longest vec should be picked
        if let Some(data) = data.vec2 {
            out_vec2 += data;
        }
        // this is incompatible with the OpenXR spec
        // the spec states that the float with the largest absolute value should be picked
        if let Some(data) = data.f32 {
            out_f32 += data;
        }
        if let Some(data) = data.bool {
            out_bool |= data;
        }
    }

    if let Some(path) = sub_path {
        if let Some(vec2) = vec2 {
            *vec2.entry_with_path(path).or_default() += out_vec2;
        }
        if let Some(f32) = f32 {
            *f32.entry_with_path(path).or_default() += out_f32;
        }
        if let Some(bool) = bool {
            *bool.entry_with_path(path).or_default() |= out_bool;
        }
    } else {
        if let Some(vec2) = vec2 {
            *vec2.0 += out_vec2;
        }
        if let Some(f32) = f32 {
            *f32.0 += out_f32;
        }
        if let Some(bool) = bool {
            *bool.0 |= out_bool;
        }
    }
}
