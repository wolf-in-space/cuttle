use bevy::{color::palettes::css, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cuttle::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CuttlePlugin, WorldInspectorPlugin::new()))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);
    cmds.spawn((Sdf, Transform::default(), Circle(50.), Fill(css::SKY_BLUE)));
}
