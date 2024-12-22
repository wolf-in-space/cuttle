use bevy::prelude::*;
use cuttle::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CuttlePlugin))
        .add_systems(Startup, spawn)
        .run();
}

const AXIS_AMOUNT: u32 = 1000;
const AMOUNT: u32 = AXIS_AMOUNT * AXIS_AMOUNT;
const X_SIZE: f32 = 1200.;
const X_HALF_SIZE: f32 = X_SIZE / 2.;
const Y_SIZE: f32 = 700.;
const Y_HALF_SIZE: f32 = Y_SIZE / 2.;
const X_DISTANCE: f32 = X_SIZE / AXIS_AMOUNT as f32;
const Y_DISTANCE: f32 = Y_SIZE / AXIS_AMOUNT as f32;

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);
    let mut entities = Vec::with_capacity(AMOUNT as usize);
    for x in 0..AXIS_AMOUNT {
        for y in 0..AXIS_AMOUNT {
            let (x, y) = (x as f32, y as f32);
            entities.push((
                Sdf,
                Transform::from_xyz(
                    x * X_DISTANCE - X_HALF_SIZE,
                    y * Y_DISTANCE - Y_HALF_SIZE,
                    0.,
                ),
                builtins::Circle { radius: 0.5 },
                Fill(Srgba::new(
                    f32::sin(x / 100.) + 0.5,
                    f32::cos(y / 100.) + 0.5,
                    0.,
                    1.,
                )),
            ));
        }
    }
    println!("SPAWNED_CIRCLES={}", AMOUNT);
    cmds.spawn_batch(entities);
}
