use bevy::{color::palettes::css, prelude::*, time::common_conditions::on_timer};
use bevy_comdf::prelude::*;
use rand::{thread_rng, Rng};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            delete_and_spawn.run_if(on_timer(Duration::from_secs(1))),
        )
        .run();
}

fn setup(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());
}

fn delete_and_spawn(current: Query<Entity, With<Point>>, mut cmds: Commands) {
    for entity in current.into_iter() {
        cmds.entity(entity).despawn();
    }
    for i in 0..thread_rng().gen_range(0..30) {
        cmds.spawn((
            RenderSdfBundle::new()
                .with_pos([(i % 10) as f32 * 100. - 500., (i / 10) as f32 * 100. - 100.]),
            Point,
            Added(40.),
            Fill(css::SKY_BLUE.into()),
        ));
    }
}