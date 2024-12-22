use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use cuttle::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CuttlePlugin))
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut cmds: Commands) {
    cmds.spawn(Camera2d);
    cmds.spawn((
        UiSdf,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Quad {
            half_size: Vec2::new(50., 20.),
        },
        Fill(tailwind::SKY_800),
    ))
    .with_children(|c| {
        c.spawn((
            UiSdf,
            Node {
                width: Val::Percent(10.0),
                height: Val::Percent(10.0),
                ..default()
            },
            builtins::Circle { radius: 15. },
            BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.1)),
            Fill(Srgba::new (1.0, 0.0, 0.0, 1.0)),
        ));

        c.spawn((
            UiSdf,
            Node {
                width: Val::Percent(10.0),
                height: Val::Percent(10.0),
                ..default()
            },
            builtins::Circle { radius: 10. },
            Annular { annular: 5. },
            BackgroundColor(Color::srgba(0.0, 1.0, 0.0, 0.1)),
            Fill(Srgba::new (0.0, 1.0, 0.0, 1.0)),
        ));

        c.spawn((
            UiSdf,
            Node {
                width: Val::Percent(10.0),
                height: Val::Percent(10.0),
                ..default()
            },
            builtins::Circle { radius: 20. },
            BackgroundColor(Color::srgba(0.0, 0.0, 1.0, 0.1)),
            Fill(Srgba::new (0.0, 0.0, 1.0, 1.0)),
        ));
    });
}
