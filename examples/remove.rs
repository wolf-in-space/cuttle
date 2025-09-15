use bevy::{prelude::*, time::common_conditions::on_timer};
use cuttle::prelude::*;
use rand::{Rng, rng};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CuttlePlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            delete_and_spawn.run_if(on_timer(Duration::from_secs(1))),
        )
        .run();
}

fn setup(mut cmds: Commands) {
    cmds.spawn(Camera2d);
}

fn delete_and_spawn(current: Query<Entity, With<Sdf>>, mut cmds: Commands) {
    for entity in current.into_iter() {
        cmds.entity(entity).despawn();
    }
    for i in 0..rng().random_range(0..30) {
        cmds.spawn((
            Sdf,
            Transform::from_xyz(
                (i % 10) as f32 * 100. - 500.,
                (i / 10) as f32 * 100. - 100.,
                0.,
            ),
            Circle(40.),
            Fill(Srgba::new(
                ((i % 10) + 1) as f32 * 0.1,
                ((i / 10) + 1) as f32 * 0.333,
                0.,
                1.,
            )),
        ));
    }
}
