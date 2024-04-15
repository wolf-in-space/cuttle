use bevy::app::App;
use bevy::ecs::component::Component;
use bevy::ecs::reflect::ReflectComponent;
use bevy::reflect::prelude::*;
use bitflags::bitflags;

use crate::operations::OperationsFlag;

pub fn plugin(app: &mut App) {
    app.register_type::<RenderableVariant>()
        .register_type::<OperationsFlag>()
        .register_type::<RenderSdf>();
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct RenderableVariant {
    #[reflect(ignore)]
    pub flag: VariantFlag,
    #[reflect(ignore)]
    pub binding: u32,
}


#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct RenderSdf(#[reflect(ignore)] pub Vec<(OperationsFlag, u32)>);
