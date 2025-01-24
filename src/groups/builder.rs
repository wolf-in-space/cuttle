use crate::bounding::{make_compute_aabb_system, Bounding};
use crate::calculations::Calculation;
use crate::components::initialization::{
    init_render_data, ComponentOrder, CuttleRenderData, CuttleStructComponent,
    CuttleWrapperComponent,
};
use crate::components::ComponentInfo;
use crate::groups::global::GlobalGroupInfos;
use crate::groups::{CuttleGroup, GroupData, GroupPlugin};
use crate::shader::wgsl_struct::ToWgslFn;
use crate::shader::{AddSnippet, ToComponentShaderInfo, ToRenderDataShaderInfo};
use bevy::app::{App, PostUpdate};
use bevy::prelude::Component;
use bevy::reflect::Typed;
use convert_case::{Case, Casing};
use std::any::{type_name, TypeId};
use std::mem;

pub struct CuttleGroupBuilder<'a, G: CuttleGroup> {
    pub(crate) group: GroupData<G>,
    pub(crate) app: &'a mut App,
}

impl<G: CuttleGroup> CuttleGroupBuilder<'_, G> {
    pub fn calculation(
        &mut self,
        name: impl Into<String>,
        wgsl_type: impl Into<String>,
    ) -> &mut Self {
        self.group.calculations.push(Calculation {
            name: name.into(),
            wgsl_type: wgsl_type.into(),
        });
        self
    }

    /// Adds a snippet of wgsl code to the shader generated for this Group
    ///
    /// ```
    /// # use bevy::core_pipeline::core_2d::Transparent2d;
    /// # use bevy::prelude::*;
    /// # use cuttle::groups::{CuttleGroup, builder::CuttleGroupBuilderAppExt};
    /// # let mut app = App::new();
    /// # #[derive(Component, Default)]
    /// # struct MyGroup;
    /// # impl CuttleGroup for MyGroup {
    /// #     type Phase = Transparent2d;
    /// # }
    ///
    /// app.cuttle_group::<MyGroup>()
    /// .snippet(stringify!(
    ///     fn my_component(input: MyComponent) {
    ///         distance += input.value;
    ///     }
    /// ));
    ///
    /// ```
    pub fn snippet(&mut self, snippet: impl Into<String>) -> &mut Self {
        self.group.snippets.push(AddSnippet::Inline(snippet.into()));
        self
    }

    /// Takes a file path to a wgsl file to be added to the shader
    /// generated for this Group.
    /// Supports hot reloading.
    /// ```
    /// # use bevy::core_pipeline::core_2d::Transparent2d;
    /// # use bevy::prelude::*;
    /// # use cuttle::groups::{CuttleGroup, builder::CuttleGroupBuilderAppExt};
    /// # let mut app = App::new();
    /// # #[derive(Component, Default)]
    /// # struct MyGroup;
    /// # impl CuttleGroup for MyGroup {
    /// #     type Phase = Transparent2d;
    /// # }
    ///
    /// app.cuttle_group::<MyGroup>()
    /// // Adds an embedded file to the Group.
    /// // Can be hot reloaded if bevy`s 'embedded_watcher' feature is enabled.
    /// .snippet_file("embedded://cuttle/builtins/builtins.wgsl")
    /// // Adds a file from assets to the Group.
    /// // Can be hot reloaded if bevy`s 'file_watcher' feature is enabled.
    /// .snippet_file("groups/my_group.wgsl");
    /// ```
    ///
    /// see [`builtins.wgsl`](https://github.com/wolf-in-space/cuttle/blob/main/src/builtins/builtins.wgsl) for an example
    pub fn snippet_file(&mut self, path: impl Into<String>) -> &mut Self {
        self.group.snippets.push(AddSnippet::File(path.into()));
        self
    }

    /// Registers a component to affect any entity of this Group that it is added to
    ///
    /// ```
    /// # use bevy::prelude::{Component, Reflect};
    /// # use bevy::render::render_resource::ShaderType;
    ///
    /// #[derive(Component, Reflect, ShaderType, Clone, Debug)]
    /// struct MyComponent {
    ///     value: f32,
    /// }
    ///
    /// ```
    /// Example wgsl code for MyComponent:
    /// ```wgsl
    /// fn my_component(input: MyComponent) {
    ///     distance += input.value;
    /// }
    /// ```
    pub fn component<C: CuttleStructComponent>(&mut self, sort: impl Into<u32>) -> &mut Self {
        self.register_component_manual(sort, Some(C::wgsl_type), Some(C::to_render_data), None)
    }

    pub fn wrapper_component<C: CuttleWrapperComponent>(
        &mut self,
        sort: impl Into<u32>,
    ) -> &mut Self {
        self.register_component_manual(sort, Some(C::wgsl_type), Some(C::to_render_data), None)
    }

    /// Registers a marker component to work with this group.
    ///
    /// ```
    /// # use bevy::core_pipeline::core_2d::Transparent2d;
    /// # use bevy::prelude::{App, Component, Reflect};
    /// # use bevy::render::render_resource::ShaderType;
    /// # use cuttle::groups::{CuttleGroup, builder::CuttleGroupBuilderAppExt};
    /// # use cuttle::prelude::{Sdf, SdfOrder};
    /// # let mut app = App::new();
    /// # #[derive(Default, Component)]
    /// # struct MyGroup;
    /// # impl CuttleGroup for MyGroup { type Phase = Transparent2d; }
    ///
    /// #[derive(Component, Reflect, Clone, Debug)]
    /// struct Intersect;
    ///
    /// app.cuttle_group::<Sdf>()
    ///     .marker_component::<Intersect>(SdfOrder::Operations)
    ///     .snippet(stringify!(
    ///         fn my_marker_component() {
    ///             distance *= 2.0;
    ///         }
    ///     ));
    /// ```
    pub fn marker_component<C: Component + Typed>(&mut self, sort: impl Into<u32>) -> &mut Self {
        self.register_component_manual::<C, f32>(sort, None, None, None)
    }

    pub fn register_component_manual<C: Component + Typed, R: CuttleRenderData>(
        &mut self,
        sort: impl Into<u32>,
        to_wgsl: Option<ToWgslFn>,
        to_render_data: Option<fn(&C) -> R>,
        extension_override: Option<u8>,
    ) -> &mut Self {
        let Some(function_name) = C::type_ident().map(|i| i.to_case(Case::Snake)) else {
            panic!(
                "Registering Component '{}' is not a named type",
                type_name::<C>()
            );
        };

        let to_render_data = to_render_data
            .map(|to| init_render_data(self.app, to))
            .and_then(|binding| to_wgsl.map(|to_wgsl| ToRenderDataShaderInfo { binding, to_wgsl }));

        let order = ComponentOrder {
            sort: sort.into(),
            id: TypeId::of::<C>(),
            extension_override,
        };
        let to_shader_info = ToComponentShaderInfo {
            function_name,
            to_render_data,
        };
        self.group.component_infos.push(ComponentInfo {
            order,
            to_shader_info,
        });
        GlobalGroupInfos::register_component::<C>(self.app);
        self
    }

    pub fn affect_bounds<C: Component>(&mut self, set: Bounding, func: fn(&C) -> f32) -> &mut Self {
        self.app
            .add_systems(PostUpdate, make_compute_aabb_system(func, set));
        self
    }
}

impl<G: CuttleGroup> Drop for CuttleGroupBuilder<'_, G> {
    fn drop(&mut self) {
        self.app.insert_resource(mem::take(&mut self.group));
    }
}

pub trait CuttleGroupBuilderAppExt {
    fn cuttle_group<G: CuttleGroup>(&mut self) -> CuttleGroupBuilder<G>;
}

impl CuttleGroupBuilderAppExt for App {
    fn cuttle_group<G: CuttleGroup>(&mut self) -> CuttleGroupBuilder<G> {
        if !self.is_plugin_added::<GroupPlugin<G>>() {
            self.add_plugins(GroupPlugin::<G>::new());
        }
        let group = self.world_mut().remove_resource::<GroupData<G>>().unwrap();
        let mut builder = CuttleGroupBuilder { group, app: self };
        builder.calculation("color", "vec4<f32>");
        builder
    }
}
