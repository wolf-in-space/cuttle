use bevy::{
    color::palettes::{css, tailwind},
    prelude::*,
};
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

    cmds.sdf(Gradient {
        color: tailwind::NEUTRAL_200.into(),
        intervall: 1.,
    })
    .operation::<Base>((Point, Added(50.), Fill(tailwind::SKY_400.into())))
    .operation::<Union>((
        transform_from_pos(35., 10.),
        Rectangle(Vec2::new(30., 30.)),
        Fill(tailwind::SKY_400.into()),
    ));

    cmds.sdf(Gradient {
        color: tailwind::NEUTRAL_200.into(),
        intervall: 1.,
    })
    .operation::<Base>((
        transform_from_pos(0., 200.),
        Line(25.),
        Added(15.),
        Fill(css::RED.into()),
    ))
    .operation::<Subtract>((
        transform_from_pos(35., 210.),
        Point,
        Added(20.),
        Fill(css::SEA_GREEN.into()),
    ));
}

fn transform_from_pos(x: f32, y: f32) -> TransformBundle {
    TransformBundle::from_transform(Transform::from_translation(Vec3::new(x, y, 0.)))
}
