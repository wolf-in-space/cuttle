use bevy::{color::palettes::css, prelude::*};
use cuttle::prelude::*;
use cuttle::CuttleFlags;

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
            CuttleFlags::default(),
            Transform::from_xyz(35., 10., 0.),
            Quad {
                half_size: Vec2::splat(30.),
            },
            Fill(css::REBECCA_PURPLE),
            Subtract::default(),
        ))
        .id();

    cmds.spawn((
        Sdf,
        Extensions(vec![subtract]),
        builtins::Circle { radius: 100. },
        Fill(css::SKY_BLUE),
    ));
}
