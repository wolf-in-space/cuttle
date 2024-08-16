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

    cmds.spawn((
        Name::new("Circle"),
        RenderSdfBundle::new().with_pos([0., 0.]),
        Point,
        Added(50.),
        Fill(css::RED.into()),
    ));

    cmds.spawn((
        Name::new("Donut"),
        RenderSdfBundle::new().with_pos([0., 200.]),
        Point,
        Added(35.),
        Annular(15.),
        Fill(css::REBECCA_PURPLE.into()),
    ));

    cmds.spawn((
        Name::new("Gradient Circle"),
        RenderSdfBundle::new().with_pos([0., -200.]),
        Point,
        Added(50.),
        Fill(css::BLACK.into()),
        Gradient {
            color: css::MEDIUM_ORCHID.into(),
            intervall: 1.,
        },
    ));

    cmds.spawn((
        Name::new("Rounded Square"),
        RenderSdfBundle::new().with_pos([-200., -200.]),
        Rectangle(Vec2::new(30., 30.)),
        Added(20.),
        Fill(css::TURQUOISE.into()),
    ));

    cmds.spawn((
        Name::new("Square"),
        RenderSdfBundle::new().with_pos([-200., 0.]),
        Rectangle(Vec2::new(50., 50.)),
        Fill(css::GREEN.into()),
    ));

    cmds.spawn((
        Name::new("Rectangle"),
        RenderSdfBundle::new().with_pos([-200., 200.]),
        Rectangle(Vec2::new(70., 30.)),
        Fill(css::LAWN_GREEN.into()),
    ));

    cmds.spawn((
        Name::new("Annular Square"),
        RenderSdfBundle::new().with_pos([-400., 200.]),
        Rectangle(Vec2::new(30., 30.)),
        Added(10.),
        Annular(10.),
        Fill(css::STEEL_BLUE.into()),
    ));

    cmds.spawn((
        Name::new("Rotated Square"),
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

    cmds.spawn((
        Name::new("Gradient Line"),
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

    cmds.spawn((
        Name::new("Line"),
        RenderSdfBundle::new().with_pos([200., 0.]),
        Line(32.),
        Added(32.),
        Fill(css::BLUE.into()),
    ));

    cmds.spawn((
        Name::new("Annular Line"),
        RenderSdfBundle::new().with_pos([200., -200.]),
        Line(32.),
        Annular(10.),
        Added(22.),
        Fill(css::LIGHT_YELLOW.into()),
    ));
}
