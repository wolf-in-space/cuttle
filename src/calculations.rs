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

impl Calculation {
    pub fn new(name: impl Into<String>, wgsl_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            wgsl_type: wgsl_type.into(),
        }
    }
}

#[derive(Debug, Default, Component, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct Calculations(pub(crate) Vec<Calculation>);
