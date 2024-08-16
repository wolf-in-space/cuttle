use crate::aabb::CombinedAABB;
use crate::components::extract::{SdfBufferIndices, SdfBuffers};
use crate::flag::SdfFlags;
use crate::shader::bindgroups::bind_group;
use crate::shader::NewShader;
use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::system::lifetimeless::{Read, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::math::FloatOrd;
use bevy::prelude::*;
use bevy::render::{
    render_phase::{
        AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
        RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
    },
    render_resource::{
        binding_types::{storage_buffer_read_only, uniform_buffer},
        BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BlendState,
        BufferUsages, ColorTargetState, ColorWrites, FragmentState, FrontFace, IndexFormat,
        MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
        RawBufferVec, RenderPipelineDescriptor, Shader, ShaderStages, ShaderType,
        SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, VertexBufferLayout,
        VertexFormat, VertexState, VertexStepMode,
    },
    renderer::{RenderDevice, RenderQueue},
    texture::BevyDefault,
    view::{ExtractedView, ViewUniform, ViewUniformOffset, ViewUniforms},
    Extract, Render, RenderApp, RenderSet,
};
use bevy::utils::HashMap;
use bevy_comdf_core::aabb::AABB;
use bytemuck::NoUninit;
use itertools::Itertools;
use std::ops::Range;

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
                ((
                    add_new_sdf_to_pipeline,
                    queue_sdfs,
                    prepare_sdfs,
                    write_buffers,
                    (
                        redo_bindgroups,
                        prepare_view_bind_groups.after(RenderSet::PrepareBindGroups),
                    ),
                )
                    .chain(),)
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
    indices: Vec<u32>,
    aabb: AABB,
    key: SdfPipelineKey,
    sort: f32,
}

#[derive(Resource, Default, Deref, DerefMut)]
struct ExtractedSdfs(EntityHashMap<ExtractedSdf>);

fn extract_render_sdf(
    query: Extract<
        Query<(
            Entity,
            &SdfBufferIndices,
            &CombinedAABB,
            &SdfFlags,
            &GlobalTransform,
        )>,
    >,
    mut extracted: ResMut<ExtractedSdfs>,
) {
    extracted.0 = query
        .into_iter()
        .map(|(entity, indices, aabb, flags, tranform)| {
            (
                entity,
                ExtractedSdf {
                    key: SdfPipelineKey {
                        flags: flags.clone(),
                    },
                    aabb: aabb.0.clone(),
                    indices: indices.0.clone(),
                    sort: tranform.translation().z,
                },
            )
        })
        .collect();
}

fn queue_sdfs(
    sdfs: Res<ExtractedSdfs>,
    views: Query<Entity, With<ExtractedView>>,
    sdf_pipeline: Res<SdfPipeline>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SdfPipeline>>,
    cache: Res<PipelineCache>,
    mut render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
) {
    let draw_function = draw_functions.read().id::<DrawSdf>();
    for view_entity in views.into_iter() {
        let Some(render_phase) = render_phases.get_mut(&view_entity) else {
            continue;
        };
        for (&entity, sdf) in sdfs.iter() {
            let pipeline = pipelines.specialize(&cache, &sdf_pipeline, sdf.key.clone());
            render_phase.add(Transparent2d {
                sort_key: FloatOrd(sdf.sort),
                entity,
                pipeline,
                draw_function,
                batch_range: 0..0,
                extra_index: PhaseItemExtraIndex::NONE,
            });
        }
    }
}

#[derive(Component, Debug)]
pub struct SdfBatch {
    range: Range<u32>,
    key: SdfPipelineKey,
}

#[derive(Debug, ShaderType, NoUninit, Clone, Copy)]
#[repr(C)]
pub struct SdfInstance {
    size: Vec2,
    pos: Vec2,
    indices_start: u32,
}

