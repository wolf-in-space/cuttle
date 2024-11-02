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
    cmds.spawn(Camera2d);

    cmds.spawn((
        Name::new("Circle"),
        WorldSdf,
        Point::default(),
        Rounded { rounded: 50. },
        Fill(css::RED),
    ));

    cmds.spawn((
        Name::new("Donut"),
        WorldSdf,
        Transform::from_xyz(0., 200., 0.),
        Point::default(),
        Rounded { rounded: 15. },
        Annular { annular: 15. },
        Fill(css::REBECCA_PURPLE),
    ));

    cmds.spawn((
        Name::new("Gradient Circle"),
        WorldSdf,
        Transform::from_xyz(0., -200., 0.),
        Point::default(),
        Rounded { rounded: 50. },
        Fill(css::BLACK),
        // Gradient {
        //     color: css::MEDIUM_ORCHID.into(),
        //     intervall: 1.,
        // },
    ));

    cmds.spawn((
        Name::new("Rounded Square"),
        WorldSdf,
        Transform::from_xyz(-200., -200., 0.),
        Quad {
            half_size: Vec2::splat(30.),
        },
        Rounded { rounded: 20. },
        Fill(css::TURQUOISE),
    ));

    cmds.spawn((
        Name::new("Square"),
        WorldSdf,
        Transform::from_xyz(-200., 0., 0.),
        Quad {
            half_size: Vec2::splat(50.),
        },
        Fill(css::GREEN),
    ));

    cmds.spawn((
        Name::new("Rectangle"),
        WorldSdf,
        Transform::from_xyz(-200., 200., 0.),
        Quad {
            half_size: Vec2::new(70., 30.),
        },
        Fill(css::LAWN_GREEN),
    ));

    cmds.spawn((
        Name::new("Annular Square"),
        WorldSdf,
        Transform::from_xyz(-400., 200., 0.),
        Quad {
            half_size: Vec2::splat(30.),
        },
        Rounded { rounded: 10. },
        Annular { annular: 10. },
        Fill(css::STEEL_BLUE),
    ));

    cmds.spawn((
        Name::new("Rotated Square"),
        WorldSdf,
        Transform::from_xyz(-400., 0., 0.).with_rotation(Quat::from_rotation_z(PI * 0.25)),
        Quad {
            half_size: Vec2::splat(30.),
        },
        Rounded { rounded: 20. },
        Fill(css::ROYAL_BLUE),
    ));

    cmds.spawn((
        Name::new("Gradient Line"),
        WorldSdf,
        Transform::from_xyz(200., 200., 0.),
        Line { length: 32. },
        Annular { annular: 10. },
        Rounded { rounded: 22. },
        Fill(css::CADET_BLUE),
        // Gradient {
        //     color: css::BLACK.into(),
        //     intervall: 1.25,
        // },
    ));

    cmds.spawn((
        Name::new("Line"),
        WorldSdf,
        Transform::from_xyz(200., 0., 0.),
        Line { length: 32. },
        Rounded { rounded: 32. },
        Fill(css::BLUE),
    ));

    cmds.spawn((
        Name::new("Annular Line"),
        WorldSdf,
        Transform::from_xyz(-400., -200., 0.),
        Line { length: 32. },
        Annular { annular: 10. },
        Rounded { rounded: 22. },
        Fill(css::LIGHT_YELLOW),
    ));
}
