use bevy::{color::palettes::css, prelude::*};
use bevy_comdf::prelude::*;

pub fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, spawn)
        .run();
}

pub fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());
    for i in 0..30 {
        cmds.spawn((
            RenderSdfBundle::default(),
            TransformBundle::from_transform(Transform::from_translation(Vec3::new(
                (i % 10) as f32 * 100. - 500.,
                (i / 10) as f32 * 100. - 100.,
                0.,
            ))),
            Point,
            Added(40.),
            Fill(css::SKY_BLUE.into()),
        ));
    }
}
