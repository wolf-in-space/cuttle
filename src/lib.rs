use bevy_app::{App, Plugin};
use cuttle_core::CuttleCorePlugin;
use cuttle_sdf::SdfPlugin;

pub mod prelude {
    pub use crate::CuttlePlugin;
    pub use cuttle_core::prelude::*;
    pub use cuttle_macros::Cuttle;
    pub use cuttle_sdf::*;
}

pub struct CuttlePlugin;
impl Plugin for CuttlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CuttleCorePlugin);
        app.add_plugins(
            #[cfg(feature = "sdf")]
            SdfPlugin,
        );
    }
}
