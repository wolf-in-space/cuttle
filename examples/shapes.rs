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

    //Circle
    cmds.spawn((
        TransformBundle::from_transform(Transform::from_translation(Vec3::new(0., 0., 0.))),
        RenderSdfBundle::default(),
        Point,
        Added(50.),
        Fill(css::RED.into()),
    ));

    //Annular circle
    cmds.spawn((
        TransformBundle::from_transform(Transform::from_translation(Vec3::new(0., 200., 0.))),
        RenderSdfBundle::default(),
        Point,
        Added(35.),
        Annular(15.),
        Fill(css::REBECCA_PURPLE.into()),
    ));

    //Rounded square
    cmds.spawn((
        TransformBundle::from_transform(Transform::from_translation(Vec3::new(-200., -200., 0.))),
        RenderSdfBundle::default(),
        Rectangle(Vec2::new(30., 30.)),
        Added(20.),
        Fill(css::TURQUOISE.into()),
    ));

    //Square
    cmds.spawn((
        TransformBundle::from_transform(Transform::from_translation(Vec3::new(-200., 0., 0.))),
        RenderSdfBundle::default(),
        Rectangle(Vec2::new(50., 50.)),
        Fill(css::GREEN.into()),
    ));

    //Rectangle
    cmds.spawn((
        TransformBundle::from_transform(Transform::from_translation(Vec3::new(-200., 200., 0.))),
        RenderSdfBundle::default(),
        Rectangle(Vec2::new(70., 30.)),
        Fill(css::LAWN_GREEN.into()),
    ));

    //Line
    cmds.spawn((
        TransformBundle::from_transform(Transform::from_translation(Vec3::new(200., 0., 0.))),
        RenderSdfBundle::default(),
        Line(22.),
        Added(8.),
        Fill(css::BLUE.into()),
    ));
}
