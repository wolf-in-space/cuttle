use bevy::ecs::component::Component;
use bevy::render::render_resource::{
    BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages,
};
use bevy::utils::{default, HashMap, HashSet};

#[derive(Default, Debug, Clone, Component)]
pub struct SdfCalculationBuilder {
    input: Vec<(String, String)>,
    pub(crate) extra: HashMap<RenderSdfFlag, Lines>,
    calculations: HashMap<Calculation, Vec<String>>,
    // pub operation_snippets: HashSet<OperationsFlag>,
    pub(crate) binding: u32,
}

impl SdfCalculationBuilder {
    pub fn new(binding: u32) -> Self {
        Self {
            binding,
            ..default()
        }
    }

    pub(crate) fn build(&self) -> Lines {
        [
            format!(
                "@group(1) @binding({bind}) var<storage, read> data{bind}: array<Sdf{bind}>;",
                bind = self.binding
            )
            .into(),
            self.sdf_struct(),
            format!(
                "fn calc_sdf{bind}(input_index: u32, world_position: vec2<f32>) -> SdfResult {{",
                bind = self.binding
            )
            .into(),
            format!("let input = data{}[input_index];", self.binding).into(),
            self.calculations(),
            stringify!(return result;).into(),
            "}".into(),
        ]
        .into()
    }

    pub fn input_len(&self) -> usize {
        self.input.len()
    }

    fn sdf_struct(&self) -> Lines {
        // gen_struct(
        //     &self.input,
        //     format!("Sdf{}", self.binding),
        //     Lines::new(),
        //     |_| "".into(),
        // )
        todo!()
    }

    fn calculations(&self) -> Lines {
        [
            self.calculation(Position),
            self.calculation(Operations),
            self.calculation(Distance),
            self.calculation(PixelColor),
        ]
        .into()
    }

    pub fn calc(&mut self, kind: Calculation, calc: impl Into<String>) {
        self.calculations.entry(kind).or_default().push(calc.into());
    }

    pub fn extra(&mut self, flag: RenderSdfFlag, extra: Lines) {
        self.extra.insert(flag, extra);
    }

    pub fn input(&mut self, name: impl Into<String>, wgsl_type: impl Into<String>) {
        self.input.push((name.into(), wgsl_type.into()));
    }

    fn calculation(&self, kind: Calculation) -> Lines {
        let mut result = Vec::new();
        let name = kind.var_name();

        if let Some(default) = kind.default_val() {
            result.push(format!("var {name} = {default};"));
        }
        if let Some(calculations) = self.calculations.get(&kind) {
            result.extend(calculations.iter().map(|calc| format!("{name} = {calc};")));
        };

        result.into()
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
    Operations,
    Distance,
    PixelColor,
}
pub use Calculation::*;

use crate::shader::lines::Lines;

impl Calculation {
    fn var_name(&self) -> &str {
        match self {
            Position => "position",
            Operations => "result",
            Distance => "result.distance",
            PixelColor => "result.color",
        }
    }

    fn default_val(&self) -> Option<&str> {
        match self {
            Position => Some("world_position"),
            Operations => Some("SdfResult()"),
            Distance => None,
            PixelColor => None,
        }
    }
}
