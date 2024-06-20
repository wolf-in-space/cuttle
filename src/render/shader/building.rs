use super::{lines::Lines, variants::SdfCalculationBuilder};
use crate::components::RenderSdfFlag;
use crate::{linefy, operations::OperationsFlag};
use bevy::ecs::component::Component;
use bevy::render::render_resource::{Shader, VertexBufferLayout, VertexFormat, VertexStepMode};
use bevy::utils::{default, HashMap, HashSet};
use itertools::Itertools;
use std::{fs::File, io::Write};

#[derive(Default, Clone, Component)]
pub struct SdfShaderBuilder {
    variants: HashMap<u32, SdfCalculationBuilder>,
    binding: u32,
    //
    variants_extras: HashMap<RenderSdfFlag, Lines>,
    operation_snippets: HashSet<OperationsFlag>,
}

impl SdfShaderBuilder {
    pub fn new(binding: u32) -> Self {
        Self {
            binding,
            ..default()
        }
    }

    pub fn add_sdf_calculation(&mut self, calc: SdfCalculationBuilder) {
        self.variants_extras.extend(calc.extra.clone());
        self.operation_snippets
            .extend(calc.operation_snippets.clone());
        self.variants.insert(calc.binding, calc);
    }

    pub fn to_shader(
        &self,
        key: &SdfPipelineKey,
        snippets: &HashMap<OperationsFlag, Lines>,
    ) -> Shader {
        let code = self.gen_shader_code(snippets);

        let filepath = format!("assets/sdf_shaders/{:?}.wgsl", key);
        let filepath = filepath.replace("RenderableSdf", "");
        let filepath = filepath.replace("SdfPipelineKey", "sdf");
        let filepath = filepath.replace("Smooth", "S");
        let filepath = filepath.replace(']', "");
        let filepath = filepath.replace('[', "");
        let filepath = filepath.replace('(', "");
        let filepath = filepath.replace(')', "");
        let filepath = filepath.replace(',', "");
        let mut file = File::create(filepath).unwrap();
        file.write_all(code.as_bytes()).unwrap();
        file.flush().unwrap();

        // println!("{}", code);
        Shader::from_wgsl(
            code,
            format!("Generated in '{}' for sdf {:?}", file!(), key),
        )
    }

    fn gen_shader_code(&self, snippets: &HashMap<OperationsFlag, Lines>) -> String {
        Lines::from([
            self.gen_structs(),
            self.gen_extra(),
            self.gen_snippets(snippets),
            self.gen_sdf_calcs(),
            self.gen_vertex_shader(),
            self.gen_fragment_shader(),
        ])
        .into_file_str()
    }

    fn gen_sdf_calcs(&self) -> Lines {
        self.variants
            .values()
            .map(|c| c.build())
            .collect_vec()
            .into()
    }

    fn gen_snippets(&self, snippets: &HashMap<OperationsFlag, Lines>) -> Lines {
        self.operation_snippets
            .iter()
            .map(|flag| snippets.get(flag).cloned().unwrap_or_default())
            .collect_vec()
            .into()
    }

    fn gen_extra(&self) -> Lines {
        self.variants_extras.values().cloned().collect_vec().into()
    }

    fn gen_structs(&self) -> Lines {
        linefy! {
            #import bevy_sprite::mesh2d_functions::mesh2d_position_world_to_clip as world_to_clip;

            struct SdfResult {
                distance: f32,
                color: vec3<f32>,
            }

            struct VertexIn {
                @builtin(vertex_index) index: u32,
                @location(0) size: vec2<f32>,
                @location(1) translation: vec2<f32>,
                @location(2) sdf_index: u32,
            }

            struct VertexOut {
                @builtin(position) position: vec4<f32>,
                @location(0) world_position: vec2<f32>,
                @location(1) sdf_index: u32,
            }
        }
    }

    fn gen_vertex_shader(&self) -> Lines {
        linefy! {
            @vertex
            fn vertex(input: VertexIn) -> VertexOut {
                let vertex_x = f32(input.index & 0x1u) - 0.5;
                let vertex_y = f32((input.index & 0x2u) >> 1u) - 0.5;
                let vertex_direction = vec2<f32>(vertex_x, vertex_y);

                var out: VertexOut;
                out.world_position = vertex_direction * input.size * 2.0;
                out.world_position += input.translation;
                out.position = world_to_clip(vec4(out.world_position, 0.0, 1.0));
                out.sdf_index = input.sdf_index;
                return out;
            }
        }
    }

    fn gen_fragment_shader(&self) -> Lines {
        Lines::block(
            "@fragment fn fragment(input: VertexOut) -> @location(0) vec4<f32>".into(),
            [
                format!(
                    "let result = calc_sdf{}(input.sdf_index, input.world_position);",
                    self.binding
                )
                .into(),
                linefy! {
                    let alpha = smoothstep(0.0, 1.0, -result.distance);
                    return vec4(result.color, alpha);
                },
            ],
        )
    }

    pub fn vertex_buffer_layout(&self) -> VertexBufferLayout {
        VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Instance,
            [
                VertexFormat::Float32x2,
                VertexFormat::Float32x2,
                VertexFormat::Uint32,
            ],
        )
    }
}

pub(crate) fn gen_struct(
    vars: &[(String, String)],
    name: impl Into<String>,
    default_vars: Lines,
    var_prefix: fn(u8) -> String,
) -> Lines {
    Lines::new().add(format!("struct {}", name.into())).block([
        default_vars,
        vars.iter()
            .fold((Lines::new(), 0), |(lines, i), (name, wgsl_type)| {
                (
                    lines.add(format!("{}{name}: {},", var_prefix(i), wgsl_type)),
                    i + 1,
                )
            })
            .0,
    ])
}