fn prepare_sdfs(
    mut cmds: Commands,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    mut pipeline: ResMut<SdfPipeline>,
    sdfs: Res<ExtractedSdfs>,
) {
    let mut batches = Vec::new();
    // let mut vertex_buffer = Vec::new();
    pipeline.sdf_data_indices.clear();
    pipeline.vertex_buffer.clear();

    println!("AAAAAAAAAA");
    for transparent_phase in phases.values_mut() {
        println!("HASIDUGH");
        let mut batch_index = 0;
        let mut batch_key = None;

        for index in 0..transparent_phase.items.len() {
            let item = &transparent_phase.items[index];
            let Some(sdf) = sdfs.get(&item.entity) else {
                batch_key = None;
                continue;
            };

            if batch_key != Some(&sdf.key) {
                batch_index = index;
                batch_key = Some(&sdf.key);
                let index = index as u32;
                batches.push((
                    item.entity,
                    SdfBatch {
                        key: sdf.key.clone(),
                        range: index..index,
                    },
                ));
            }

            let indices_start = pipeline.sdf_data_indices.len() as u32;
            let instance = SdfInstance {
                size: sdf.aabb.size(),
                pos: sdf.aabb.pos(),
                indices_start,
            };

            // println!(
            //     "i={index}, b={batch_index}, indices={:?}, \ninstance={instance:?}, \nsdf={sdf:?}",
            //     sdf.indices
            // );

            pipeline.vertex_buffer.push(instance);

            pipeline
                .sdf_data_indices
                .extend(sdf.indices.iter().copied());

            // pipeline.sdf_data_indices.push(0);

            transparent_phase.items[batch_index].batch_range_mut().end += 1;
            batches.last_mut().unwrap().1.range.end += 1;
        }
    }

    // dbg!(&batches);
    // dbg!(vertex_buffer);
    // dbg!(pipeline.sdf_data_indices.values());
    cmds.insert_or_spawn_batch(batches);
}

#[derive(Event, Debug, PartialEq, Clone)]
pub struct SdfSpecializationData {
    pub shader: Handle<Shader>,
    pub bind_group_layout: BindGroupLayout,
    pub bindings: Vec<usize>,
}

fn redo_bindgroups(
    device: Res<RenderDevice>,
    mut pipeline: ResMut<SdfPipeline>,
    buffers: Res<SdfBuffers>,
) {
    let keys = pipeline.bind_groups.keys().cloned().collect_vec();
    for key in keys {
        let specialization = &pipeline.specialization[&key];
        let bindings_buffers = specialization
            .bindings
            .iter()
            .filter(|b| !buffers[**b].buffer.is_empty())
            .map(|b| (*b as u32, &buffers[*b].buffer))
            .collect_vec();
        let (_, new_bindgroup) = bind_group(&bindings_buffers, &device);
        let bindgroup = pipeline.bind_groups.get_mut(&key).unwrap();
        *bindgroup = new_bindgroup;
    }
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
                bindings: new.bindings.clone(),
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
    pub global_layout: BindGroupLayout,
    pub bind_groups: HashMap<SdfPipelineKey, BindGroup>,
    pub indices: RawBufferVec<u16>,
    pub specialization: HashMap<SdfPipelineKey, SdfSpecializationData>,
    pub vertex_buffer: RawBufferVec<SdfInstance>,
    pub sdf_data_indices: RawBufferVec<u32>,
}

impl FromWorld for SdfPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let mut indices = RawBufferVec::new(BufferUsages::INDEX);
        *indices.values_mut() = vec![2, 0, 1, 1, 3, 2];

        let view_layout = render_device.create_bind_group_layout(
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
            vertex_buffer: RawBufferVec::new(BufferUsages::VERTEX),
            sdf_data_indices: RawBufferVec::new(BufferUsages::STORAGE),
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
        }) = self.specialization.get(&key)
        else {
            panic!("Specialize data not loaded into sdf pipeline for key {key:?}");
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

    let Some(indices_binding) = pipeline.sdf_data_indices.binding() else {
        return;
    };

    for entity in &views {
        let view_bind_group = render_device.create_bind_group(
            "sdf_view_bind_group",
            &pipeline.global_layout,
            &BindGroupEntries::sequential((view_binding.clone(), indices_binding.clone())),
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
) {
    pipeline.indices.write_buffer(&device, &queue);
    pipeline.sdf_data_indices.write_buffer(&device, &queue);
    pipeline.vertex_buffer.write_buffer(&device, &queue);
    buffers.iter_mut().for_each(|buffer| {
        buffer.buffer.write_buffer(&device, &queue);
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
            // error!("Cancelled draw: 'item not found'");
            return RenderCommandResult::Failure;
        };
        let pipeline = pipeline.into_inner();
        let Some(vertices) = pipeline.vertex_buffer.buffer() else {
            error!("Cancelled draw: 'bevy_comdf sdf vertices buffer not available'");
            return RenderCommandResult::Failure;
        };
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
        pass.draw_indexed(0..6, 0, instance.range.clone());
        // info!("DRAW {:?}", instance.range);
        RenderCommandResult::Success
    }
}
