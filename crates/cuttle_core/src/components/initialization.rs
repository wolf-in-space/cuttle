use crate::components::arena::IndexArena;
use crate::components::buffer::CompBuffer;
use crate::configs::global::GlobalConfigInfos;
use crate::indices::CuttleComponentIndex;
use crate::internal_prelude::*;
use crate::pipeline::extract::extract_cuttle_comp;
use crate::shader::wgsl_struct::WgslTypeInfos;
use crate::shader::RenderDataWgsl;
use bevy_reflect::Typed;
use bevy_render::render_resource::encase::internal::WriteInto;
use bevy_render::render_resource::ShaderSize;
use bevy_render::RenderApp;
use std::any::TypeId;
use std::fmt::Debug;

#[derive(Debug, Reflect)]
pub struct ComponentOrder {
    pub id: TypeId,
    pub sort: u32,
    pub extension_override: Option<u8>,
}

/// Returns the registered binding for the component or register a new binding
/// to return and do the setup needed once per component.
pub fn init_render_data<C: Component, R: CuttleRenderData>(
    app: &mut App,
    to_render_data: fn(&C) -> R,
) -> u32 {
    let id = TypeId::of::<C>();
    let mut globals = app.world_mut().resource_mut::<GlobalConfigInfos>();
    if let Some(binding) = globals.component_bindings.get(&id) {
        return *binding;
    }

    let binding = globals.component_bindings.len() as u32;
    globals.component_bindings.insert(id, binding);

    let buffer = globals.buffer_entity.id();
    CompBuffer::<C, R>::init(app, buffer, to_render_data);

    app.register_required_components::<C, CuttleComponentIndex<C>>();
    app.init_resource::<IndexArena<C>>();

    app.sub_app_mut(RenderApp)
        .add_systems(ExtractSchedule, extract_cuttle_comp::<C, R>);

    binding
}

pub trait Cuttle: Typed + Component {
    const HAS_RENDER_DATA: bool;
    type RenderData: CuttleRenderData;
    const EXTENSION_INDEX_OVERRIDE: Option<u8>;

    fn wgsl_type(wgsl_types: &WgslTypeInfos) -> RenderDataWgsl;

    fn to_render_data(&self) -> Self::RenderData;

    fn sort() -> u32;
}

pub trait CuttleRenderData: Debug + ShaderSize + Default + Typed + WriteInto {}
impl<T: Debug + ShaderSize + Default + Typed + WriteInto> CuttleRenderData for T {}
