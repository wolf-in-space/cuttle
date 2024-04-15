use bevy::{
    prelude::Component,
    render::render_resource::{
        BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages, VertexFormat,
    },
    utils::{default, HashMap},
};

#[derive(Default, Debug, Clone, Component)]
pub struct VariantShaderBuilder {
    input: Vec<(String, VertexFormat)>,
    pub(crate) extra: HashMap<VariantFlag, Lines>,
    calculations: HashMap<Calculation, Vec<String>>,
    pub(crate) binding: u32,
}

impl VariantShaderBuilder {
    pub fn new(binding: u32) -> Self {
        Self {
            binding,
            ..default()
        }
    }

    pub(crate) fn build(&self) -> Lines {
        linefy! {
            bind => self.binding, layout => self.sdf_struct(), calculations => self.calculations();

            @group(1) @binding({bind}) var<storage, read> data{bind}: array<Sdf{bind}>;

            {layout}

            fn calc_sdf{bind}(input_index: u32, world_position: vec2<f32>) -> SdfResult {
                let input = data{bind}[input_index];
                var result: SdfResult;
                {calculations}
                return result;
            }
        }
    }

    fn sdf_struct(&self) -> Lines {
        gen_struct(
            &self.input,
            format!("Sdf{}", self.binding),
            Lines::new(),
            |_| "".into(),
        )
    }

    fn calculations(&self) -> Lines {
        [
            self.calculation(Position, "let position"),
            self.calculation(Distance, "result.distance"),
            self.calculation(PixelColor, "result.color"),
        ]
        .into()
    }

    pub fn calc(&mut self, kind: Calculation, calc: impl Into<String>) {
        let calc = calc.into();
        let calc = calc.replace("<prev>", &self.var(kind));
        self.calculations.entry(kind).or_default().push(calc);
    }

    pub fn extra(&mut self, flag: VariantFlag, extra: Lines) {
        self.extra.insert(flag, extra);
    }

    pub fn input(&mut self, (name, format): (impl Into<String>, VertexFormat)) {
        self.input.push((name.into(), format));
    }

    fn var(&self, kind: Calculation) -> String {
        self.calculations
            .get(&kind)
            .and_then(|calc| {
                if calc.is_empty() {
                    None
                } else {
                    Some(format!("{}{}", kind.var_name(), calc.len() - 1))
                }
            })
            .unwrap_or_else(|| kind.default_val())
    }

    fn calculation(&self, kind: Calculation, final_name: impl Into<String>) -> Lines {
        let Some(calculations) = self.calculations.get(&kind) else {
            return Lines::default();
        };

        let name = kind.var_name();
        let final_name = final_name.into();
        let last_calc_index = calculations.len() - 1;

        calculations
            .iter()
            .enumerate()
            .map(|(i, calc)| {
                if i == last_calc_index {
                    format!("{final_name} = {calc};")
                } else {
                    format!("let {name}{i} = {calc};")
                }
            })
            .collect_vec()
            .into()
    }

    pub fn bindgroup_layout_entry(&self) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
            binding: self.binding,
            visibility: ShaderStages::VERTEX_FRAGMENT,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Calculation {
    Position,
    Distance,
    PixelColor,
}
use itertools::Itertools;
pub use Calculation::*;

use super::{building::gen_struct, lines::Lines};
use crate::{flag::VariantFlag, linefy};

impl Calculation {
    fn var_name(&self) -> &str {
        match self {
            Position => "position",
            Distance => "distance",
            PixelColor => "color",
        }
    }

    fn default_val(&self) -> String {
        match self {
            Position => "world_position",
            Distance => "0.0",
            PixelColor => "vec3<f32>(1.0)",
        }
        .to_string()
    }
}
