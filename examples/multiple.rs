use bevy::prelude::*;
use cuttle::prelude::*;
use rand::{thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CuttlePlugin))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);
    for i in 0..30 {
        let base = (
            Name::new(format!("[{} : {}]", i / 10, i % 10)),
            WorldSdf,
            Transform::from_xyz(
                (i % 10) as f32 * 100. - 500.,
                (i / 10) as f32 * 100. - 100.,
                0.,
            ),
            Fill(Srgba::new(
                ((i % 10) + 1) as f32 * 0.1,
                ((i / 10) + 1) as f32 * 0.333,
                0.,
                0.,
            )),
        );
        match thread_rng().gen_range(0..2) {
            0 => cmds.spawn((base, (Point, Rounded { rounded: 40. }))),
            _ => cmds.spawn((
                base,
                (Quad {
                    half_size: Vec2::splat(40.),
                },),
            )),
        };
    }
}
