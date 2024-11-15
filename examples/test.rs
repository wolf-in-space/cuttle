use bevy::{color::palettes::tailwind, prelude::*};
use bevy_comdf::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, spawn)
        .add_systems(Update, (move_boxes, move_balls, rotate))
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);

    box_op_circle::<Unioni>(&mut cmds, [-600., 400.]);
    box_op_circle::<SmoothUnion>(&mut cmds, [-600., 300.]);
}

fn box_op_circle<O: Default + Component>(cmds: &mut Commands, pos: impl Into<Vec2>) {
    let pos = pos.into();

    let sdf = cmds
        .spawn((
            WorldSdf,
            Transform::from_xyz(pos.x, pos.y, 0.),
            Point::default(),
            Rounded { rounded: 20. },
            Fill(tailwind::SKY_400),
            // Gradient {
            //     color: tailwind::NEUTRAL_200.into(),
            //     intervall: 1.,
            // },
        ))
        .id();
    cmds.spawn((
        ExtendSdf::new(sdf),
        Transform::from_xyz(pos.x + 15., pos.y + 15., 0.),
        Quad {
            half_size: Vec2::splat(15.),
        },
        Fill(tailwind::SKY_400),
        O::default(),
        MovingBox,
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
