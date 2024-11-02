use super::wgsl_struct::WgslTypeInfos;
use crate::{calculations::Calculations, components::SdfCompInfos};
use convert_case::{Case, Casing};
use std::fmt::Write;

pub fn gen_shader(
    infos: &SdfCompInfos,
    wgsl_types: &WgslTypeInfos,
    calcs: &Calculations,
    snippets: String,
) -> String {
    let export = "#define_import_path bevy_comdf::gen\n";
    let selector = comp_selector(infos);
    let stuff = structs_and_bindings(infos, wgsl_types);
    let calcs = calculations(calcs);

    let shader = format!("{export}\n{stuff}\n{calcs}\n{snippets}\n{selector}");
    // println!("{shader}");
    shader
}

fn calculations(calcs: &Calculations) -> String {
    calcs
        .iter()
        .try_fold(String::new(), |mut result, calc| {
            writeln!(result, "var<private> {}: {};", calc.name, calc.wgsl_type)?;
            Ok::<_, std::fmt::Error>(result)
        })
        .unwrap()
}

fn comp_selector(infos: &SdfCompInfos) -> String {
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

fn structs_and_bindings(infos: &SdfCompInfos, wgsl_types: &WgslTypeInfos) -> String {
    infos
        .iter()
        .enumerate()
        .flat_map(|(i, info)| {
            let structure = wgsl_types.info_to_wgsl(info);
            let binding = format!(
                "@group(2) @binding({i}) var<storage, read> comps{i}: array<{}>;\n\n",
                info.name
            );
            [structure, binding]
        })
        .collect()
}
