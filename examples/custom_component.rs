use bevy::{color::palettes::css, prelude::*, render::render_resource::ShaderType};
use bevy_comdf::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin, do_a_wave))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);
    cmds.spawn((
        WorldSdf,
        Point::default(),
        Rounded { rounded: 200. },
        DoAWave {
            amplitude: 50.,
            frequency: 10.,
        },
        Fill(css::SKY_BLUE),
    ));
}

fn do_a_wave(app: &mut App) {
    app.sdf::<DoAWave>()
        .affect_bounds(BoundingSet::Add, |s| s.amplitude)
        .register(4000);
    app.add_sdf_shader(stringify!(
        fn do_a_wave(comp: DoAWave) {
            let norm = normalize(position);
            let angle = atan(norm.y / norm.x);
            distance += (sin(angle * comp.frequency) + 0.5) * comp.amplitude;
        }
    ));
}

#[derive(Clone, Debug, Default, Component, ShaderType, Reflect)]
struct DoAWave {
    amplitude: f32,
    frequency: f32,
}
