use crate::components::arena::IndexArena;
use crate::components::buffer::{CompBuffer, GlobalBuffer};
use crate::configs::builder::CuttleBuilder;
use crate::configs::global::GlobalConfigInfos;
use crate::indices::{CuttleComponentIndex, added_cuttle_component};
use crate::internal_prelude::*;
use crate::pipeline::extract::{extract_cuttle_comp, extract_cuttle_global};
use crate::shader::wgsl_struct::{WgslType, WgslTypes};
use crate::shader::{AddSnippet, RenderData, Snippets};
use bevy_reflect::Typed;
use bevy_render::RenderApp;
use bevy_render::render_resource::ShaderSize;
use bevy_render::render_resource::encase::internal::WriteInto;
use bevy_render::sync_world::RenderEntity;
use convert_case::{Case, Casing};
use std::fmt::Debug;

pub trait Cuttle: Component + Typed + Sized {
    fn build(entity: CuttleBuilder<Self>);
}

pub trait CuttleRenderData: Debug + ShaderSize + Default + Typed + WriteInto {}
impl<T: Debug + ShaderSize + Default + Typed + WriteInto> CuttleRenderData for T {}

pub fn init_component_render_data<C: Component, R: CuttleRenderData>(
    app: &mut App,
    entity: Entity,
    to_render_data: fn(&C) -> R,
) {
    if app.world().entity(entity).contains::<RenderData>() {
        return;
    }

    let mut globals = app.world_mut().resource_mut::<GlobalConfigInfos>();
    let binding = globals.binding();
    let buffer = globals.buffer_entity.id();

    CompBuffer::<C, R>::init(app, buffer, to_render_data);

    app.register_required_components::<C, CuttleComponentIndex<C>>();
    app.init_resource::<IndexArena<C>>();

    app.sub_app_mut(RenderApp)
        .add_systems(ExtractSchedule, extract_cuttle_comp::<C, R>);

    let wgsl_types = app.world().resource::<WgslTypes>();
    let WgslType { type_name, snippet } = wgsl_types.get_type::<R>();
    let mut entity = app.world_mut().entity_mut(entity);

    if let Some(snippet) = snippet {
        entity
            .get_mut::<Snippets>()
            .unwrap()
            .push(AddSnippet::Inline(snippet));
    }

    entity.insert(RenderData { binding, type_name });
}

pub fn init_global_render_data<C: Component, R: CuttleRenderData>(
    app: &mut App,
    config_entity: Entity,
    to_render_data: fn(&C) -> R,
) {
    let buffer_entity = app
        .world()
        .entity(config_entity)
        .get::<RenderEntity>()
        .unwrap()
        .id();
    let binding = GlobalBuffer::init(app, buffer_entity, to_render_data);

    let WgslType { type_name, snippet } = app.world().resource::<WgslTypes>().get_type::<R>();

    let snippet = format!(
        "@group(3) @binding({}) var<storage, read> {}: {};\n\n{}",
        binding,
        type_name.to_case(Case::Snake),
        type_name,
        snippet.unwrap_or_default()
    );

    app.world_mut()
        .entity_mut(config_entity)
        .get_mut::<Snippets>()
        .unwrap()
        .push(AddSnippet::Inline(snippet));

    app.sub_app_mut(RenderApp)
        .add_systems(ExtractSchedule, extract_cuttle_global::<C, R>);
}
