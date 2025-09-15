use bevy::prelude::*;
use cuttle::prelude::*;
use rand::{rng, Rng};

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
            Sdf,
            Transform::from_xyz(
                (i % 10) as f32 * 100. - 500.,
                (i / 10) as f32 * 100. - 100.,
                0.,
            ),
            Fill(Srgba::new(
                ((i % 10) + 1) as f32 * 0.1,
                ((i / 10) + 1) as f32 * 0.333,
                0.,
                1.,
            )),
        );
        match rng().random_range(0..2) {
            0 => cmds.spawn((base, (Circle(40.)))),
            _ => cmds.spawn((base, (Quad(Vec2::splat(40.))))),
        };
    }
}
