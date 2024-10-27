use bevy::{color::palettes::tailwind, prelude::*};
use bevy_comdf::prelude::*;
use operations::ExtendSdf;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // EditorPlugin::new(),
            // FrameTimeDiagnosticsPlugin,
            bevy_comdf::plugin,
        ))
        .add_systems(Startup, spawn)
        .add_systems(Update, (move_boxes, move_balls, rotate))
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);

    box_op_circle::<Unioni>(&mut cmds, [0., 0.]);
    box_op_circle::<SmoothUnion>(&mut cmds, [0., 200.]);
    box_op_circle::<Subtract>(&mut cmds, [200., 0.]);
    box_op_circle::<SmoothSubtract>(&mut cmds, [200., 200.]);
    box_op_circle::<Intersect>(&mut cmds, [-200., 0.]);
    box_op_circle::<SmoothIntersect>(&mut cmds, [-200., 200.]);

    spin::<SmoothUnion>(&mut cmds, -400.);
    spin::<SmoothIntersect>(&mut cmds, 400.);
}

fn spin<OP: Default + Component>(cmds: &mut Commands, x: f32) {
    let sdf = cmds
        .spawn((
            WorldSdf,
            Quad {
                half_size: Vec2::new(15., 220.),
            },
            Transform::from_xyz(x, -320., 0.),
            Fill(tailwind::AMBER_400.into()),
            Rotate { speed: 0.1 },
        ))
        .id();

    let make_ball = |pos: f32, color: Srgba, offset: f32| {
        (
            ExtendSdf::new(sdf),
            Transform::from_xyz(x, pos, 0.),
            Point { hi: 10. },
            Rounded { rounded: 10. },
            Fill(color.into()),
            MovingBall { offset, start: x },
            OP::default(),
        )
    };

    [
        (make_ball(-120., tailwind::GREEN_400, 0.)),
        (make_ball(-160., tailwind::RED_400, 0.3)),
        (make_ball(-200., tailwind::TEAL_400, 0.6)),
        (make_ball(-240., tailwind::SKY_400, 0.9)),
        (make_ball(-280., tailwind::EMERALD_400, 1.2)),
        (make_ball(-320., tailwind::ZINC_400, 1.5)),
        (make_ball(-360., tailwind::FUCHSIA_400, 1.8)),
    ]
    .map(|bundle| {
        cmds.spawn(bundle);
    });

    cmds.spawn((
        ExtendSdf::new(sdf),
        Transform::from_xyz(x, -400., 0.),
        Quad {
            half_size: Vec2::splat(10.),
        },
        Fill(tailwind::GREEN_400.into()),
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
        Transform::from_xyz(x, -440., 0.),
        Fill(tailwind::GREEN_400.into()),
        MovingBall {
            offset: 2.4,
            start: x,
        },
        Rotate { speed: 5. },
        OP::default(),
    ));

    cmds.spawn((
        ExtendSdf::new(sdf),
        Transform::from_xyz(x, -480., 0.),
        Point::default(),
        Rounded { rounded: 7. },
        Fill(tailwind::GREEN_400.into()),
        MovingBall {
            offset: 2.7,
            start: x,
        },
        Annular { annular: 3. },
        OP::default(),
    ));

    cmds.spawn((
        ExtendSdf::new(sdf),
        Transform::from_xyz(x, -520., 0.),
        Quad {
            half_size: Vec2::splat(7.),
        },
        Annular { annular: 3. },
        Fill(tailwind::GREEN_400.into()),
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
            Point::default(),
            Rounded { rounded: 50. },
            Fill(tailwind::SKY_400.into()),
            // Gradient {
            //     color: tailwind::NEUTRAL_200.into(),
            //     intervall: 1.,
            // },
        ))
        .id();
    cmds.spawn((
        ExtendSdf::new(sdf),
        Transform::from_xyz(pos.x + 25., pos.y + 25., 0.),
        Quad {
            half_size: Vec2::splat(30.),
        },
        Fill(tailwind::SKY_400.into()),
        O::default(),
        MovingBox,
    ));
}

#[derive(Component)]
struct MovingBox;

fn move_boxes(mut query: Query<&mut Transform, With<MovingBox>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.translation += time.elapsed_secs().cos() * 1.3;
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
