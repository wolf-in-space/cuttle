use super::{
    calculations::{gen_calculations, CalculationInfo, CalculationStructures, SdfCalculation},
    lines::Lines,
    CompShaderInfos,
};
use crate::{
    components::buffer::ShaderInput,
    flag::{Comp, Flag, SdfFlags},
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
            .iter()
            .map(|(_, flag)| shader_infos.gather(*flag, |i| i.snippets.clone()).collect())
            .collect(),
        flags
            .iter()
            .skip(1)
            .map(|(op, _)| {
                op_infos[op.bits().trailing_zeros() as usize]
                    .snippets
                    .clone()
            })
            .collect(),
    ])
}

fn gen_inputs(flags: &SdfFlags, bindings: &[usize], shader_infos: &CompShaderInfos) -> Lines {
    flags
        .iter()
        .zip_eq(bindings)
        .map(|((_, flag), binding)| {
            if flag.bits() == 0 {
                Lines::new()
            } else {
                gen_shader_input(
                    *flag,
                    *binding,
                    shader_infos.gather(*flag, |i| i.inputs.iter()).flatten(),
                )
            }
        })
        .collect()
}

fn gen_shader_input<'a>(
    flag: Flag<Comp>,
    binding: usize,
    inputs: impl Iterator<Item = &'a ShaderInput>,
) -> Lines {
    let flag = u64::to_string(&flag.bits());
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
        .iter()
        .skip(1)
        .map(|(_, flag)| {
            if flag.bits() == 0 {
                Lines::new()
            } else {
                gen_sdf_function(
                    *flag,
                    shader_infos
                        .gather(*flag, |i| i.calculations.iter())
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
    flag: Flag<Comp>,
    calcs: impl Iterator<Item = &'a CalculationInfo>,
    structures: &CalculationStructures,
) -> Lines {
    let flag = u64::to_string(&flag.bits());
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
        #import bevy_sprite::mesh2d_functions::mesh2d_position_world_to_clip as world_to_clip;

        @vertex
        fn vertex(input: VertexIn) -> VertexOut {
            let vertex_x = f32(input.index & 0x1u) - 0.5;
            let vertex_y = f32((input.index & 0x2u) >> 1u) - 0.5;
            let vertex_direction = vec2<f32>(vertex_x, vertex_y);

            var out: VertexOut;
            out.world_position = vertex_direction * input.size;
            out.world_position -= input.translation;
            out.position = world_to_clip(vec4(out.world_position, 0.0, 1.0));
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
    let flag @ (_, comp_flag) = flags[0];
    let comp_calcs = comp_infos
        .gather(comp_flag, |i| i)
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
        .iter()
        .enumerate()
        .skip(1)
        .flat_map(|(i, (op, comp))| {
            let info = &op_infos[op.bits().trailing_zeros() as usize];
            [
                line_f!(
                    "op = sdf{}(indices[vertex.data_index + {i}], vertex.world_position);",
                    comp.as_str(),
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
            if flag.1.bits() == 0 {
                Lines::new()
            } else {
                line_f!(
                    "let input = data{}[indices[vertex.data_index]];",
                    comp_flag.as_str(),
                )
            },
            line_f!("let world_position = vertex.world_position;"),
            line_f!("var op: SdfResult;"),
            line_f!("var result: SdfResult;"),
            gen_calculations(before_ops.iter().copied(), structures, true),
            op_calcs,
            gen_calculations(after_ops.iter().copied(), structures, false),
            linefy! {
                let alpha = step(0.0, -result.distance);
                // return vec4(vec3(result.distance / 100.0), 0.5);
                return vec4(result.color, alpha);
            },
            // let alpha = smoothstep(0.0, 1.0, -result.distance);
        ],
    )
}
