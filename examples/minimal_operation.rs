use bevy::{color::palettes::css, prelude::*};
use cuttle::prelude::*;
use cuttle::SdfInternals;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, cuttle::plugin))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);

    let subtract = cmds
        .spawn((
            SdfInternals::default(),
            Transform::from_xyz(35., 10., 0.),
            Quad {
                half_size: Vec2::splat(30.),
            },
            Fill(css::REBECCA_PURPLE),
            Subtract::default(),
        ))
        .id();

    cmds.spawn((
        WorldSdf,
        SdfExtensions(vec![subtract]),
        Point,
        Rounded { rounded: 100. },
        Fill(css::SKY_BLUE),
    ));
}
