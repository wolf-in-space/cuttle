use super::{queue::RenderPhaseBuffers, RenderPhase, SdfPipelineKey, UsePipeline};
use crate::{
    components::extract::SdfBuffers,
    flag::SdfFlags,
    shader::{bindgroups::bind_group, NewShader},
};
use bevy::{
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_resource::{
            binding_types::{storage_buffer_read_only, uniform_buffer},
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BlendState,
            BufferUsages, ColorTargetState, ColorWrites, FragmentState, FrontFace,
            MultisampleState, PolygonMode, PrimitiveState, RawBufferVec, RenderPipelineDescriptor,
            ShaderStages, SpecializedRenderPipeline, TextureFormat, VertexBufferLayout,
            VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedView, ViewUniform, ViewUniforms},
    },
    utils::HashMap,
};
use itertools::Itertools;
use std::marker::PhantomData;

#[derive(Event, Debug, PartialEq, Clone)]
pub struct SdfSpecializationData {
    pub shader: Handle<Shader>,
    pub bind_group_layout: BindGroupLayout,
    pub bindings: Vec<usize>,
}

pub(crate) fn redo_bindgroups(
    device: Res<RenderDevice>,
    mut pipeline: ResMut<SdfPipeline>,
    buffers: Res<SdfBuffers>,
) {
    let keys = pipeline.bind_groups.keys().cloned().collect_vec();
    for key in keys {
        let specialization = &pipeline.specialization[&key];
        let bindings_buffers = bindings_to_bindrgoup_entries(&specialization.bindings, &buffers);

        let (_, new_bindgroup) = bind_group(&bindings_buffers, &device);
        let bindgroup = pipeline.bind_groups.get_mut(&key).unwrap();
        *bindgroup = new_bindgroup;
    }
}

fn bindings_to_bindrgoup_entries<'a>(
    bindings: &[usize],
    buffers: &'a SdfBuffers,
) -> Vec<(u32, &'a RawBufferVec<u8>)> {
    bindings
        .iter()
        .copied()
        .filter(|b| !buffers[*b].buffer.is_empty())
        .sorted()
        .dedup()
        .map(|b| (b as u32, &buffers[b].buffer))
        .collect_vec()
}

pub(crate) fn add_new_sdf_to_pipeline(
    mut new_shaders: EventReader<NewShader>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut pipeline: ResMut<SdfPipeline>,
    mut buffers: ResMut<SdfBuffers>,
) {
    for new in new_shaders.read() {
        for i in new.bindings.iter() {
            buffers[*i].buffer.write_buffer(&device, &queue)
        }
        let bindings_buffers = bindings_to_bindrgoup_entries(&new.bindings, &buffers);

        let (bind_group_layout, bind_group) = bind_group(&bindings_buffers, &device);
        pipeline.specialization.insert(
            new.flags.clone(),
            SdfSpecializationData {
                bindings: new.bindings.clone(),
                shader: new.shader.clone(),
                bind_group_layout,
            },
        );
        pipeline.bind_groups.insert(new.flags.clone(), bind_group);
    }
}

#[derive(Resource)]
pub struct SdfPipeline {
    pub global_layout: BindGroupLayout,
    pub bind_groups: HashMap<SdfFlags, BindGroup>,
    pub indices: RawBufferVec<u16>,
    pub specialization: HashMap<SdfFlags, SdfSpecializationData>,
}

impl FromWorld for SdfPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let queue = world.resource::<RenderQueue>();

        let mut indices = RawBufferVec::new(BufferUsages::INDEX);
        *indices.values_mut() = vec![2, 0, 1, 1, 3, 2];
        indices.write_buffer(device, queue);

        let view_layout = device.create_bind_group_layout(
            "sdf_pipeline_view_uniform_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    uniform_buffer::<ViewUniform>(true),
                    storage_buffer_read_only::<u32>(false),
                ),
            ),
        );

        SdfPipeline {
            indices,
            global_layout: view_layout,
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
            ..
        }) = self.specialization.get(&key.flags)
        else {
            panic!("Specialize data not loaded into sdf pipeline");
        };

        let multisample_count = match key.pipeline {
            UsePipeline::World => 4,
            UsePipeline::Ui => 1,
        };

        let vertex_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Instance,
            [
                VertexFormat::Float32x2,
                VertexFormat::Float32x2,
                VertexFormat::Uint32,
            ],
        );

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![vertex_layout],
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
            layout: vec![self.global_layout.clone(), bind_group_layout.clone()],
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
                count: multisample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some(format!("SdfPipeline for Sdf '{key:?}'").into()),
            push_constant_ranges: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct SdfViewBindGroup<P: RenderPhase> {
    pub value: BindGroup,
    marker: PhantomData<P>,
}

pub fn prepare_view_bind_groups<P: RenderPhase>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
    phase_buffers: Res<RenderPhaseBuffers<P>>,
    pipeline: Res<SdfPipeline>,
    views: Query<Entity, With<ExtractedView>>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    let Some(indices_binding) = phase_buffers.indices.binding() else {
        return;
    };

    for entity in &views {
        let view_bind_group = render_device.create_bind_group(
            "sdf_view_bind_group",
            &pipeline.global_layout,
            &BindGroupEntries::sequential((view_binding.clone(), indices_binding.clone())),
        );

        commands.entity(entity).insert(SdfViewBindGroup::<P> {
            value: view_bind_group,
            marker: PhantomData,
        });
    }
}

pub fn write_comp_buffers(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut buffers: ResMut<SdfBuffers>,
) {
    buffers.iter_mut().for_each(|buffer| {
        buffer.buffer.write_buffer(&device, &queue);
    });
}

pub fn write_phase_buffers<P: RenderPhase>(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut buffers: ResMut<RenderPhaseBuffers<P>>,
) {
    buffers.vertex.write_buffer(&device, &queue);
    buffers.indices.write_buffer(&device, &queue);
}
