use crate::shader::calculations::{CalcInit, RegisterSdfCalculationAppExt, SdfCalculation};
use bevy::{
    app::App,
    math::{Vec2, Vec3},
};
use core::str;

pub fn plugin(app: &mut App) {
    app.register_sdf_calculation::<Position>()
        .register_sdf_calculation::<Distance>()
        .register_sdf_calculation::<PixelColor>();
}

pub struct Position;
impl SdfCalculation for Position {
    type Value = Vec2;

    fn name() -> &'static str {
        "position"
    }

    fn initialization() -> CalcInit {
        CalcInit::Const("world_position")
    }

    fn order() -> usize {
        1000
    }
}
/*
pub struct ResultCalcs;
impl SdfCalculation for ResultCalcs {
    type Value = SdfResult;

    fn name() -> &'static str {
        "result"
    }

    fn order() -> usize {
        Position::order() + 1000
    }
}

pub struct OpCalcs;
impl SdfCalculation for OpCalcs {
    type Value = SdfResult;

    fn name() -> &'static str {
        "op"
    }

    fn order() -> usize {
        ResultCalcs::order()
    }
}
*/
pub struct Distance;
impl SdfCalculation for Distance {
    type Value = f32;

    fn name() -> &'static str {
        "result.distance"
    }

    fn initialization() -> CalcInit {
        CalcInit::None
    }

    fn order() -> usize {
        Position::order() + 1000
    }
}

pub struct PixelColor;
impl SdfCalculation for PixelColor {
    type Value = Vec3;

    fn name() -> &'static str {
        "result.color"
    }

    fn initialization() -> CalcInit {
        CalcInit::None
    }

    fn order() -> usize {
        Distance::order() + 1000
    }
}
