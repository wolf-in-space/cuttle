use bevy::{color::palettes::css, prelude::*};
use cuttle::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CuttlePlugin))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);

    let subtract = cmds
        .spawn((
            Sdf,
            Transform::from_xyz(35., 10., 0.),
            Quad(Vec2::splat(30.)),
            Fill(css::REBECCA_PURPLE),
        ))
        .id();

    cmds.spawn((
        Extends(subtract),
        Transform::default(),
        Circle(75.),
        Fill(css::SKY_BLUE),
        Intersect,
    ));
}
