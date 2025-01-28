use crate::shader::ComponentShaderInfo;
use convert_case::{Case, Casing};
use std::fmt::Write;

pub fn gen_shader(infos: &[ComponentShaderInfo], snippets: String) -> String {
    let selector = comp_selector(infos);
    let stuff = structs_and_bindings(infos);

    let shader = format!("{snippets}\n{stuff}\n{selector}");

    // println!("SHADER:\n{}", shader);
    shader
}

fn comp_selector(infos: &[ComponentShaderInfo]) -> String {
    let fn_header = "fn component(comp_id: u32, index: u32)";
    let switch = "  switch comp_id ";
    let switch_body: String = infos
        .iter()
        .enumerate()
        .try_fold(String::new(), |mut result, (i, info)| {
            let snake = info.function_name.to_case(Case::Snake);
            writeln!(result, "    case u32({i}): {{")?;
            match info.render_data {
                Some(_) => {
                    writeln!(result, "      let info = comps{i}[index];")?;
                    writeln!(result, "      {snake}(info);")?;
                }
                None => {
                    writeln!(result, "      {snake}();")?;
                }
            }
            writeln!(result, "    }}")?;
            Ok::<_, std::fmt::Error>(result)
        })
        .unwrap();
    let default_case = format!("    case default: {}", "{}\n");

    format!(
        "{fn_header}{0}{switch}{0}{switch_body}{default_case}  {1}{1}",
        "{\n", "}\n"
    )
}

fn structs_and_bindings(infos: &[ComponentShaderInfo]) -> String {
    infos
        .iter()
        .enumerate()
        .flat_map(|(i, info)| {
            info.render_data.clone().map(|render| {
                format!(
                    "@group(2) @binding({}) var<storage, read> comps{}: array<{}>;\n\n{}",
                    render.binding, i, render.wgsl.name, render.wgsl.definition
                )
            })
        })
        .collect()
}
