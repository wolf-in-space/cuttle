use bevy::{color::palettes::tailwind, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_comdf::{
    implementations::operations::{
        Base, Intersect, SmoothIntersect, SmoothSubtract, SmoothUnion, Subtract, Union,
    },
    operations::Operation,
    prelude::*,
};
use bevy_editor_pls::EditorPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EditorPlugin::new(),
            FrameTimeDiagnosticsPlugin,
            bevy_comdf::plugin,
        ))
        .add_systems(Startup, spawn)
        .add_systems(Update, (move_boxes, move_balls, rotate))
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());

    box_op_circle::<Union>(&mut cmds, [0., 0.]);
    box_op_circle::<SmoothUnion>(&mut cmds, [0., 200.]);
    box_op_circle::<Subtract>(&mut cmds, [200., 0.]);
    box_op_circle::<SmoothSubtract>(&mut cmds, [200., 200.]);
    box_op_circle::<Intersect>(&mut cmds, [-200., 0.]);
    box_op_circle::<SmoothIntersect>(&mut cmds, [-200., 200.]);

    spin::<SmoothUnion>(&mut cmds, -400.);
    spin::<SmoothIntersect>(&mut cmds, 400.);
}

fn spin<OP: Operation>(cmds: &mut Commands, x: f32) {
    let make_ball = |pos: f32, color: Srgba, offset: f32| {
        (
            SdfBundle::default().with_pos([x, pos]),
            Point,
            Added(10.),
            Fill(color.into()),
            MovingBall { offset, start: x },
        )
    };

    cmds.sdf(RenderSdfBundle::default())
        .operation::<Base>((
            SdfBundle::default().with_pos([x, -320.]),
            Rectangle(Vec2::new(15., 220.)),
            Fill(tailwind::AMBER_400.into()),
            Rotate { speed: 0.5 },
        ))
        .operation::<OP>(make_ball(-120., tailwind::GREEN_400, 0.))
        .operation::<OP>(make_ball(-160., tailwind::RED_400, 0.3))
        .operation::<OP>(make_ball(-200., tailwind::TEAL_400, 0.6))
        .operation::<OP>(make_ball(-240., tailwind::SKY_400, 0.9))
        .operation::<OP>(make_ball(-280., tailwind::EMERALD_400, 1.2))
        .operation::<OP>(make_ball(-320., tailwind::ZINC_400, 1.5))
        .operation::<OP>(make_ball(-360., tailwind::FUCHSIA_400, 1.8))
        .operation::<OP>((
            SdfBundle::default().with_pos([x, -400.]),
            Rectangle(Vec2::splat(10.)),
            Fill(tailwind::GREEN_400.into()),
            MovingBall {
                offset: 2.1,
                start: x,
            },
        ))
        .operation::<OP>((
            SdfBundle::default().with_pos([x, -440.]),
            Rectangle(Vec2::splat(10.)),
            Fill(tailwind::GREEN_400.into()),
            MovingBall {
                offset: 2.4,
                start: x,
            },
            Rotate { speed: 5. },
        ))
        .operation::<OP>((
            SdfBundle::default().with_pos([x, -480.]),
            Point,
            Added(7.),
            Fill(tailwind::GREEN_400.into()),
            MovingBall {
                offset: 2.7,
                start: x,
            },
            Annular(3.),
        ))
        .operation::<OP>((
            SdfBundle::default().with_pos([x, -520.]),
            Rectangle(Vec2::splat(7.)),
            Annular(3.),
            Fill(tailwind::GREEN_400.into()),
            MovingBall {
                offset: 3.0,
                start: x,
            },
        ));
}

fn box_op_circle<O: Operation>(cmds: &mut Commands, pos: impl Into<Vec2>) {
    let pos = pos.into();
    cmds.sdf((
        RenderSdfBundle::default(),
        Gradient {
            color: tailwind::NEUTRAL_200.into(),
            intervall: 1.,
        },
    ))
    .operation::<Base>((
        SdfBundle::default().with_pos(pos),
        Point,
        Added(50.),
        Fill(tailwind::SKY_400.into()),
    ))
    .operation::<O>((
        SdfBundle::default().with_pos(pos + 25.),
        Rectangle(Vec2::new(30., 30.)),
        Fill(tailwind::SKY_400.into()),
        MovingBox,
    ));
}

#[derive(Component)]
struct MovingBox;

fn move_boxes(mut query: Query<&mut Transform, With<MovingBox>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.translation += time.elapsed_seconds().cos() * 1.3;
    }
}

#[derive(Component)]
struct MovingBall {
    start: f32,
    offset: f32,
}

fn move_balls(mut query: Query<(&mut Transform, &MovingBall)>, time: Res<Time>) {
    for (mut transform, ball) in &mut query {
        transform.translation.x = (time.elapsed_seconds() + ball.offset).cos() * 100. + ball.start;
    }
}

#[derive(Component)]
struct Rotate {
    speed: f32,
}

fn rotate(mut query: Query<(&mut Transform, &Rotate)>, time: Res<Time>) {
    for (mut transform, rotate) in &mut query {
        transform.rotate_z(rotate.speed * time.delta_seconds());
    }
}
