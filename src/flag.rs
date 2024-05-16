use bevy_app::App;
use bevy_ecs::component::Component;
use bitflags::bitflags;

use crate::operations::OperationsFlag;

pub fn plugin(app: &mut App) {
    app.register_type::<OperationsFlag>();
}

bitflags! {
    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Component)]
    pub struct RenderableSdf: u32 {
        //Primitives
        const Point = 1 << 0;
        const Rectangle = 1 << 1;
        const Line = 1 << 2;
        //Transforms
        const Translated = 1 << 5;
        const Rotated = 1 << 6;
        const Transform = 1 << 7;
        //Shape modifiers
        const Added = 1 << 10;
        const Annular = 1 << 11;
        const Stretched = 1 << 12;
        const Bend = 1 << 13;
        //Color
        const Fill = 1 << 20;
        const Gradient = 1 << 21;
        const Border = 1 << 22;
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component)]
pub struct SdfPipelineKey(pub Vec<(OperationsFlag, RenderableSdf)>);
