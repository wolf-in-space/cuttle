use bevy::{
    color::palettes::{css, tailwind},
    prelude::*,
};
use bevy_comdf::{
    implementations::operations::{Base, Subtract, Union},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_comdf::plugin))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());

    cmds.sdf((
        RenderSdfBundle::default(),
        Gradient {
            color: tailwind::NEUTRAL_200.into(),
            intervall: 1.,
        },
    ))
    .operation::<Base>((
        SdfBundle::default(),
        Point,
        Added(50.),
        Fill(tailwind::SKY_400.into()),
    ))
    .operation::<Union>((
        SdfBundle::default().with_pos([35., 10.]),
        Rectangle(Vec2::new(30., 30.)),
        Fill(tailwind::SKY_400.into()),
    ));

    cmds.sdf((
        RenderSdfBundle::default(),
        Gradient {
            color: tailwind::NEUTRAL_200.into(),
            intervall: 1.,
        },
    ))
    .operation::<Base>((
        SdfBundle::default().with_pos([0., 200.]),
        Line(25.),
        Added(15.),
        Fill(css::RED.into()),
    ))
    .operation::<Subtract>((
        SdfBundle::default().with_pos([35., 210.]),
        Point,
        Added(20.),
        Fill(css::SEA_GREEN.into()),
    ));
}
