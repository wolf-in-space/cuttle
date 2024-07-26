use bevy::{color::palettes::css, prelude::*};
use bevy_comdf::prelude::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());

    // Circle
    cmds.spawn((
        RenderSdfBundle::new().with_pos([0., 0.]),
        Point,
        Added(50.),
        Fill(css::RED.into()),
    ));

    // Donut / Annular Circle
    cmds.spawn((
        RenderSdfBundle::new().with_pos([0., 200.]),
        Point,
        Added(35.),
        Annular(15.),
        Fill(css::REBECCA_PURPLE.into()),
    ));

    // Gradient Circle
    cmds.spawn((
        RenderSdfBundle::new().with_pos([0., -200.]),
        Point,
        Added(50.),
        Fill(css::BLACK.into()),
        Gradient {
            color: css::MEDIUM_ORCHID.into(),
            intervall: 1.,
        },
    ));

    // Rounded square
    cmds.spawn((
        RenderSdfBundle::new().with_pos([-200., -200.]),
        Rectangle(Vec2::new(30., 30.)),
        Added(20.),
        Fill(css::TURQUOISE.into()),
    ));

    // Square
    cmds.spawn((
        RenderSdfBundle::new().with_pos([-200., 0.]),
        Rectangle(Vec2::new(50., 50.)),
        Fill(css::GREEN.into()),
    ));

    // Rectangle
    cmds.spawn((
        RenderSdfBundle::new().with_pos([-200., 200.]),
        Rectangle(Vec2::new(70., 30.)),
        Fill(css::LAWN_GREEN.into()),
    ));

    // Annular square
    cmds.spawn((
        RenderSdfBundle::new().with_pos([-400., 200.]),
        Rectangle(Vec2::new(30., 30.)),
        Added(10.),
        Annular(10.),
        Fill(css::STEEL_BLUE.into()),
    ));

    // Rotated square
    cmds.spawn((
        RenderSdfBundle {
            sdf: SdfBundle {
                transform: TransformBundle::from_transform(
                    Transform::from_translation(Vec3::new(-400., 0., 0.))
                        .with_rotation(Quat::from_rotation_z(PI * 0.25)),
                ),
                ..default()
            },
            ..default()
        },
        Rectangle(Vec2::new(30., 30.)),
        Added(20.),
        Fill(css::ROYAL_BLUE.into()),
    ));

    // Gradient Line
    cmds.spawn((
        RenderSdfBundle::new().with_pos([200., 200.]),
        Line(32.),
        Annular(10.),
        Added(22.),
        Fill(css::CADET_BLUE.into()),
        Gradient {
            color: css::BLACK.into(),
            intervall: 1.25,
        },
    ));

    // Line
    cmds.spawn((
        RenderSdfBundle::new().with_pos([200., 0.]),
        Line(32.),
        Added(32.),
        Fill(css::BLUE.into()),
    ));

    // Annular Line
    cmds.spawn((
        RenderSdfBundle::new().with_pos([200., -200.]),
        Line(32.),
        Annular(10.),
        Added(22.),
        Fill(css::LIGHT_YELLOW.into()),
    ));
}
