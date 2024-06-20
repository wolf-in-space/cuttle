use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<Fill>();
    app.register_type::<Gradient>();
    app.register_type::<Border>();
}

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Fill(pub Color);

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Gradient {
    pub color: Color,
    pub intervall: f32,
}

impl Gradient {
    pub fn new(color: Color, intervall: f32) -> Self {
        Self { color, intervall }
    }
}

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Border {
    pub color: Color,
    pub thickness: f32,
}

impl Border {
    pub fn new(color: Color, thickness: f32) -> Self {
        Self { color, thickness }
    }
}
