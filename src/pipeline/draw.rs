use super::queue::RenderPhaseBuffers;
use super::specialization::SdfViewBindGroup;
use super::RenderPhase;
use super::{queue::SdfBatch, specialization::SdfPipeline};
use crate::components::buffer::CompBufferBindgroup;
use crate::extensions::OpBindgroup;
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
    type ViewQuery = (Read<ViewUniformOffset>, Read<SdfViewBindGroup>);
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view: (&'w ViewUniformOffset, &'w SdfViewBindGroup),
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
    type Param = (
        SRes<SdfPipeline>,
        SRes<RenderPhaseBuffers>,
        SRes<CompBufferBindgroup>,
        SRes<OpBindgroup>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<SdfBatch>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        sdf_instance: Option<&'w SdfBatch>,
        (pipeline, vertices, comp_buffers, op_buffers): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(instance) = sdf_instance else {
            return RenderCommandResult::Skip;
        };
        let pipeline = pipeline.into_inner();
        let Some(vertices) = vertices.into_inner().vertex.buffer() else {
            return RenderCommandResult::Failure("cuttle sdf vertices buffer not available");
        };
        let Some(indices) = pipeline.indices.buffer() else {
            return RenderCommandResult::Failure("cuttle sdf indices buffer not available");
        };
        let Some(comp_bind_group) = &comp_buffers.into_inner().0 else {
            return RenderCommandResult::Failure("bind_group not found for key");
        };
        let Some(op_bind_group) = &op_buffers.into_inner().0 else {
            return RenderCommandResult::Failure("bind_group not found for key");
        };

        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.set_bind_group(1, op_bind_group, &[]);
        pass.set_bind_group(2, comp_bind_group, &[]);
        pass.set_index_buffer(indices.slice(..), 0, IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, instance.range.clone());
        // info!("DRAW {:?}", instance.range);
        RenderCommandResult::Success
    }
}
