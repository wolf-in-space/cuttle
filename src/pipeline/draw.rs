use super::queue::RenderPhaseBuffers;
use super::specialization::SdfViewBindGroup;
use super::RenderPhase;
use super::{queue::SdfBatch, specialization::SdfPipeline};
use bevy::prelude::error;
use bevy::{
    ecs::system::{
        lifetimeless::{Read, SRes},
        SystemParamItem,
    },
    render::{
        render_phase::{RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass},
        render_resource::IndexFormat,
        view::ViewUniformOffset,
    },
};

pub type DrawSdf = (SetItemPipeline, SetSdfViewBindGroup, DrawSdfDispatch);

pub struct SetSdfViewBindGroup;
impl<P: RenderPhase> RenderCommand<P> for SetSdfViewBindGroup {
    type Param = ();
    type ViewQuery = (Read<ViewUniformOffset>, Read<SdfViewBindGroup<P>>);
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view: (&'w ViewUniformOffset, &'w SdfViewBindGroup<P>),
        _entity: Option<()>,
        _param: (),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (offset, bind_group) = view;
        pass.set_bind_group(0, &bind_group.value, &[offset.offset]);
        RenderCommandResult::Success
    }
}

pub struct DrawSdfDispatch;
impl<P: RenderPhase> RenderCommand<P> for DrawSdfDispatch {
    type Param = (SRes<SdfPipeline>, SRes<RenderPhaseBuffers<P>>);
    type ViewQuery = ();
    type ItemQuery = Read<SdfBatch>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        sdf_instance: Option<&'w SdfBatch>,
        (pipeline, buffers): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(instance) = sdf_instance else {
            return RenderCommandResult::Failure;
        };
        let pipeline = pipeline.into_inner();
        let Some(vertices) = buffers.into_inner().vertex.buffer() else {
            error!("Cancelled draw: 'bevy_comdf sdf vertices buffer not available'");
            return RenderCommandResult::Failure;
        };
        let Some(indices) = pipeline.indices.buffer() else {
            error!("Cancelled draw: 'bevy_comdf sdf indices buffer not available'");
            return RenderCommandResult::Failure;
        };
        let Some(bind_group) = pipeline.bind_groups.get(&instance.key.flags) else {
            error!(
                "Cancelled draw: 'bind_group not found for key {:?}'",
                instance.key
            );
            return RenderCommandResult::Failure;
        };

        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.set_bind_group(1, bind_group, &[]);
        pass.set_index_buffer(indices.slice(..), 0, IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, instance.range.clone());
        // info!("DRAW {:?}", instance.range);
        RenderCommandResult::Success
    }
}
