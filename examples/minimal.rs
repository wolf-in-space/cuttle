use bevy::{
    color::palettes::css,
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cuttle::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CuttlePlugin, WorldInspectorPlugin::new()))
        .add_systems(Startup, spawn)
        .edit_schedule(PostUpdate, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);
    cmds.spawn((Sdf, Transform::default(), Circle(50.), Fill(css::SKY_BLUE)));
}
