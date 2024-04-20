use bevy_app::App;
use bevy_ecs::component::Component;
use bitflags::bitflags;

use crate::operations::OperationsFlag;

pub fn plugin(app: &mut App) {
    app.register_type::<OperationsFlag>();
}

bitflags! {
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct VariantFlag: u32 {
        //Primitives
        const Point = 1 << 0;
        const Rectangle = 1 << 1;
        const Line = 1 << 2;
        //Transforms
        const Translated = 1 << 10;
        const Rotated = 1 << 11;
        //Shape modifiers
        const Added = 1 << 21;
        const Annular = 1 << 22;
        const Stretched = 1 << 23;
        const Bend = 1 << 24;
        //Color
        const FillColor = 1 << 30;
        const GradientColor = 1 << 31;
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component)]
pub struct RenderableSdf {
    pub flag: VariantFlag,
    pub binding: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component)]
pub struct SdfPipelineKey(pub Vec<(OperationsFlag, u32)>);
