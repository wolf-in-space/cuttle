use bevy::color::palettes::css;
use bevy::{color::palettes::tailwind, prelude::*};
use cuttle::extensions::ExtendSdf;
use cuttle::prelude::*;
use std::f32::consts::PI;
use bevy::window::WindowResolution;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(1920., 1080.),
                    decorations: false,
                    ..default()
                }),
                ..default()
            }),
            CuttlePlugin,
        ))
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            (
                move_boxes,
                move_balls,
                rotate,
                animate_morph,
                animate_repetition,
            ),
        )
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);

    morph(&mut cmds, [100., 250.], 0.3);
    morph(&mut cmds, [200., 250.], 0.5);
    morph(&mut cmds, [300., 250.], 0.8);
    morph(&mut cmds, [400., 250.], 1.0);
    morph(&mut cmds, [500., 250.], 1.5);
    morph(&mut cmds, [600., 250.], 2.0);
    morph(&mut cmds, [700., 250.], 3.0);

    morph2(&mut cmds, [100., 100.], 0.3);
    morph2(&mut cmds, [200., 100.], 0.5);
    morph2(&mut cmds, [300., 100.], 0.8);
    morph2(&mut cmds, [400., 100.], 1.0);
    morph2(&mut cmds, [500., 100.], 1.5);
    morph2(&mut cmds, [600., 100.], 2.0);
    morph2(&mut cmds, [700., 100.], 3.0);

    box_op_circle::<Unioni>(&mut cmds, [-100., 250.]);
    box_op_circle::<SmoothUnion>(&mut cmds, [-100., 100.]);
    box_op_circle::<Subtract>(&mut cmds, [-250., 250.]);
    box_op_circle::<SmoothSubtract>(&mut cmds, [-250., 100.]);
    box_op_circle::<Intersect>(&mut cmds, [-400., 250.]);
    box_op_circle::<SmoothIntersect>(&mut cmds, [-400., 100.]);
    box_op_circle::<Xor>(&mut cmds, [-550., 250.]);
    box_op_circle::<SmoothXor>(&mut cmds, [-550., 100.]);

    spin::<SmoothUnion>(&mut cmds, -500., -50., |cmds, x, y| {
        cmds.spawn((
            WorldSdf,
            Quad {
                half_size: Vec2::new(15., 220.),
            },
            Transform::from_xyz(x, y - 40. * 5., 0.),
            Fill(tailwind::GRAY_100),
            Rotate { speed: 0.2 },
        ))
        .id()
    });

    spin::<SmoothSubtract>(&mut cmds, 0., -50., |cmds, x, y| {
        cmds.spawn((
            WorldSdf,
            Quad {
                half_size: Vec2::new(100., 220.),
            },
            Transform::from_xyz(x, y - 40. * 5., 0.),
            Fill(tailwind::GRAY_100),
            Rotate { speed: 0.2 },
        ))
        .id()
    });

    cmds.spawn((
        WorldSdf,
        Transform::from_xyz(500., -250., -100.),
        Point,
        Rounded { rounded: 10. },
        Fill(css::RED),
        Repetition {
            repetitions: Vec2::new(3., 5.),
            scale: 1.,
        },
        Rotate { speed: 0.3 },
        AnimateRepetitionDistance {
            speed: 1.,
            scale: 0.7,
        },
    ));
}

#[derive(Component)]
struct AnimateRepetitionDistance {
    speed: f32,
    scale: f32,
}

fn animate_repetition(
    mut query: Query<(&mut Repetition, &AnimateRepetitionDistance)>,
    time: Res<Time>,
) {
    for (mut repetition, animate) in &mut query {
        repetition.scale =
            (time.elapsed_secs() * animate.speed).sin() * animate.scale + 1. + animate.scale;
    }
}

fn morph(cmds: &mut Commands, pos: impl Into<Vec2>, scale: f32) {
    let pos = pos.into().extend(0.);
    let quad = cmds
        .spawn((
            WorldSdf,
            Quad {
                half_size: Vec2::new(25., 25.),
            },
            Transform::from_translation(pos),
            Fill(tailwind::AMBER_400),
        ))
        .id();

    cmds.spawn((
        ExtendSdf::new(quad),
        Point,
        Rounded { rounded: 15. },
        Transform::from_translation(pos),
        Fill(tailwind::TEAL_400),
        Morph::default(),
        AnimateMorph { speed: 1., scale },
    ));
}

fn morph2(cmds: &mut Commands, pos: impl Into<Vec2>, scale: f32) {
    let pos = pos.into().extend(0.);
    let quad = cmds
        .spawn((
            WorldSdf,
            Line { length: 30. },
            Rounded { rounded: 15. },
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(PI * 0.5)),
            Fill(tailwind::RED_700),
        ))
        .id();

    cmds.spawn((
        ExtendSdf::new(quad),
        Quad { half_size: Vec2::splat(20.) },
        Transform::from_translation(pos),
        Fill(tailwind::BLUE_700),
        Morph::default(),
        AnimateMorph { speed: 1., scale },
    ));
}

