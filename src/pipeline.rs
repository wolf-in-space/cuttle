use crate::components::extract::{SdfBindings, SdfBufferIndex, SdfBuffers};
use crate::flag::SdfFlags;
use crate::shader::bindgroups::bind_group;
use crate::shader::NewShader;
use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::ecs::system::lifetimeless::{Read, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::math::FloatOrd;
use bevy::prelude::*;
use bevy::render::render_phase::{PhaseItemExtraIndex, ViewSortedRenderPhases};
use bevy::render::render_resource::{VertexFormat, VertexStepMode};
use bevy::render::renderer::RenderQueue;
use bevy::render::{
    render_phase::{
        AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
        SetItemPipeline, TrackedRenderPass,
    },
    render_resource::{
        binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayout,
        BindGroupLayoutEntries, BlendState, BufferUsages, ColorTargetState, ColorWrites,
        FragmentState, FrontFace, IndexFormat, MultisampleState, PipelineCache, PolygonMode,
        PrimitiveState, PrimitiveTopology, RawBufferVec, RenderPipelineDescriptor, Shader,
        ShaderStages, SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat,
        VertexBufferLayout, VertexState,
    },
    renderer::RenderDevice,
    texture::BevyDefault,
    view::{ExtractedView, ViewUniform, ViewUniformOffset, ViewUniforms},
    Extract, Render, RenderApp, RenderSet,
};
use bevy::utils::hashbrown::HashMap;
use bevy_comdf_core::aabb::AABB;
use bytemuck::{bytes_of, Pod, Zeroable};
use itertools::Itertools;

pub struct SdfPipelinePlugin;
impl Plugin for SdfPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<SpecializedRenderPipelines<SdfPipeline>>()
            .init_resource::<ExtractedSdfs>()
            .add_event::<SdfSpecializationData>()
            .add_render_command::<Transparent2d, DrawSdf>()
            .add_systems(ExtractSchedule, extract_render_sdf)
            .add_systems(
                Render,
                (
                    (
                        (sort_sdfs_into_batches, add_new_sdf_to_pipeline),
                        queue_sdfs,
                        write_buffers,
                    )
                        .chain(),
                    prepare_view_bind_groups,
                )
                    .after(RenderSet::ExtractCommands)
                    .before(RenderSet::Render),
            );
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<SdfPipeline>();
        }
    }
}

#[derive(Debug)]
struct ExtractedSdf {
    index: u32,
    aabb: AABB,
    key: SdfPipelineKey,
}

#[derive(Resource, Default, Deref, DerefMut)]
struct ExtractedSdfs(Vec<ExtractedSdf>);

fn extract_render_sdf(
    query: Extract<Query<(&SdfBufferIndex, &AABB, &SdfFlags)>>,
    mut extracted: ResMut<ExtractedSdfs>,
) {
    extracted.0 = query
        .into_iter()
        .map(|(index, aabb, flags)| ExtractedSdf {
            key: SdfPipelineKey {
                flags: flags.clone(),
            },
            aabb: aabb.clone(),
            index: index.0 as u32,
        })
        .collect();
}

