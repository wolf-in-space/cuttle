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
    cmds.spawn((
        WorldSdf,
        Point { hi: 0. },
        Rounded { rounded: 50. },
        Fill(css::SKY_BLUE),
    ));
}
