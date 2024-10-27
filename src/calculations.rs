use bevy::{
    app::App,
    prelude::{Deref, DerefMut, Resource},
};

pub fn plugin(app: &mut App) {
    app.init_resource::<Calculations>();
}

pub struct Calculation {
    pub name: String,
    pub wgsl_type: String,
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct Calculations(Vec<Calculation>);
