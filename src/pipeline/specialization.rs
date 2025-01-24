use super::{queue::GroupInstanceBuffer, CuttlePipelineKey, SortedCuttlePhaseItem};
use crate::components::buffer::build_buffer_layout;
use crate::groups::global::GlobalGroupInfos;
use crate::groups::GroupId;
use bevy::image::BevyDefault;
use bevy::render::RenderApp;
use bevy::utils::HashMap;
use bevy::{
    core_pipeline::core_2d::CORE_2D_DEPTH_FORMAT,
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_resource::{
            binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, BlendState, BufferUsages, ColorTargetState, ColorWrites,
            CompareFunction, DepthBiasState, DepthStencilState, FragmentState, FrontFace,
            MultisampleState, PolygonMode, PrimitiveState, RawBufferVec, RenderPipelineDescriptor,
            ShaderStages, SpecializedRenderPipeline, StencilFaceState, StencilState, TextureFormat,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, ViewUniform, ViewUniforms},
    },
};

#[derive(Resource)]
pub struct CuttlePipeline {
    pub _common_shader: Handle<Shader>,
    pub vertex_shader: Handle<Shader>,
    pub fragment_shaders: HashMap<GroupId, Handle<Shader>>,
    pub global_layout: BindGroupLayout,
    pub op_layout: BindGroupLayout,
    pub comp_layout: BindGroupLayout,
    pub indices: RawBufferVec<u16>,
}

impl CuttlePipeline {
    pub fn init(app: &mut App, fragment_shaders: HashMap<GroupId, Handle<Shader>>) {
        let comp_count = app
            .world()
            .resource::<GlobalGroupInfos>()
            .component_bindings
            .len() as u32;

        let world = app.sub_app_mut(RenderApp).world_mut();

        let device = world.resource::<RenderDevice>();
        let queue = world.resource::<RenderQueue>();

        let mut indices = RawBufferVec::new(BufferUsages::INDEX);
        *indices.values_mut() = vec![2, 0, 1, 1, 3, 2];
        indices.write_buffer(device, queue);

        let global_layout = device.create_bind_group_layout(
            "cuttle pipeline view uniform layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (uniform_buffer::<ViewUniform>(true),),
            ),
        );

        let comp_layout =
            build_buffer_layout(comp_count, device, "cuttle component buffers layout");
        let op_layout = build_buffer_layout(1, device, "cuttle index buffers layout");

        let asset_server = world.resource_mut::<AssetServer>();
        let _common_shader = asset_server.load::<Shader>("embedded://cuttle/shader/common.wgsl");
        let vertex_shader = asset_server.load::<Shader>("embedded://cuttle/shader/vertex.wgsl");

        let pipeline = CuttlePipeline {
            indices,
            global_layout,
            _common_shader,
            vertex_shader,
            fragment_shaders,
            comp_layout,
            op_layout,
        };

        world.insert_resource(pipeline);
    }
}

impl SpecializedRenderPipeline for CuttlePipeline {
    type Key = CuttlePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let depth_stencil = if key.has_depth {
            Some(DepthStencilState {
                format: CORE_2D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            })
        } else {
            None
        };

        let vertex_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Instance,
            [
                VertexFormat::Float32x2,
                VertexFormat::Float32,
                VertexFormat::Uint32,
                VertexFormat::Uint32,
            ],
        );

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: self.vertex_shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: self.fragment_shaders[&key.group_id].clone(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![
                self.global_layout.clone(),
                self.op_layout.clone(),
                self.comp_layout.clone(),
            ],
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil,
            multisample: MultisampleState {
                count: key.multisample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some(format!("CuttlePipeline for Key '{key:?}'").into()),
            push_constant_ranges: Vec::new(),
            zero_initialize_workgroup_memory: true,
        }
    }
}

#[derive(Component)]
pub struct CuttleViewBindGroup {
    pub value: BindGroup,
}

pub fn prepare_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
    pipeline: Res<CuttlePipeline>,
    views: Query<Entity, With<ExtractedView>>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    for entity in &views {
        let view_bind_group = render_device.create_bind_group(
            "cuttle_view_bind_group",
            &pipeline.global_layout,
            &BindGroupEntries::single(view_binding.clone()),
        );

        commands.entity(entity).insert(CuttleViewBindGroup {
            value: view_bind_group,
        });
    }
}

pub fn write_group_buffer<P: SortedCuttlePhaseItem>(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut buffers: ResMut<GroupInstanceBuffer<P>>,
) {
    buffers.vertex.write_buffer(&device, &queue);
}
