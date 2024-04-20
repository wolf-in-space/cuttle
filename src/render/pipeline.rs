use crate::flag::SdfPipelineKey;
use bevy_asset::Handle;
use bevy_ecs::{
    prelude::*,
    system::Resource,
    world::{FromWorld, World},
};
use bevy_render::{
    render_resource::{
        binding_types::uniform_buffer, BindGroup, BindGroupLayout, BindGroupLayoutEntries,
        BlendState, BufferUsages, BufferVec, ColorTargetState, ColorWrites, FragmentState,
        FrontFace, MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology,
        RenderPipelineDescriptor, Shader, ShaderStages, SpecializedRenderPipeline, TextureFormat,
        VertexBufferLayout, VertexState,
    },
    renderer::RenderDevice,
    texture::BevyDefault,
    view::ViewUniform,
};
use bevy_utils::hashbrown::HashMap;

#[derive(Event, Debug, PartialEq, Clone)]
pub struct SdfSpecializationData {
    pub vertex_layout: VertexBufferLayout,
    pub shader: Handle<Shader>,
    pub bind_group_layout: BindGroupLayout,
}

#[derive(Resource)]
pub struct SdfPipeline {
    pub view_layout: BindGroupLayout,
    pub bind_group_buffers: Vec<BufferVec<u8>>,
    pub bind_groups: HashMap<SdfPipelineKey, BindGroup>,
    pub indices: BufferVec<u16>,
    pub specialization: HashMap<SdfPipelineKey, SdfSpecializationData>,
}

impl FromWorld for SdfPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let mut indices = BufferVec::new(BufferUsages::INDEX);
        indices.values_mut().append(&mut vec![2, 0, 1, 1, 3, 2]);

        let view_layout = render_device.create_bind_group_layout(
            "sdf_pipeline_view_uniform_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        SdfPipeline {
            bind_group_buffers: Vec::new(),
            indices,
            view_layout,
            bind_groups: HashMap::new(),
            specialization: HashMap::new(),
        }
    }
}

impl SpecializedRenderPipeline for SdfPipeline {
    type Key = SdfPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let Some(SdfSpecializationData {
            shader,
            bind_group_layout,
            vertex_layout,
            ..
        }) = self.specialization.get(&key)
        else {
            panic!("Specialize data not loaded into pipeline for key {key:?}");
        };

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![vertex_layout.clone()],
            },
            fragment: Some(FragmentState {
                shader: shader.clone(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![self.view_layout.clone(), bind_group_layout.clone()],
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some(format!("SdfPipeline for Sdf '{key:?}'").into()),
            push_constant_ranges: Vec::new(),
        }
    }
}
