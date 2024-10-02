use bevy::{color::palettes::css, prelude::*};
use bevy_comdf::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());
    for i in 0..30 {
        cmds.spawn((
            Name::new(format!("[{} : {}]", i / 10, i % 10)),
            RenderSdfBundle::new()
                .with_pos([(i % 10) as f32 * 100. - 500., (i / 10) as f32 * 100. - 100.]),
            Point,
            Added(40.),
            Fill(css::SKY_BLUE.into()),
        ));
    }
}
