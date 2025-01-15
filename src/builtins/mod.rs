use bevy::prelude::*;

pub mod sdf;
pub struct BuiltinsPlugin;
impl Plugin for BuiltinsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(sdf::plugin);
    }
}
