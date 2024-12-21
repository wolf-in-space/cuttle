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
            builtins::Circle { radius: 15. },
            Fill(tailwind::EMERALD_400),
        ));

        c.spawn((
            UiSdf,
            builtins::Circle { radius: 10. },
            Annular { annular: 5. },
            Fill(tailwind::EMERALD_400),
        ));

        c.spawn((
            UiSdf,
            builtins::Circle { radius: 15. },
            Fill(tailwind::EMERALD_400),
        ));
    });
}
