use crate::components::arena::IndexArena;
use crate::components::buffer::CompBuffer;
use crate::groups::GlobalGroupInfos;
use crate::indices::CuttleComponentIndex;
use crate::pipeline::extract::extract_cuttle_comp;
use crate::shader::wgsl_struct::WgslTypeInfos;
use crate::shader::RenderDataWgsl;
use bevy::prelude::*;
use bevy::reflect::Typed;
use bevy::render::render_resource::encase::private::WriteInto;
use bevy::render::render_resource::ShaderSize;
use bevy::render::RenderApp;
use std::any::TypeId;
use std::fmt::Debug;
use std::ops::Deref;

pub struct ComponentOrder {
    pub id: TypeId,
    pub sort: u32,
}

/// Returns the registered binding for the component or register a new binding
/// to return and do the setup needed once per component.
pub(crate) fn init_render_data<C: Component, R: CuttleRenderData>(
    app: &mut App,
    to_render_data: fn(&C) -> R,
) -> u32 {
    let id = TypeId::of::<C>();
    let mut globals = app.world_mut().resource_mut::<GlobalGroupInfos>();
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

pub trait CuttleRenderData: Debug + ShaderSize + Default + Typed + WriteInto {}
impl<T: Debug + ShaderSize + Default + Typed + WriteInto> CuttleRenderData for T {}

pub trait CuttleStructComponent: Typed + Component {
    type RenderData: CuttleRenderData;

    fn wgsl_type(wgsl_types: &WgslTypeInfos) -> RenderDataWgsl {
        wgsl_types.wgsl_type_for_struct::<Self::RenderData>()
    }

    fn to_render_data(&self) -> Self::RenderData;
}

/// Blanket implementation for Structs that have
/// the same data as a Component and on the GPU.
impl<C: Component + CuttleRenderData + Clone> CuttleStructComponent for C {
    type RenderData = C;
    fn to_render_data(&self) -> Self::RenderData {
        self.clone()
    }
}

/// A
///
/// ```
/// # use bevy::prelude::{App, Component, Deref, Reflect};
/// # use cuttle::groups::CuttleGroupBuilderAppExt;
/// # use cuttle::prelude::{Sdf, SdfOrder};
/// # let mut app = App::new();
///
/// #[derive(Component, Reflect, Deref)]
/// struct Rounded(f32);
///
/// app.cuttle_group::<Sdf>().wrapper_component::<Rounded>(SdfOrder::Distance);
///
/// ```
pub trait CuttleWrapperComponent: Typed + Component {
    type RenderData: CuttleRenderData;
    fn wgsl_type(wgsl_types: &WgslTypeInfos) -> RenderDataWgsl {
        wgsl_types.wgsl_type_for_builtin::<Self::RenderData>()
    }
    fn to_render_data(&self) -> Self::RenderData;
}

impl<C, R> CuttleWrapperComponent for C
where
    C: Component + Deref<Target = R> + Typed,
    R: CuttleRenderData + Clone,
{
    type RenderData = R;

    fn to_render_data(&self) -> Self::RenderData {
        self.deref().clone()
    }
}
