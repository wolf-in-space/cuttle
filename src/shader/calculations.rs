use super::lines::Lines;
use crate::components::buffer::BufferType;
use bevy::{
    app::{App, Startup},
    ecs::system::{ResMut, Resource},
    log::error,
    prelude::Deref,
    utils::TypeIdMap,
};
use itertools::Itertools;
use std::{any::TypeId, hash::Hash};

pub fn plugin(app: &mut App) {
    app.init_resource::<CalculationStructures>();
}

pub trait RegisterSdfCalculationAppExt {
    fn register_sdf_calculation<Calc: SdfCalculation>(&mut self) -> &mut Self;
}

impl RegisterSdfCalculationAppExt for App {
    fn register_sdf_calculation<Calc: SdfCalculation>(&mut self) -> &mut Self {
        self.add_systems(Startup, register_calculation_structure::<Calc>);
        self
    }
}

pub trait SdfCalculation: 'static {
    type Value: BufferType;
    fn order() -> usize {
        10000
    }
    fn name() -> &'static str;
    fn initialization() -> CalcInit {
        CalcInit::Default
    }

    fn calculation_structure() -> CalculationStructure {
        CalculationStructure {
            wgsl_type: Self::Value::descriptor().wgsl_type,
            order: Self::order(),
            name: Self::name(),
            initialization: Self::initialization(),
        }
    }

    fn calc(order: usize, calculation: impl Into<String>) -> CalculationInfo {
        CalculationInfo {
            id: TypeId::of::<Self>(),
            order,
            calculation: calculation.into(),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CalcInit {
    None,
    Default,
    Const(&'static str),
}

#[derive(Resource, Default, Deref)]
pub struct CalculationStructures(TypeIdMap<CalculationStructure>);

fn register_calculation_structure<Calc: SdfCalculation>(mut calcs: ResMut<CalculationStructures>) {
    calcs
        .0
        .insert(TypeId::of::<Calc>(), Calc::calculation_structure());
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalculationStructure {
    pub order: usize,
    name: &'static str,
    wgsl_type: &'static str,
    initialization: CalcInit,
}

#[derive(Debug, Clone)]
pub struct CalculationInfo {
    pub id: TypeId,
    pub order: usize,
    calculation: String,
}

impl CalculationInfo {
    pub fn new<C: SdfCalculation>(order: usize, calculation: String) -> Self {
        Self {
            id: TypeId::of::<C>(),
            order,
            calculation,
        }
    }
}

pub(crate) fn gen_calculations<'a>(
    calcs: impl Iterator<Item = &'a CalculationInfo>,
    structures: &CalculationStructures,
    init_calcs: bool,
) -> Lines {
    let mut calc_map: TypeIdMap<_> = structures
        .iter()
        .map(|(k, structure)| (*k, (structure, Vec::new())))
        .collect();

    for calc in calcs {
        match calc_map.get_mut(&calc.id) {
            Some((_, calcs)) => calcs.push(calc),
            None => error!("CalculationStructure not found for {calc:?}, you probably need to register it with 'app.register_sdf_calculation'"),
        }
    }

    calc_map
        .into_iter()
        .map(|(_, v)| v)
        .sorted_by_key(|(key, _)| key.order)
        .map(|(key, calcs)| {
            let init = match key.initialization {
                CalcInit::None => "".into(),
                _ if !init_calcs => "".into(),
                CalcInit::Default => format!("var {}: {};", key.name, key.wgsl_type),
                CalcInit::Const(init) => format!("var {}: {} = {};", key.name, key.wgsl_type, init),
            };
            let calculations = calcs
                .into_iter()
                .sorted_by_key(|calc| calc.order)
                .map(|calc| format!("{} = {};", key.name, calc.calculation).into())
                .collect();
            [init.into(), calculations].into()
        })
        .collect()
}
