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
    cmds.spawn((
        Sdf,
        Point,
        Rounded { rounded: 50. },
        Fill(css::SKY_BLUE),
    ));
}
