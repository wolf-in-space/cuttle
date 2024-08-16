use bevy::{color::palettes::css, prelude::*};
use bevy_comdf::{
    implementations::operations::{Base, Subtract},
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
    cmds.sdf(RenderSdfBundle::new())
        .operation::<Base>((
            SdfBundle::default(),
            Point,
            Added(50.),
            Fill(css::SKY_BLUE.into()),
        ))
        .operation::<Subtract>((
            SdfBundle::default().with_pos([35., 10.]),
            Rectangle(Vec2::new(30., 30.)),
        ));
}
