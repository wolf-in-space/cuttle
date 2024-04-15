use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<FillColor>();
    app.register_type::<GradientColor>();
}

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct FillColor(pub Color);


#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct GradientColor(pub Color);

