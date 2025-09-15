use super::{CuttlePipelineKey, queue::ConfigInstanceBuffer};
use crate::components::buffer::{build_buffer_layout, build_comp_layout, build_global_layouts};
use crate::configs::{ConfigId, CuttleConfig};
use crate::internal_prelude::*;
use crate::shader::CuttleShader;
use bevy_asset::{AssetServer, Handle};
use bevy_core_pipeline::core_2d::CORE_2D_DEPTH_FORMAT;
use bevy_ecs::system::RunSystemOnce;
use bevy_image::BevyDefault;
use bevy_mesh::VertexBufferLayout;
use bevy_render::RenderApp;
use bevy_render::render_resource::binding_types::uniform_buffer;
use bevy_render::render_resource::{
    BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BlendState, BufferUsages,
    ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
    FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology,
    RawBufferVec, RenderPipelineDescriptor, ShaderStages, SpecializedRenderPipeline,
    StencilFaceState, StencilState, TextureFormat, VertexFormat, VertexState, VertexStepMode,
};
use bevy_render::renderer::{RenderDevice, RenderQueue};
use bevy_render::view::{ExtractedView, ViewUniform, ViewUniforms};
use bevy_shader::Shader;
use std::collections::HashMap;

#[derive(Resource)]
pub struct CuttlePipeline {
    pub _common_shader: Handle<Shader>,
    pub vertex_shader: Handle<Shader>,
    pub fragment_shaders: HashMap<ConfigId, Handle<Shader>>,
    pub view_layout: BindGroupLayout, // group 0
    pub op_layout: BindGroupLayout,   // group 1
    pub comp_layout: BindGroupLayout, // group 2
    pub global_layouts: HashMap<ConfigId, BindGroupLayout>, // group 3
    pub indices: RawBufferVec<u16>,
}

impl CuttlePipeline {
    pub fn init(app: &mut App) {
        let fragment_shaders = app
            .world_mut()
            .run_system_once(|shaders: Query<(&ConfigId, &CuttleShader)>| {
                shaders
                    .iter()
                    .map(|(id, shader)| (*id, shader.0.clone()))
                    .collect::<HashMap<_, _>>()
            })
            .unwrap();
        let world = app.sub_app_mut(RenderApp).world_mut();

        let device = world.resource::<RenderDevice>();
        let queue = world.resource::<RenderQueue>();

        let mut indices = RawBufferVec::new(BufferUsages::INDEX);
        *indices.values_mut() = vec![2, 0, 1, 1, 3, 2];
        indices.write_buffer(device, queue);

        let view_layout = device.create_bind_group_layout(
            "cuttle pipeline view uniform layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (uniform_buffer::<ViewUniform>(true),),
            ),
        );

        let op_layout = build_buffer_layout(1, device, "cuttle index buffers layout");
        let comp_layout = world.run_system_once(build_comp_layout).unwrap();
        let global_layouts = world.run_system_once(build_global_layouts).unwrap();

        let asset_server = world.resource_mut::<AssetServer>();
        let _common_shader =
            asset_server.load::<Shader>("embedded://cuttle_core/shader/common.wgsl");
        let vertex_shader =
            asset_server.load::<Shader>("embedded://cuttle_core/shader/vertex.wgsl");

        let pipeline = CuttlePipeline {
            indices,
            _common_shader,
            vertex_shader,
            fragment_shaders,
            view_layout,
            op_layout,
            comp_layout,
            global_layouts,
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
                entry_point: Some("vertex".into()),
                shader_defs: Vec::new(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: self.fragment_shaders[&key.group_id].clone(),
                shader_defs: Vec::new(),
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![
                self.view_layout.clone(),
                self.op_layout.clone(),
                self.comp_layout.clone(),
                self.global_layouts.get(&key.group_id).unwrap().clone(),
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
            &pipeline.view_layout,
            &BindGroupEntries::single(view_binding.clone()),
        );

        commands.entity(entity).insert(CuttleViewBindGroup {
            value: view_bind_group,
        });
    }
}

pub fn write_group_buffer<Config: CuttleConfig>(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut buffers: ResMut<ConfigInstanceBuffer<Config>>,
) {
    buffers.vertex.write_buffer(&device, &queue);
}
