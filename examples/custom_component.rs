use bevy::{color::palettes::css, prelude::*, render::render_resource::ShaderType};
use cuttle::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CuttlePlugin, do_a_wave))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);
    cmds.spawn((
        Sdf,
        Transform::default(),
        Circle(200.),
        DoAWave {
            amplitude: 50.,
            frequency: 10.,
        },
        Fill(css::SKY_BLUE),
    ));
}

fn do_a_wave(app: &mut App) {
    app.cuttle_config::<Sdf>()
        .component::<DoAWave>()
        .affect_bounds(Bounding::Add, |&DoAWave { amplitude, .. }| amplitude)
        .snippet(stringify!(
            fn do_a_wave(comp: DoAWave) {
                let norm = normalize(position);
                let angle = atan(norm.y / norm.x);
                distance += (sin(angle * comp.frequency) + 0.5) * comp.amplitude;
            }
        ));
}

#[derive(Clone, Debug, Default, Component, ShaderType, Reflect, Cuttle)]
#[cuttle(sort(SdfOrder::Distance))]
struct DoAWave {
    amplitude: f32,
    frequency: f32,
}
