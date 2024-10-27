use bevy::{color::palettes::css, prelude::*, render::render_resource::ShaderType};
use bevy_comdf::prelude::*;
use operations::ExtendSdf;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin, add_sdf_comp))
        .add_systems(Startup, spawn)
        .add_systems(Update, update)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);
    let sdf = cmds
        .spawn((
            WorldSdf,
            Point::default(),
            Rounded { rounded: 275. },
            Fill(css::ORANGE_RED.into()),
        ))
        .id();
    cmds.spawn((
        ExtendSdf::new(sdf),
        Fire {
            amplitude: 500.,
            frequency: 0.17,
            time: 0.,
        },
        Fill(css::ORANGE_RED.into()),
        SmoothIntersect { smoothness: 75. },
    ));
}

fn add_sdf_comp(app: &mut App) {
    app.sdf::<Fire>().affect_aabb().register(4000);
    // app.add_sdf_shader(stringify!(
    //     fn fire(comp: Fire) {
    //         let norm = normalize(position);
    //         let angle = atan(norm.y / norm.x);
    //         distance +=
    //             abs(sin(angle * comp.frequency + comp.time) + 0.5) * pow(comp.amplitude, 2.8);
    //     }
    // ));
    app.add_sdf_shader(stringify!(
        fn fire(comp: Fire) {
            let pos = position.xy * comp.frequency + comp.time * 2.5;
            distance = sin(pos.x) + cos(pos.y) + cos(pos.y) * sin(pos.x);
        }
    ));
}

fn update(mut fires: Query<(&mut Fire, &mut Transform)>, time: Res<Time>) {
    for (mut fire, mut transform) in &mut fires {
        transform.rotate_z(0.0174);
        fire.time = time.elapsed_secs();
    }
}

#[derive(Clone, Debug, Default, Component, ShaderType, Reflect)]
struct Fire {
    amplitude: f32,
    frequency: f32,
    time: f32,
}

impl AddToBoundingRadius for Fire {
    fn compute(&self) -> f32 {
        self.amplitude
    }
}
