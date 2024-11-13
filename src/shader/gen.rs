use crate::calculations::Calculation;
use crate::components::initialization::ComponentShaderInfo;
use convert_case::{Case, Casing};
use std::fmt::Write;

pub fn gen_shader(
    infos: &[ComponentShaderInfo],
    calculations: &[Calculation],
    snippets: String,
) -> String {
    let export = "#define_import_path bevy_comdf::gen\n";
    let selector = comp_selector(infos);
    let stuff = structs_and_bindings(infos);
    let calculations = gen_calculations(calculations);

    let shader = format!("{export}\n{stuff}\n{calculations}\n{snippets}\n{selector}");
    shader
}

fn gen_calculations(calculations: &[Calculation]) -> String {
    calculations
        .iter()
        .try_fold(String::new(), |mut result, calc| {
            writeln!(result, "var<private> {}: {};", calc.name, calc.wgsl_type)?;
            Ok::<_, std::fmt::Error>(result)
        })
        .unwrap()
}

fn comp_selector(infos: &[ComponentShaderInfo]) -> String {
    let fn_header = "fn component(comp_id: u32, index: u32)";
    let switch = "  switch comp_id ";
    let switch_body: String = infos
        .iter()
        .enumerate()
        .try_fold(String::new(), |mut result, (i, info)| {
            let snake = info.name.to_case(Case::Snake);
            writeln!(result, "    case u32({i}): {{")?;
            writeln!(result, "      let info = comps{i}[index];")?;
            writeln!(result, "      {snake}(info);")?;
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
            let binding = format!(
                "@group(2) @binding({}) var<storage, read> comps{}: array<{}>;\n\n",
                info.binding, i, info.name,
            );
            [info.struct_wgsl.clone(), binding]
        })
        .collect()
}
