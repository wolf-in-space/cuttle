use bevy::prelude::*;
use bevy_comdf::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, spawn)
        .run();
}

const AMOUNT: u32 = 1000;

const RATIO: u32 = 3;
const X_AMOUNT: u32 = (AMOUNT / 10) * (10 - RATIO);
const Y_AMOUNT: u32 = (AMOUNT / 10) * RATIO;
const X_SIZE: f32 = 1200.;
const X_HALF_SIZE: f32 = X_SIZE / 2.;
const Y_SIZE: f32 = 700.;
const Y_HALF_SIZE: f32 = Y_SIZE / 2.;
const X_DISTANCE: f32 = X_SIZE / X_AMOUNT as f32;
const Y_DISTANCE: f32 = Y_SIZE / Y_AMOUNT as f32;

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);
    let mut entities = Vec::with_capacity(AMOUNT as usize);
    println!("SPAWNED_CIRCLES={}", X_AMOUNT * Y_AMOUNT);
    for x in 0..X_AMOUNT {
        for y in 0..Y_AMOUNT {
            let (x, y) = (x as f32, y as f32);
            entities.push((
                WorldSdf,
                Transform::from_xyz(
                    x * X_DISTANCE - X_HALF_SIZE,
                    y * Y_DISTANCE - Y_HALF_SIZE,
                    x * y,
                ),
                Point::default(),
                Rounded { rounded: 1. },
                Fill(Color::srgb(
                    f32::sin(x / 100.) + 0.5,
                    f32::cos(y / 100.) + 0.5,
                    0.,
                )),
            ));
        }
    }
    cmds.spawn_batch(entities);
}
