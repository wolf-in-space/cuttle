use super::{lines::Lines, variants::VariantShaderBuilder};
use crate::{
    flag::{RenderSdf, VariantFlag},
    linefy,
    operations::OperationsFlag,
};
use bevy::{
    prelude::Component,
    render::render_resource::{Shader, VertexBufferLayout, VertexFormat, VertexStepMode},
    utils::{default, HashMap, HashSet},
};
use itertools::Itertools;

#[derive(Default, Clone, Component)]
pub struct SdfShaderBuilder {
    input: Vec<(String, VertexFormat)>,
    variants: HashMap<u32, VariantShaderBuilder>,
    variants_extras: HashMap<VariantFlag, Lines>,
    operations: Lines,
    pub operation_snippets: HashSet<OperationsFlag>,
}

impl SdfShaderBuilder {
    pub fn new() -> Self {
        Self { ..default() }
    }

    pub fn add_sdf_calculation(&mut self, calc: VariantShaderBuilder) {
        self.variants_extras.extend(calc.extra.clone());
        self.variants.insert(calc.binding, calc);
    }

    pub fn input(&mut self, (name, format): (impl Into<String>, VertexFormat)) {
        self.input.push((name.into(), format));
    }

    pub fn operation(&mut self, op: impl Into<Lines>) {
        op.into().lines.into_iter().for_each(|op| {
            let op = op.replace("<prev>", "result");
            let op = format!("result = {op}");
            self.operations.lines.push(op);
        });
    }

    pub fn to_shader(&self, key: &RenderSdf, snippets: &HashMap<OperationsFlag, Lines>) -> Shader {
        let code = self.gen_shader_code(snippets);
        println!("{}", code);
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

    fn gen_vertex_in(&self) -> Lines {
        gen_struct(
            &self.input,
            "VertexIn",
            linefy![
                @builtin(vertex_index) index: u32,
                @location(0) size: vec2<f32>,
                @location(1) translation: vec2<f32>,
            ],
            |i| format!("@location({})", i + 2),
        )
    }

    fn gen_vertex_out(&self) -> Lines {
        gen_struct(
            &self.input,
            "VertexOut",
            linefy![
                @builtin(position) position: vec4<f32>,
                @location(0) world_position: vec2<f32>,
            ],
            |i| format!("@location({})", i + 1),
        )
    }

    fn gen_vertex_out_assigns(&self) -> Lines {
        self.input.iter().fold(Lines::new(), |accu, (name, _)| {
            accu.add(format!("out.{name} = input.{name};"))
        })
    }

    fn gen_structs(&self) -> Lines {
        [
            linefy! {

            #import bevy_sprite::mesh2d_functions::mesh2d_position_world_to_clip as world_to_clip;

            struct SdfResult {
                distance: f32,
                color: vec3<f32>,
            }

            },
            self.gen_vertex_in(),
            self.gen_vertex_out(),
        ]
        .into()
    }

    fn gen_vertex_shader(&self) -> Lines {
        linefy! {
            vertex_out_assigns => self.gen_vertex_out_assigns();
            @vertex
            fn vertex(input: VertexIn) -> VertexOut {
                let vertex_x = f32(input.index & 0x1u) - 0.5;
                let vertex_y = f32((input.index & 0x2u) >> 1u) - 0.5;
                let vertex_direction = vec2<f32>(vertex_x, vertex_y);

                var out: VertexOut;
                out.world_position = vertex_direction * input.size * 2.0;
                out.world_position += input.translation;
                out.position = world_to_clip(vec4(out.world_position, 0.0, 1.0));
                {vertex_out_assigns}
                return out;
            }
        }
    }

    fn gen_fragment_shader(&self) -> Lines {
        linefy! {
            ops => self.operations.clone();

            @fragment
            fn fragment(input: VertexOut) -> @location(0) vec4<f32> {
                var result: SdfResult;
                {ops}
                let alpha = step(result.distance, 0.0);
                return vec4(result.color, alpha);
            }
        }
    }

    pub fn vertex_buffer_layout(&self) -> VertexBufferLayout {
        VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Instance,
            [VertexFormat::Float32x2, VertexFormat::Float32x2]
                .iter()
                .chain(self.input.iter().map(|(_, f)| f))
                .copied(),
        )
    }
}

pub(crate) fn gen_struct(
    vars: &[(String, VertexFormat)],
    name: impl Into<String>,
    default_vars: Lines,
    var_prefix: fn(u8) -> String,
) -> Lines {
    Lines::new().add(format!("struct {}", name.into())).block([
        default_vars,
        vars.iter()
            .fold((Lines::new(), 0), |(lines, i), (name, format)| {
                (
                    lines.add(format!("{}{name}: {},", var_prefix(i), as_wgsl(*format))),
                    i + 1,
                )
            })
            .0,
    ])
}

pub(crate) fn as_wgsl(format: VertexFormat) -> &'static str {
    use VertexFormat::*;
    match format {
        Float32 => "f32",
        Float32x2 => "vec2<f32>",
        Float32x3 => "vec3<f32>",
        Uint32 => "u32",
        _ => unimplemented!(),
    }
}
