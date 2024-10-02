use bevy::{color::palettes::css, core_pipeline::core_2d::Transparent2d, prelude::*};
use bevy_comdf::{pipeline::UsePipeline, prelude::*};

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
    cmds.spawn(Camera2dBundle::default());
    let mut entities = Vec::with_capacity(AMOUNT as usize);
    for x in 0..X_AMOUNT {
        for y in 0..Y_AMOUNT {
            entities.push((
                RenderSdf::<Transparent2d>::new(UsePipeline::World),
                RenderSdfBundle::default().with_pos([
                    x as f32 * X_DISTANCE - X_HALF_SIZE,
                    y as f32 * Y_DISTANCE - Y_HALF_SIZE,
                ]),
                Point,
                Added(1.),
                Fill(css::SKY_BLUE.into()),
            ));
        }
    }
    cmds.spawn_batch(entities);
}
