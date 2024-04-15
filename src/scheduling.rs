use bevy::prelude::*;
use ComdfRenderUpdateSet::*;
use ComdfRenderPostUpdateSet::*;

pub fn plugin(app: &mut App) {
    app.configure_sets(
        Update,
        (
            BuildVariantFlags,
            AssignVariantBindings,
            (AssignVariantIndices, BuildRenderSdfKeys),
        )
            .chain(),
    );

    app.configure_sets(
        PostUpdate,
        (
            BuildShaders,
            GatherDataForExtract,
        )
    );

}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ComdfRenderUpdateSet {
    BuildVariantFlags,
    AssignVariantBindings,
    AssignVariantIndices,
    BuildRenderSdfKeys,
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ComdfRenderPostUpdateSet {
    BuildShaders,
    GatherDataForExtract,
}
