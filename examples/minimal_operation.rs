use bevy::window::Monitor;
use bevy::{color::palettes::css, prelude::*};
use bevy_inspector_egui::quick::FilterQueryInspectorPlugin;
use cuttle::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CuttlePlugin,
            FilterQueryInspectorPlugin::<(Without<Observer>, Without<Monitor>)>::new(),
        ))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);

    let subtract = cmds
        .spawn((
            Sdf,
            Transform::from_xyz(35., 10., 0.),
            Quad(Vec2::splat(30.)),
            Fill(css::REBECCA_PURPLE),
        ))
        .id();

    cmds.spawn((
        Extension::new(subtract),
        Transform::default(),
        Circle(75.),
        Fill(css::SKY_BLUE),
        Intersect,
    ));
}