#[derive(Component)]
struct AnimateMorph {
    speed: f32,
    scale: f32,
}

fn animate_morph(mut morphs: Query<(&AnimateMorph, &mut Morph)>, time: Res<Time>) {
    for (animate, mut morph) in &mut morphs {
        morph.morph = (time.elapsed_secs() * animate.speed).sin() * animate.scale * 0.5 + 0.5;
    }
}

fn spin<OP: Default + Component>(
    cmds: &mut Commands,
    x: f32,
    y: f32,
    spin: fn(&mut Commands, f32, f32) -> Entity,
) {
    let sdf = spin(cmds, x, y);

    let make_ball = |pos: f32, color: Srgba, offset: f32| {
        (
            ExtendSdf::new(sdf),
            Transform::from_xyz(x, pos, 0.),
            Point,
            Rounded { rounded: 10. },
            Fill(color),
            MovingBall { offset, start: x },
            OP::default(),
        )
    };

    [
        (make_ball(y - 40. * 0., tailwind::GREEN_400, 0.)),
        (make_ball(y - 40. * 1., tailwind::RED_400, 0.3)),
        (make_ball(y - 40. * 2., tailwind::TEAL_400, 0.6)),
        (make_ball(y - 40. * 3., tailwind::SKY_400, 0.9)),
        (make_ball(y - 40. * 4., tailwind::EMERALD_400, 1.2)),
        (make_ball(y - 40. * 5., tailwind::ZINC_400, 1.5)),
        (make_ball(y - 40. * 6., tailwind::FUCHSIA_400, 1.8)),
    ]
    .map(|bundle| {
        cmds.spawn(bundle);
    });

    cmds.spawn((
        ExtendSdf::new(sdf),
        Transform::from_xyz(x, y - 40. * 7., 0.),
        Quad {
            half_size: Vec2::splat(10.),
        },
        Fill(tailwind::GREEN_400),
        MovingBall {
            offset: 2.1,
            start: x,
        },
        OP::default(),
    ));

    cmds.spawn((
        ExtendSdf::new(sdf),
        Quad {
            half_size: Vec2::splat(10.),
        },
        Transform::from_xyz(x, y - 40. * 8., 0.),
        Fill(tailwind::GREEN_400),
        MovingBall {
            offset: 2.4,
            start: x,
        },
        Rotate { speed: 5. },
        OP::default(),
    ));

    cmds.spawn((
        ExtendSdf::new(sdf),
        Transform::from_xyz(x, y - 40. * 9., 0.),
        Point,
        Rounded { rounded: 7. },
        Fill(tailwind::GREEN_400),
        MovingBall {
            offset: 2.7,
            start: x,
        },
        Annular { annular: 3. },
        OP::default(),
    ));

    cmds.spawn((
        ExtendSdf::new(sdf),
        Transform::from_xyz(x, y - 40. * 10., 0.),
        Quad {
            half_size: Vec2::splat(7.),
        },
        Annular { annular: 3. },
        Fill(tailwind::GREEN_400),
        MovingBall {
            offset: 3.0,
            start: x,
        },
        OP::default(),
    ));
}

fn box_op_circle<O: Default + Component>(cmds: &mut Commands, pos: impl Into<Vec2>) {
    let pos = pos.into();

    let sdf = cmds
        .spawn((
            WorldSdf,
            Transform::from_xyz(pos.x, pos.y, 0.),
            Point,
            Rounded { rounded: 30. },
            Fill(tailwind::SKY_400),
            // Gradient {
            //     color: tailwind::NEUTRAL_200.into(),
            //     intervall: 1.,
            // },
        ))
        .id();
    cmds.spawn((
        ExtendSdf::new(sdf),
        Transform::from_xyz(pos.x, pos.y, 0.),
        Quad {
            half_size: Vec2::splat(25.),
        },
        Fill(tailwind::FUCHSIA_400),
        O::default(),
        MovingBox,
        DistanceGradient {
            interval: 1.,
            color: Vec3::ZERO,
        },
    ));
}

#[derive(Component)]
struct MovingBox;

fn move_boxes(mut query: Query<&mut Transform, With<MovingBox>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.translation += time.elapsed_secs().cos() * 1.;
    }
}

#[derive(Component)]
struct MovingBall {
    start: f32,
    offset: f32,
}

fn move_balls(mut query: Query<(&mut Transform, &MovingBall)>, time: Res<Time>) {
    for (mut transform, ball) in &mut query {
        transform.translation.x = (time.elapsed_secs() + ball.offset).cos() * 100. + ball.start;
    }
}

#[derive(Component)]
struct Rotate {
    speed: f32,
}

fn rotate(mut query: Query<(&mut Transform, &Rotate)>, time: Res<Time>) {
    for (mut transform, rotate) in &mut query {
        transform.rotate_z(rotate.speed * time.delta_secs());
    }
}
