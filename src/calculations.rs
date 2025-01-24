use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Calculations>()
        .register_type::<Calculation>();
}

#[derive(Debug, Clone, Reflect)]
pub struct Calculation {
    pub name: String,
    pub wgsl_type: String,
}

#[derive(Debug, Default, Component, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct Calculations(pub(crate) Vec<Calculation>);
