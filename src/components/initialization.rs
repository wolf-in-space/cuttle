use crate::bounding::{BoundingSet, InitBoundingFn};
use crate::components::arena::IndexArena;
use crate::components::buffer::{BufferFns, CompBuffer};
use crate::components::build_set_flag_bit;
use crate::groups::{GlobalGroupInfos, GroupBuilder, SdfGroup};
use crate::pipeline::extract::build_extract_sdf_comp;
use crate::shader::wgsl_struct::WgslTypeInfos;
use bevy::prelude::*;
use bevy::reflect::{TypeInfo, Typed};
use bevy::render::render_resource::encase::private::WriteInto;
use bevy::render::render_resource::ShaderSize;
use bevy::render::RenderApp;
use serde::{Deserialize, Serialize};
use std::any::{type_name, TypeId};
use std::marker::PhantomData;

pub type InitComponentFn = fn(&mut App, u8) -> ComponentShaderInfo;

pub struct InitComponentInfo {
    pub(crate) sort: u32,
    pub(crate) init_fn: InitComponentFn,
    pub(crate) init_bounding: Option<InitBoundingFn>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ComponentShaderInfo {
    pub name: String,
    pub(crate) render_data: Option<RenderDataShaderInfo>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RenderDataShaderInfo {
    pub binding: u32,
    pub struct_wgsl: String,
}

pub(crate) fn init_components(
    app: &mut App,
    mut init_infos: Vec<InitComponentInfo>,
) -> Vec<ComponentShaderInfo> {
    init_infos.sort_by_key(|info| info.sort);
    init_infos
        .into_iter()
        .enumerate()
        .map(|(i, init)| {
            if let Some(mut bounding_fn) = init.init_bounding {
                bounding_fn(app);
            }
            (init.init_fn)(app, i as u8)
        })
        .collect()
}

pub(crate) fn common_component_init<C: Component, R: Typed, G: SdfGroup>(app: &mut App, pos: u8) {
    app.init_resource::<IndexArena<C>>();
    app.add_observer(build_set_flag_bit::<C, G, OnAdd, true>(pos));
    app.add_observer(build_set_flag_bit::<C, G, OnRemove, false>(pos));
}

pub(crate) fn init_zst_component<C, G>(app: &mut App, pos: u8) -> ComponentShaderInfo
where
    C: Component + Typed,
    G: SdfGroup,
{
    common_component_init::<C, C, G>(app, pos);

    let Some(name) = C::type_ident() else {
        panic!("Component {} is not a named struct!", type_name::<C>())
    };

    ComponentShaderInfo {
        name: name.to_string(),
        render_data: None,
    }
}

pub(crate) fn init_component_with_render_data<C, R, G>(
    app: &mut App,
    pos: u8,
) -> ComponentShaderInfo
where
    C: Component,
    R: SdfRenderDataFrom<C>,
    G: SdfGroup,
{
    common_component_init::<C, R, G>(app, pos);
    app.sub_app_mut(RenderApp)
        .add_systems(ExtractSchedule, build_extract_sdf_comp::<C, R>(pos));

    let binding = global_init_component::<C, R>(app);

    let (TypeInfo::Struct(structure), Some(name)) = (R::type_info(), R::type_ident()) else {
        panic!(
            "Render data {} for component {} is not a named struct",
            type_name::<R>(),
            type_name::<C>()
        )
    };

    let struct_wgsl = app
        .world()
        .resource::<WgslTypeInfos>()
        .structure_to_wgsl(structure, name);

    ComponentShaderInfo {
        name: name.to_string(),
        render_data: Some(RenderDataShaderInfo {
            struct_wgsl,
            binding,
        }),
    }
}

pub(crate) fn global_init_component<C: Component, R: SdfRenderData>(app: &mut App) -> u32 {
    let id = TypeId::of::<C>();
    if let Some(binding) = app
        .world()
        .resource::<GlobalGroupInfos>()
        .component_bindings
        .get(&id)
    {
        return *binding;
    }

    let mut globals = app
        .world_mut()
        .remove_resource::<GlobalGroupInfos>()
        .unwrap();
    let binding = globals.component_bindings.len() as u32;
    globals.component_bindings.insert(id, binding);

    let render_world = app.sub_app_mut(RenderApp).world_mut();
    render_world
        .entity_mut(globals.buffer_entity.id())
        .insert(CompBuffer::<R>::default());
    let mut buffer_fns = render_world.resource_mut::<BufferFns>();
    buffer_fns.write.push(CompBuffer::<R>::write);
    buffer_fns.bindings.push(CompBuffer::<R>::get_binding_res);

    app.world_mut().insert_resource(globals);

    binding
}

pub trait ZstSdfComponent: Component + Sized + Typed {
    const SORT: u32;
}

pub trait SdfRenderData: ShaderSize + Default + Typed + WriteInto {}
impl<T: ShaderSize + Default + Typed + WriteInto> SdfRenderData for T {}

pub struct ComponentBuilder<'a, G, C, R> {
    pub builder: &'a mut GroupBuilder<'a, G>,
    pub init_bounding: Option<InitBoundingFn>,
    pub marker: PhantomData<(C, R)>,
}

pub trait SdfRenderDataFrom<C>: SdfRenderData {
    fn from_sdf_comp(comp: &C) -> Self;
}

impl<C> SdfRenderDataFrom<C> for C
where
    C: Component + SdfRenderData + Clone,
{
    fn from_sdf_comp(comp: &C) -> C {
        comp.clone()
    }
}

pub trait SdfComponent: Component + Sized {
    type RenderData: SdfRenderDataFrom<Self>;
    const AFFECT_BOUNDS: BoundingSet = BoundingSet::None;
    const SORT: u32;

    #[allow(unused)]
    fn affect_bounds(comp: &Self) -> f32 {
        0.
    }
    fn registration_data() -> RegisterSdfComponent<Self, Self::RenderData> {
        RegisterSdfComponent {
            affect_bounds: Self::AFFECT_BOUNDS,
            affect_bounds_fn: match Self::AFFECT_BOUNDS {
                BoundingSet::None | BoundingSet::Apply => None,
                BoundingSet::Add | BoundingSet::Multiply => Some(Self::affect_bounds),
            },
            sort: Self::SORT,
            marker: default(),
        }
    }
}

pub struct RegisterSdfComponent<C: Component, R: SdfRenderDataFrom<C>> {
    pub affect_bounds: BoundingSet,
    pub affect_bounds_fn: Option<fn(&C) -> f32>,
    pub sort: u32,
    pub marker: PhantomData<R>,
}

impl<C: Component, R: SdfRenderDataFrom<C>> Default for RegisterSdfComponent<C, R> {
    fn default() -> Self {
        Self {
            affect_bounds: default(),
            affect_bounds_fn: None,
            sort: 0,
            marker: PhantomData,
        }
    }
}
