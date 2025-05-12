use crate::bounding::{Bounding, make_compute_aabb_system};
use crate::components::initialization::{
    Cuttle, init_component_render_data, init_global_render_data,
};
use crate::components::{Sort, register_cuttle};
use crate::configs::{CuttleConfig, initialize_config};
use crate::internal_prelude::*;
use crate::prelude::{ComputeBounding, CuttleRenderData};
use crate::shader::{AddSnippet, FunctionName, Snippets};
use bevy_ecs::component::Mutable;
use bevy_ecs::system::RunSystemOnce;
use std::marker::PhantomData;
use std::ops::Deref;
use variadics_please::all_tuples;

pub struct CuttleConfigBuilder<'a, Config> {
    pub(crate) config: Entity,
    pub(crate) app: &'a mut App,
    marker: PhantomData<Config>,
}

impl<Config: CuttleConfig> CuttleConfigBuilder<'_, Config> {
    fn get_comp_mut<C: Component<Mutability = Mutable>>(&mut self) -> &mut C {
        self.app
            .world_mut()
            .get_mut::<C>(self.config)
            .unwrap()
            .into_inner()
    }

    pub fn global_manual<C: Cuttle>(&mut self) -> CuttleBuilder<C> {
        CuttleBuilder::new(self.app, self.config, true)
    }

    pub fn global<C: Cuttle + Default>(&mut self) -> &mut Self {
        self.global_with(C::default())
    }

    pub fn global_with<C: Cuttle>(&mut self, value: C) -> &mut Self {
        C::build(self.global_manual::<C>());
        self.app.world_mut().entity_mut(self.config).insert(value);
        self
    }

    pub fn variable(&mut self, name: impl Into<String>, wgsl_type: impl Into<String>) -> &mut Self {
        self.snippet(format!(
            "var<private> {}: {};",
            name.into(),
            wgsl_type.into()
        ));
        self
    }

    /// Adds a snippet of wgsl code to the shader generated for this Group
    ///
    /// ```
    /// # use bevy_core_pipeline::core_2d::Transparent2d;
    /// # use bevy_app::prelude::*;
    /// # use bevy_ecs::prelude::Component;
    /// # use cuttle_core::prelude::*;
    /// # #[derive(Component, Default)]
    /// # struct MyGroup;
    /// # impl CuttleConfig for MyGroup {
    /// #     type Phase = Transparent2d;
    /// # }
    /// # let mut app = App::new();
    ///
    /// app.cuttle_config::<MyGroup>()
    /// .snippet(stringify!(
    ///     fn my_component(input: MyComponent) {
    ///         distance += input.value;
    ///     }
    /// ));
    ///
    /// ```
    pub fn snippet(&mut self, snippet: impl Into<String>) -> &mut Self {
        self.get_comp_mut::<Snippets>()
            .push(AddSnippet::Inline(snippet.into()));
        self
    }

    /// Takes a file path to a wgsl file to be added to the shader
    /// generated for this Group.
    /// Supports hot reloading.
    /// ```
    /// # use bevy_core_pipeline::core_2d::Transparent2d;
    /// # use bevy_app::prelude::*;
    /// # use bevy_ecs::prelude::Component;
    /// # use cuttle_core::prelude::*;
    /// # #[derive(Component, Default)]
    /// # struct MyGroup;
    /// # impl CuttleConfig for MyGroup {
    /// #     type Phase = Transparent2d;
    /// # }
    /// # let mut app = App::new();
    ///
    /// app.cuttle_config::<MyGroup>()
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
        self.get_comp_mut::<Snippets>()
            .push(AddSnippet::File(path.into()));
        self
    }

    /// Registers a component to affect any entity of this Group that it is added to
    ///
    /// ```
    /// # use bevy_ecs::prelude::{Component};
    /// # use bevy_reflect::Reflect;
    /// # use bevy_render::render_resource::ShaderType;
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
        C::build(self.component_manual::<C>());
        self
    }

    pub fn components<C: CuttleComponentBundle>(&mut self) -> &mut Self {
        C::register(self);
        self
    }

    pub fn component_manual<C: Component>(&mut self) -> CuttleBuilder<C> {
        CuttleBuilder::new(self.app, self.config, false)
    }

    pub fn affect_bounds<C: Component>(&mut self, set: Bounding, func: fn(&C) -> f32) -> &mut Self {
        self.app.add_systems(
            PostUpdate,
            make_compute_aabb_system(func, set).in_set(ComputeBounding),
        );
        self
    }
}

