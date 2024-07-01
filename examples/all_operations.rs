use bevy::{color::palettes::css, prelude::*};
use bevy_comdf::{
    implementations::operations::{Base, Subtract, Union},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());
    cmds.sdf(())
        .operation::<Base>((Point, Added(50.), Fill(css::SKY_BLUE.into())))
        .operation::<Union>((
            TransformBundle::from_transform(Transform::from_translation(Vec3::new(35., 10., 0.))),
            Rectangle(Vec2::new(30., 30.)),
            Fill(css::SEA_GREEN.into()),
        ));

    cmds.sdf(())
        .operation::<Base>((Line(25.), Added(15.), Fill(css::RED.into())))
        .operation::<Subtract>((
            TransformBundle::from_transform(Transform::from_translation(Vec3::new(35., 10., 0.))),
            Point,
            Added(10.),
            Fill(css::SEA_GREEN.into()),
        ));
}
