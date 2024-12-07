use bevy::{color::palettes::css, prelude::*, render::render_resource::ShaderType};
use cuttle::components::initialization::SdfComponent;
use cuttle::groups::SdfGroupBuilderAppExt;
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
        Point,
        Rounded { rounded: 200. },
        DoAWave {
            amplitude: 50.,
            frequency: 10.,
        },
        Fill(css::SKY_BLUE),
    ));
}

fn do_a_wave(app: &mut App) {
    app.sdf_group::<Sdf>()
        .component::<DoAWave>()
        .snippet(stringify!(
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

impl SdfComponent for DoAWave {
    type RenderData = Self;
    const SORT: u32 = DISTANCE_POS + 500;
}