pub struct CuttleBuilder<'a, C: Component> {
    app: &'a mut App,
    component: Entity,
    config: Entity,
    global: bool,
    _marker: PhantomData<C>,
}

impl<'a, C: Component> CuttleBuilder<'a, C> {
    pub fn new(app: &'a mut App, config: Entity, global: bool) -> Self {
        let component = app
            .world_mut()
            .run_system_once_with(register_cuttle::<C>, config)
            .unwrap();
        Self {
            app,
            component,
            config,
            global,
            _marker: PhantomData,
        }
    }

    pub fn insert<B: Bundle>(&mut self, bundle: B) -> &mut Self {
        self.app
            .world_mut()
            .entity_mut(self.component)
            .insert(bundle);
        self
    }

    pub fn render_data(&mut self) -> &mut Self
    where
        C: Clone + CuttleRenderData,
    {
        self.render_data_manual::<C>(C::clone)
    }

    pub fn render_data_deref<R>(&mut self) -> &mut Self
    where
        C: Deref<Target = R>,
        R: CuttleRenderData + Clone,
    {
        self.render_data_manual::<R>(|c: &C| c.deref().clone())
    }

    pub fn render_data_from<R>(&mut self) -> &mut Self
    where
        for<'f> R: CuttleRenderData + From<&'f C>,
    {
        self.render_data_manual::<R>(|c: &C| c.into())
    }

    pub fn render_data_manual<R: CuttleRenderData>(
        &mut self,
        to_render_data: fn(&C) -> R,
    ) -> &mut Self {
        if self.global {
            init_global_render_data::<C, R>(self.app, self.config, to_render_data);
        } else {
            init_component_render_data::<C, R>(self.app, self.component, to_render_data);
        }
        self
    }

    pub fn sort(&mut self, sort: impl Into<u32>) -> &mut Self {
        self.insert(Sort(sort.into()))
    }

    pub fn name(&mut self, name: &'static str) -> &mut Self {
        self.insert(FunctionName::from_type_name(name))
    }

    pub fn snippet(&mut self, snippet: String) -> &mut Self {
        self.app
            .world_mut()
            .entity_mut(self.component)
            .get_mut::<Snippets>()
            .unwrap()
            .push(AddSnippet::Inline(snippet));
        self
    }

    pub fn affect_bounds(&mut self, set: Bounding, func: fn(&C) -> f32) -> &mut Self {
        self.app
            .add_systems(PostUpdate, make_compute_aabb_system(func, set));
        self
    }
}

pub trait CuttleGroupBuilderAppExt {
    fn cuttle_config<Config: CuttleConfig>(&mut self) -> CuttleConfigBuilder<Config>;
}

impl CuttleGroupBuilderAppExt for App {
    fn cuttle_config<Config: CuttleConfig>(&mut self) -> CuttleConfigBuilder<Config> {
        let config = initialize_config::<Config>(self);
        CuttleConfigBuilder {
            config,
            app: self,
            marker: PhantomData,
        }
    }
}

pub trait CuttleComponentBundle {
    fn register<Config: CuttleConfig>(builder: &mut CuttleConfigBuilder<Config>);
}

macro_rules! impl_cuttle_component_bundle {
    ($($C:ident),*) => {
        impl<$($C: Cuttle),*> CuttleComponentBundle for ($($C,)*) {
            fn register<Config: CuttleConfig>(builder: &mut CuttleConfigBuilder<Config>) {
                $(
                builder.component::<$C>();
                )*
            }
        }
    };
}

all_tuples!(impl_cuttle_component_bundle, 1, 15, C);
