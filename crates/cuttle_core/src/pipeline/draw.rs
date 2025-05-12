use super::queue::{ConfigInstanceBuffer, CuttleBatches};
use super::specialization::CuttlePipeline;
use super::specialization::CuttleViewBindGroup;
use super::SortedCuttlePhaseItem;
use crate::components::buffer::{Bind, CompBufferEntity, ConfigRenderEntity};
use crate::configs::CuttleConfig;
use crate::extensions::CompIndicesBindGroup;
use crate::internal_prelude::*;
use bevy_ecs::system::lifetimeless::{Read, SQuery, SRes};
use bevy_ecs::system::SystemParamItem;
use bevy_render::render_phase::{
    RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
};
use bevy_render::render_resource::IndexFormat;
use bevy_render::view::{ExtractedView, ViewUniformOffset};
use std::marker::PhantomData;

pub type DrawCuttle<G> = (SetItemPipeline, PerFrame, PerConfig<G>, PerView, PerBatch);

pub struct PerFrame;
impl<P: SortedCuttlePhaseItem> RenderCommand<P> for PerFrame {
    type Param = (
        SRes<CompIndicesBindGroup>,
        SQuery<&'static Bind, With<CompBufferEntity>>,
        SRes<CuttlePipeline>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        _view: (),
        _entity: Option<()>,
        (component_indices, bind, pipeline): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(component_indices) = &component_indices.into_inner().0 else {
            return RenderCommandResult::Failure("cuttle component indices None");
        };
        pass.set_bind_group(1, component_indices, &[]);

        let Some(Bind(Some(bind))) = bind.iter_inner().next() else {
            return RenderCommandResult::Failure("cuttle component bind group is None");
        };
        pass.set_bind_group(2, bind, &[]);

        let pipeline = pipeline.into_inner();
        let Some(indices) = pipeline.indices.buffer() else {
            return RenderCommandResult::Failure("cuttle indices buffer not available");
        };
        pass.set_index_buffer(indices.slice(..), 0, IndexFormat::Uint16);

        RenderCommandResult::Success
    }
}

pub struct PerConfig<Config: CuttleConfig>(PhantomData<Config>);
impl<Config: CuttleConfig> RenderCommand<Config::Phase> for PerConfig<Config> {
    type Param = (
        SQuery<&'static Bind, With<ConfigRenderEntity<Config>>>,
        SRes<ConfigInstanceBuffer<Config>>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        _item: &Config::Phase,
        _view: (),
        _entity: Option<()>,
        (bind, instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Bind(Some(bind))) = bind.iter_inner().next() else {
            return RenderCommandResult::Failure("cuttle component bind group is None");
        };
        pass.set_bind_group(3, bind, &[]);

        let Some(vertices) = instances.into_inner().vertex.buffer() else {
            return RenderCommandResult::Failure("cuttle vertices buffer not available");
        };

        pass.set_vertex_buffer(0, vertices.slice(..));
        RenderCommandResult::Success
    }
}

pub struct PerView;
impl<P: SortedCuttlePhaseItem> RenderCommand<P> for PerView {
    type Param = ();
    type ViewQuery = (Read<ViewUniformOffset>, Read<CuttleViewBindGroup>);
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        (offset, bind_group): (&'w ViewUniformOffset, &'w CuttleViewBindGroup),
        _entity: Option<()>,
        _param: (),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(0, &bind_group.value, &[offset.offset]);
        RenderCommandResult::Success
    }
}

pub struct PerBatch;
impl<P: SortedCuttlePhaseItem> RenderCommand<P> for PerBatch {
    type Param = SRes<CuttleBatches>;
    type ViewQuery = Read<ExtractedView>;
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        view: &ExtractedView,
        sdf_instance: Option<()>,
        batches: Res<CuttleBatches>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(batch) =
            sdf_instance.and(batches.get(&(view.retained_view_entity, item.entity())))
        else {
            return RenderCommandResult::Skip;
        };
        pass.draw_indexed(0..6, 0, batch.range.clone());

        RenderCommandResult::Success
    }
}
