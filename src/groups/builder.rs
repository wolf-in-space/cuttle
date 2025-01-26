use crate::bounding::{make_compute_aabb_system, Bounding};
use crate::calculations::{Calculation, Calculations};
use crate::components::buffer::GlobalBuffer;
use crate::components::initialization::{init_render_data, ComponentOrder, Cuttle};
use crate::components::{ComponentInfo, ComponentInfos};
use crate::groups::global::GlobalGroupInfos;
use crate::groups::{initialize_config, CuttleConfig};
use crate::pipeline::extract::extract_cuttle_global;
use crate::shader::wgsl_struct::WgslTypeInfos;
use crate::shader::{AddSnippet, Snippets, ToComponentShaderInfo, ToRenderData};
use bevy::app::{App, PostUpdate};
use bevy::prelude::{Component, Entity, ExtractSchedule};
use bevy::reflect::Typed;
use bevy::render::sync_world::RenderEntity;
use bevy::render::RenderApp;
use convert_case::{Case, Casing};
use std::any::{type_name, TypeId};
use std::marker::PhantomData;

pub struct CuttleConfigBuilder<'a, Config> {
    pub(crate) config: Entity,
    pub(crate) app: &'a mut App,
    marker: PhantomData<Config>,
}

impl<Config: CuttleConfig> CuttleConfigBuilder<'_, Config> {
    fn group_comp<C: Component>(&mut self) -> &mut C {
        self.app
            .world_mut()
            .get_mut::<C>(self.config)
            .unwrap()
            .into_inner()
    }

    pub fn global_render_data<C: Cuttle + Default>(&mut self) -> &mut Self {
        assert!(
            C::HAS_RENDER_DATA,
            "global_render_data without any render data has no point"
        );

        let mut config = self.app.world_mut().entity_mut(self.config);
        config.insert(C::default());

        let buffer_entity = config.get::<RenderEntity>().unwrap().id();
        let binding = GlobalBuffer::init(self.app, buffer_entity, C::to_render_data);

        let wgsl = C::wgsl_type(self.app.world().resource::<WgslTypeInfos>());
        self.snippet(format!(
            "@group(3) @binding({}) var<storage, read> {}: {};\n\n{}",
            binding,
            wgsl.name.to_case(Case::Snake),
            wgsl.name,
            wgsl.definition
        ));

        self.app
            .sub_app_mut(RenderApp)
            .add_systems(ExtractSchedule, extract_cuttle_global::<C, C::RenderData>);

        self
    }

    pub fn calculation(
        &mut self,
        name: impl Into<String>,
        wgsl_type: impl Into<String>,
    ) -> &mut Self {
        self.group_comp::<Calculations>().push(Calculation {
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
    /// # use cuttle::groups::{CuttleConfig, builder::CuttleGroupBuilderAppExt};
    /// # let mut app = App::new();
    /// # #[derive(Component, Default)]
    /// # struct MyGroup;
    /// # impl CuttleConfig for MyGroup {
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
        self.group_comp::<Snippets>()
            .push(AddSnippet::Inline(snippet.into()));
        self
    }

    /// Takes a file path to a wgsl file to be added to the shader
    /// generated for this Group.
    /// Supports hot reloading.
    /// ```
    /// # use bevy::core_pipeline::core_2d::Transparent2d;
    /// # use bevy::prelude::*;
    /// # use cuttle::groups::{CuttleConfig, builder::CuttleGroupBuilderAppExt};
    /// # let mut app = App::new();
    /// # #[derive(Component, Default)]
    /// # struct MyGroup;
    /// # impl CuttleConfig for MyGroup {
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
        self.group_comp::<Snippets>()
            .push(AddSnippet::File(path.into()));
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
    pub fn component<C: Cuttle>(&mut self) -> &mut Self {
        let to_render_data = if C::HAS_RENDER_DATA {
            let binding = init_render_data(self.app, C::to_render_data);
            Some(ToRenderData {
                binding,
                to_wgsl: C::wgsl_type,
            })
        } else {
            None
        };
        self.register_component_manual::<C>(C::sort(), to_render_data, C::EXTENSION_INDEX_OVERRIDE)
    }

    /// Registers a marker component to work with this group.
    ///
    /// ```
    /// # use bevy::core_pipeline::core_2d::Transparent2d;
    /// # use bevy::prelude::{App, Component, Reflect};
    /// # use bevy::render::render_resource::ShaderType;
    /// # use cuttle::groups::{CuttleConfig, builder::CuttleGroupBuilderAppExt};
    /// # use cuttle::prelude::{Sdf, SdfOrder};
    /// # let mut app = App::new();
    /// # #[derive(Default, Component)]
    /// # struct MyGroup;
    /// # impl CuttleConfig for MyGroup { type Phase = Transparent2d; }
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
        self.register_component_manual::<C>(sort, None, None)
    }

    pub fn register_component_manual<C: Component + Typed>(
        &mut self,
        sort: impl Into<u32>,
        to_render_data: Option<ToRenderData>,
        extension_override: Option<u8>,
    ) -> &mut Self {
        let Some(function_name) = C::type_ident().map(|i| i.to_case(Case::Snake)) else {
            panic!(
                "Registering Component '{}' is not a named type",
                type_name::<C>()
            );
        };

        let order = ComponentOrder {
            sort: sort.into(),
            id: TypeId::of::<C>(),
            extension_override,
        };
        let to_shader_info = ToComponentShaderInfo {
            function_name,
            to_render_data,
        };
        self.group_comp::<ComponentInfos>().push(ComponentInfo {
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

pub trait CuttleGroupBuilderAppExt {
    fn cuttle_group<Config: CuttleConfig>(&mut self) -> CuttleConfigBuilder<Config>;
}

impl CuttleGroupBuilderAppExt for App {
    fn cuttle_group<Config: CuttleConfig>(&mut self) -> CuttleConfigBuilder<Config> {
        let config = initialize_config::<Config>(self);
        CuttleConfigBuilder {
            config,
            app: self,
            marker: PhantomData,
        }
    }
}