#[derive(Component)]
pub struct SdfBatch {
    pub instance_count: u32,
    pub key: SdfPipelineKey,
    pub vertex_buffer: RawBufferVec<u8>,
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SdfInstance {
    size: Vec2,
    position: Vec2,
    index: u32,
}

fn sort_sdfs_into_batches(
    mut cmds: Commands,
    mut sdfs: ResMut<ExtractedSdfs>,
    bindings: Res<SdfBindings>,
) {
    let batches = sdfs
        .drain(..)
        .sorted_unstable_by(|sdf1, sdf2| sdf1.key.cmp(&sdf2.key).then(sdf1.index.cmp(&sdf2.index)))
        .chunk_by(|sdf| sdf.key.clone())
        .into_iter()
        .map(|(key, sdfs)| {
            let mut count = 0;
            let mut vertex_buffer = RawBufferVec::new(BufferUsages::VERTEX);
            let buffer = vertex_buffer.values_mut();
            for sdf in sdfs {
                count += 1;
                buffer.extend_from_slice(bytes_of(&sdf.aabb.size()));
                buffer.extend_from_slice(bytes_of(&sdf.aabb.pos()));
                for (_, flag) in key.flags.iter() {
                    buffer.extend_from_slice(bytes_of(&bindings[flag]))
                }
            }

            SdfBatch {
                vertex_buffer,
                instance_count: count,
                key: key.clone(),
            }
        })
        .collect_vec();

    cmds.spawn_batch(batches);
}

#[allow(clippy::too_many_arguments)]
pub fn queue_sdfs(
    sdfs: Query<(Entity, &SdfBatch)>,
    views: Query<Entity, With<ExtractedView>>,
    sdf_pipeline: Res<SdfPipeline>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SdfPipeline>>,
    cache: Res<PipelineCache>,
    mut render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    view_uniforms: Res<ViewUniforms>,
) {
    if view_uniforms.uniforms.binding().is_none() {
        return;
    }
    let draw_function = draw_functions.read().id::<DrawSdf>();
    for view_entity in views.into_iter() {
        for (entity, sdf) in sdfs.into_iter() {
            let Some(render_phase) = render_phases.get_mut(&view_entity) else {
                //warn!("Renderphase not found for queue sdfs");
                continue;
            };
            // println!("QUEUE");
            let pipeline = pipelines.specialize(&cache, &sdf_pipeline, sdf.key.clone());
            render_phase.add(Transparent2d {
                sort_key: FloatOrd(1.0),
                entity,
                pipeline,
                draw_function,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex(0),
            });
        }
    }
}

#[derive(Event, Debug, PartialEq, Clone)]
pub struct SdfSpecializationData {
    pub vertex_layout: VertexBufferLayout,
    pub shader: Handle<Shader>,
    pub bind_group_layout: BindGroupLayout,
}

fn add_new_sdf_to_pipeline(
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
        let bindings_buffers = new
            .bindings
            .iter()
            .filter(|b| !buffers[**b].buffer.is_empty())
            .map(|b| (*b as u32, &buffers[*b].buffer))
            .collect_vec();
        let key = SdfPipelineKey {
            flags: new.flags.clone(),
        };
        let (bind_group_layout, bind_group) = bind_group(&bindings_buffers, &device);
        pipeline.specialization.insert(
            key.clone(),
            SdfSpecializationData {
                vertex_layout: VertexBufferLayout::from_vertex_formats(
                    VertexStepMode::Instance,
                    [VertexFormat::Float32x2, VertexFormat::Float32x2]
                        .into_iter()
                        .chain((0..(new.bindings.len())).map(|_| VertexFormat::Uint32)),
                ),
                shader: new.shader.clone(),
                bind_group_layout,
            },
        );
        pipeline.bind_groups.insert(key, bind_group);
    }
}

#[derive(Debug, Component, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct SdfPipelineKey {
    flags: SdfFlags,
}

#[derive(Resource)]
pub struct SdfPipeline {
    pub view_layout: BindGroupLayout,
    pub bind_groups: HashMap<SdfPipelineKey, BindGroup>,
    pub indices: RawBufferVec<u16>,
    pub specialization: HashMap<SdfPipelineKey, SdfSpecializationData>,
}

impl FromWorld for SdfPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let mut indices = RawBufferVec::new(BufferUsages::INDEX);
        indices.values_mut().append(&mut vec![2, 0, 1, 1, 3, 2]);

        let view_layout = render_device.create_bind_group_layout(
            "sdf_pipeline_view_uniform_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        SdfPipeline {
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
            panic!("Specialize data not loaded into sdf pipeline for key {key:?}");
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

#[derive(Component)]
pub struct SdfViewBindGroup {
    pub value: BindGroup,
}

pub fn prepare_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<SdfPipeline>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity, With<ExtractedView>>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    for entity in &views {
        let view_bind_group = render_device.create_bind_group(
            "sdf_view_bind_group",
            &pipeline.view_layout,
            &BindGroupEntries::single(view_binding.clone()),
        );

        commands.entity(entity).insert(SdfViewBindGroup {
            value: view_bind_group,
        });
    }
}

pub fn write_buffers(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut buffers: ResMut<SdfBuffers>,
    mut pipeline: ResMut<SdfPipeline>,
    mut instances: Query<&mut SdfBatch>,
) {
    pipeline.indices.write_buffer(&device, &queue);
    buffers.iter_mut().for_each(|buffer| {
        buffer.buffer.write_buffer(&device, &queue);
    });
    instances.iter_mut().for_each(|mut instance| {
        instance.vertex_buffer.write_buffer(&device, &queue);
    });
}

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
    type Param = SRes<SdfPipeline>;
    type ViewQuery = ();
    type ItemQuery = Read<SdfBatch>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        sdf_instance: Option<&'w SdfBatch>,
        pipeline: SystemParamItem<'w, '_, Self::Param>,
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
        let pipeline = pipeline.into_inner();
        let Some(indices) = pipeline.indices.buffer() else {
            error!("Cancelled draw: 'bevy_comdf sdf indices buffer not available'");
            return RenderCommandResult::Failure;
        };
        let Some(bind_group) = pipeline.bind_groups.get(&instance.key) else {
            error!(
                "Cancelled draw: 'bind_group not found for key {:?}'",
                instance.key
            );
            return RenderCommandResult::Failure;
        };

        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.set_bind_group(1, bind_group, &[]);
        pass.set_index_buffer(indices.slice(..), 0, IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..instance.instance_count);
        RenderCommandResult::Success
    }
}
