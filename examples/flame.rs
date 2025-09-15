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
        Transform::default(),
        Flame {
            flicker: 0.5,
            sharpness: 0.8,
            base: 1.0,
            tip: 0.5,
        },
        Fill(css::SKY_BLUE),
    ));
}
