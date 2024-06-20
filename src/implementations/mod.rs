use bevy::app::App;

pub mod calculations;
pub mod components;
pub mod operations;

pub fn plugin(app: &mut App) {
    app.add_plugins((calculations::plugin, components::plugin, operations::plugin));
}
