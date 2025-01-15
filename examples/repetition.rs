use bevy::{color::palettes::css, input::common_conditions::input_just_pressed, prelude::*};
use cuttle::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CuttlePlugin))
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            (
                (|mut rep: Single<&mut Repetition>| rep.repetitions += 1.)
                    .run_if(input_just_pressed(KeyCode::KeyX)),
                (|mut rep: Single<&mut Repetition>| rep.repetitions -= 1.)
                    .run_if(input_just_pressed(KeyCode::KeyZ)),
                (|mut rep: Single<&mut Repetition>| rep.scale += 0.1)
                    .run_if(input_just_pressed(KeyCode::KeyS)),
                (|mut rep: Single<&mut Repetition>| rep.scale -= 0.1)
                    .run_if(input_just_pressed(KeyCode::KeyA)),
                // (|rep: Single<&mut Repetition>| {
                //     dbg!(rep.into_inner());
                // }),
            ),
        )
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);

    cmds.spawn((Sdf, Circle(10.), Repetition::default(), Fill(css::RED)));
}
