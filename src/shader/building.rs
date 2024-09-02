use super::{
    calculations::{gen_calculations, CalculationInfo, CalculationStructures, SdfCalculation},
    lines::Lines,
    CompShaderInfos,
};
use crate::{
    components::buffer::ShaderInput,
    flag::{CompFlag, SdfFlags},
    implementations::calculations::Position,
    line_f, linefy,
    operations::OperationInfos,
};
use itertools::Itertools;

pub fn gen_shader_wgsl(
    flags: &SdfFlags,
    bindings: &[usize],
    comp_infos: &CompShaderInfos,
    op_infos: &OperationInfos,
    structures: &CalculationStructures,
) -> Lines {
    Lines::from([
        gen_structs(),
        gen_inputs(flags, bindings, comp_infos),
        gen_snippets(flags, comp_infos, op_infos),
        gen_sdf_functions(flags, comp_infos, structures),
        gen_vertex_shader(),
        gen_fragment_shader(flags, comp_infos, op_infos, structures),
    ])
}

fn gen_snippets(
    flags: &SdfFlags,
    shader_infos: &CompShaderInfos,
    op_infos: &OperationInfos,
) -> Lines {
    Lines::from([
        flags
            .iter_comps()
            .map(|flag| shader_infos.gather(flag, |i| i.snippets.clone()).collect())
            .collect(),
        flags
            .operations
            .iter()
            .map(|(op, _)| {
                println!("{}", op.0);
                op_infos[op.minimum().unwrap()].snippets.clone()
            })
            .collect(),
    ])
}

fn gen_inputs(flags: &SdfFlags, bindings: &[usize], shader_infos: &CompShaderInfos) -> Lines {
    flags
        .iter_comps()
        .zip_eq(bindings)
        .map(|(flag, binding)| {
            if flag.is_empty() {
                Lines::new()
            } else {
                gen_shader_input(
                    flag,
                    *binding,
                    shader_infos.gather(flag, |i| i.inputs.iter()).flatten(),
                )
            }
        })
        .collect()
}

fn gen_shader_input<'a>(
    flag: &CompFlag,
    binding: usize,
    inputs: impl Iterator<Item = &'a ShaderInput>,
) -> Lines {
    let flag = flag.to_string();
    Lines::from([
        line_f!(
            "@group(1) @binding({binding}) var<storage, read> data{flag}: array<SdfInput{flag}>;"
        ),
        line_f!("struct SdfInput{flag} {{"),
        inputs
            .map(|input| line_f!("{}:{},", input.name, input.type_info.wgsl_type))
            .collect(),
        "}".into(),
    ])
}

fn gen_sdf_functions(
    flags: &SdfFlags,
    shader_infos: &CompShaderInfos,
    structures: &CalculationStructures,
) -> Lines {
    flags
        .iter_comps()
        .map(|flag| {
            if flag.is_empty() {
                Lines::new()
            } else {
                gen_sdf_function(
                    flag,
                    shader_infos
                        .gather(flag, |i| i.calculations.iter())
                        .flatten(),
                    structures,
                )
            }
        })
        .collect()
}

/// Generates the wgsl code for calculating an sdf made out of
/// an arbitrary combination of ['RenderSdfComponent']s
///
/// ex:
/// fn sdf333(index: u32, position: vec2<f32>) -> SdfResult {
///    let input = data333[index];
///    var result: SdfResult;
///    ...(see [`gen_calculations`])
///    return result;
/// }
fn gen_sdf_function<'a>(
    flag: &CompFlag,
    calcs: impl Iterator<Item = &'a CalculationInfo>,
    structures: &CalculationStructures,
) -> Lines {
    let flag = flag.to_string();
    Lines::block(
        line_f!("fn sdf{flag}(index: u32, world_position: vec2<f32>) -> SdfResult"),
        [
            line_f!("let input = data{flag}[index];"),
            "var result: SdfResult;".into(),
            gen_calculations(calcs, structures, true),
            "return result;".into(),
        ],
    )
}

fn gen_structs() -> Lines {
    linefy! {
        struct SdfResult {
            distance: f32,
            color: vec3<f32>,
        }

        struct VertexIn {
            @builtin(vertex_index) index: u32,
            @location(0) size: vec2<f32>,
            @location(1) translation: vec2<f32>,
            @location(2) data_index: u32,
        }

        struct VertexOut {
            @builtin(position) position: vec4<f32>,
            @location(0) world_position: vec2<f32>,
            @location(2) data_index: u32,
        }
    }
}

fn gen_vertex_shader() -> Lines {
    linefy! {
        #import bevy_render::view::View;
        @group(0) @binding(0) var<uniform> view: View;

        @vertex
        fn vertex(input: VertexIn) -> VertexOut {
            let vertex_x = f32(input.index & 0x1u) - 0.5;
            let vertex_y = f32((input.index & 0x2u) >> 1u) - 0.5;
            let vertex_direction = vec2<f32>(vertex_x, vertex_y);

            var out: VertexOut;
            out.world_position = vertex_direction * input.size;
            out.world_position -= input.translation;
            out.position = view.clip_from_world * vec4(out.world_position, 0.0, 1.0);
            out.data_index = input.data_index;

            return out;
        }
    }
}

///
/// @fragment
/// fn fragment(input: VertexOut) -> @location(0) vec4<f32> {
///     let
///     
/// }

fn gen_fragment_shader(
    flags: &SdfFlags,
    comp_infos: &CompShaderInfos,
    op_infos: &OperationInfos,
    structures: &CalculationStructures,
) -> Lines {
    let comp_calcs = comp_infos
        .gather(&flags.flag, |i| i)
        .flat_map(|i| i.calculations.iter())
        .sorted_by_cached_key(|i| structures[&i.id].order)
        .collect_vec();

    let split = comp_calcs
        .iter()
        .find_position(|info| info.order > Position::order())
        .map(|(i, _)| i)
        .unwrap_or_default();

    let (before_ops, after_ops) = comp_calcs.split_at(split);

    let op_calcs = flags
        .operations
        .iter()
        .enumerate()
        .flat_map(|(i, (op, comp))| {
            let info = &op_infos[op.minimum().unwrap_or(op.len())];
            [
                line_f!(
                    "op = sdf{}(indices[vertex.data_index + {}], vertex.world_position);",
                    comp.to_string(),
                    i + 1
                ),
                line_f!("result = {};", info.operation.to_string()),
            ]
        })
        .collect();

    Lines::block(
        vec![
            "@group(0) @binding(1) var<storage, read> indices: array<u32>;",
            "@fragment fn fragment(vertex: VertexOut) -> @location(0) vec4<f32>",
        ]
        .into(),
        [
            if flags.flag.is_empty() {
                Lines::new()
            } else {
                line_f!(
                    "let input = data{}[indices[vertex.data_index]];",
                    flags.flag.to_string(),
                )
            },
            line_f!("let world_position = vertex.world_position;"),
            line_f!("var op: SdfResult;"),
            line_f!("var result: SdfResult;"),
            gen_calculations(before_ops.iter().copied(), structures, true),
            op_calcs,
            gen_calculations(after_ops.iter().copied(), structures, false),
            linefy! {
                let alpha = clamp(0.0, 1.0, (-result.distance / view.frustum[0].w) * 250.0);
                //let alpha = step(0.0, -result.distance);
                return vec4(result.color, alpha);
            },
            // let alpha = smoothstep(0.0, 1.0, -result.distance);
        ],
    )
}
