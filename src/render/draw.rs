use super::{
    pipeline::SdfPipeline,
    process::{SdfBindGroups, SdfInstance, SdfViewBindGroup},
};
use bevy::{
    ecs::system::{
        lifetimeless::{Read, SRes},
        SystemParamItem,
    },
    log::error,
    render::{
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::IndexFormat,
        view::ViewUniformOffset,
    },
};

pub type DrawSdf = (SetItemPipeline, SetSdfViewBindGroup, DrawSdfDispatch);

pub struct SetSdfViewBindGroup;
impl<P: PhaseItem> RenderCommand<P> for SetSdfViewBindGroup {
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
impl<P: PhaseItem> RenderCommand<P> for DrawSdfDispatch {
    type Param = (SRes<SdfPipeline>, SRes<SdfBindGroups>);
    type ViewQuery = ();
    type ItemQuery = Read<SdfInstance>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        sdf_instance: Option<&'w SdfInstance>,
        (pipeline, variants): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(instance) = sdf_instance else {
            error!("Cancelled draw: 'item not found'");
            return RenderCommandResult::Failure;
        };
        let Some(vertices) = instance.vertex_buffer.buffer() else {
            error!("Cancelled draw: 'bevy_comdf sdf vertices buffer not available'");
            return RenderCommandResult::Failure;
        };
        let Some(indices) = pipeline.into_inner().indices.buffer() else {
            error!("Cancelled draw: 'bevy_comdf sdf indices buffer not available'");
            return RenderCommandResult::Failure;
        };
        let Some(bind_group) = variants.into_inner().0.get(&instance.key) else {
            error!(
                "Cancelled draw: 'bind_group not found for key {:?}'",
                instance.key
            );
            return RenderCommandResult::Failure;
        };

        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.set_bind_group(1, bind_group, &[]);
        pass.set_index_buffer(indices.slice(..), 0, IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..1);
        RenderCommandResult::Success
    }
}
