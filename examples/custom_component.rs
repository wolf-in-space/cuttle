use bevy::{color::palettes::css, prelude::*};
use bevy_comdf::components::buffer::SdfBuffer;
use bevy_comdf::components::RegisterSdfRenderCompAppExt;
use bevy_comdf::shader::calculations::SdfCalculation;
use bevy_comdf::shader::lines::Lines;
use bevy_comdf::shader::CompShaderInfo;
use bevy_comdf::{
    components::{buffer::BufferType, RenderSdfComponent},
    implementations::calculations::Distance,
    linefy,
    prelude::*,
};

pub fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, spawn)
        .register_sdf_render_comp::<DoAWave>()
        .run();
}

pub fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());
    cmds.spawn((
        RenderSdfBundle::default(),
        Point,
        Added(250.),
        DoAWave {
            amplitude: 50.,
            frequency: 10.,
        },
        Fill(css::SKY_BLUE.into()),
    ));
}

#[derive(Clone, Component)]
struct DoAWave {
    amplitude: f32,
    frequency: f32,
}

impl RenderSdfComponent for DoAWave {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![
                f32::shader_input("wave_amplitude"),
                f32::shader_input("wave_frequency"),
            ],
            snippets: linefy! {
                fn do_a_wave(pos: vec2<f32>, amp: f32, freq: f32) -> f32 {
                    let norm = normalize(pos);
                    let angle = atan(norm.y / norm.x);
                    return (sin(angle * freq) + 0.5) * amp;
                }
            },
            calculations: vec![Distance::calc(
                2500,
                "result.distance - do_a_wave(position, input.wave_amplitude, input.wave_frequency)",
            )],
        }
    }

    fn push_to_buffer(&self, render: &mut SdfBuffer) {
        render.push(&self.amplitude);
        render.push(&self.frequency);
    }
}
