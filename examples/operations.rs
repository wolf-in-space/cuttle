use bevy::{color::palettes::css, prelude::*};
use bevy_comdf::{
    implementations::operations::{Base, Union},
    prelude::*,
};

pub fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, spawn)
        .run();
}

pub fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());
    cmds.sdf(())
        .operation::<Base>((Point, Added(50.), Fill(css::SKY_BLUE.into())))
        .operation::<Union>((
            TransformBundle::from_transform(Transform::from_translation(Vec3::new(35., 10., 0.))),
            Rectangle(Vec2::new(30., 30.)),
            Fill(css::SEA_GREEN.into()),
        ));
}
