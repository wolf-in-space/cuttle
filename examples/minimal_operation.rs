use bevy::{color::palettes::css, prelude::*};
use bevy_comdf::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);

    let subtract = cmds
        .spawn((
            Sdf::default(),
            Transform::from_xyz(35., 10., 0.),
            Quad {
                half_size: Vec2::splat(30.),
            },
            Fill(css::REBECCA_PURPLE.into()),
            Subtract::default(),
        ))
        .id();

    cmds.spawn((
        WorldSdf,
        SdfExtensions(vec![subtract]),
        Point::default(),
        Rounded { rounded: 100. },
        Fill(css::SKY_BLUE.into()),
    ));
}
