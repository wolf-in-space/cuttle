use crate::bounding::{make_compute_aabb_system, Bounding};
use crate::calculations::Calculation;
use crate::components::arena::IndexArena;
use crate::components::buffer::BufferEntity;
use crate::components::initialization::{
    init_render_data, ComponentOrder, CuttleStructComponent, CuttleWrapperComponent,
};
use crate::indices::{
    build_set_flag_index, on_add_group_marker_initialize_indices_group_id, CuttleComponentIndex,
    CuttleIndices,
};
use crate::pipeline::SortedCuttlePhaseItem;
use crate::shader::wgsl_struct::WgslTypeInfos;
use crate::shader::{
    load_shader_to_pipeline, AddSnippet, ComponentShaderInfo, RenderDataShaderInfo, ShaderSettings,
    ToComponentShaderInfo, ToRenderDataShaderInfo,
};
use bevy::prelude::*;
use bevy::reflect::Typed;
use bevy::render::sync_world::RenderEntity;
use bevy::render::RenderApp;
use bevy::utils::TypeIdMap;
use convert_case::{Case, Casing};
use std::any::type_name;
use std::{any::TypeId, marker::PhantomData, mem};

pub trait CuttleGroup: Component + Default {
    type Phase: SortedCuttlePhaseItem;
}

pub type InitGroupFn = fn(&mut App);
#[derive(Resource, Deref, DerefMut, Default)]
pub struct InitGroupFns(Vec<InitGroupFn>);

pub type InitObserversFn = fn(&mut App, positions: Vec<Option<u8>>);

#[derive(Resource)]
pub struct GlobalGroupInfos {
    pub group_count: usize,
    pub component_bindings: TypeIdMap<u32>,
    pub component_observer_inits: TypeIdMap<InitObserversFn>,
    pub component_positions: Vec<TypeIdMap<u8>>,
    pub buffer_entity: RenderEntity,
}

impl GlobalGroupInfos {
    fn new(app: &mut App) -> Self {
        let id = app
            .sub_app_mut(RenderApp)
            .world_mut()
            .spawn(BufferEntity)
            .id();
        Self {
            group_count: 0,
            component_bindings: default(),
            component_positions: default(),
            component_observer_inits: default(),
            buffer_entity: RenderEntity::from(id),
        }
    }

    pub fn is_registered<C: Component>(&self) -> bool {
        self.component_observer_inits
            .contains_key(&TypeId::of::<C>())
    }

    pub fn register_component_positions(
        app: &mut App,
        group_id: usize,
        positions: Vec<(TypeId, u8)>,
    ) {
        let mut global = app.world_mut().resource_mut::<GlobalGroupInfos>();
        for (id, pos) in positions {
            global.component_positions[group_id].insert(id, pos);
        }
    }

    pub fn register_component<C: Component>(app: &mut App) {
        let mut global = app.world_mut().resource_mut::<GlobalGroupInfos>();
        let id = TypeId::of::<C>();
        if !global.is_registered::<C>() {
            global
                .component_observer_inits
                .insert(id, |app, positions| {
                    app.add_observer(build_set_flag_index::<true, OnAdd, C>(positions.clone()));
                    app.add_observer(build_set_flag_index::<false, OnRemove, C>(positions));
                });
        }
    }
}

pub struct ComponentInfo {
    order: ComponentOrder,
    to_shader_info: ToComponentShaderInfo,
}

#[derive(Resource)]
pub(crate) struct GroupData<G> {
    pub component_infos: Vec<ComponentInfo>,
    pub calculations: Vec<Calculation>,
    pub snippets: Vec<AddSnippet>,
    pub marker: PhantomData<G>,
}

impl<G: CuttleGroup> GroupData<G> {
    fn init(app: &mut App) {
        let world = app.world_mut();
        let group_id = world.resource::<GroupIdStore<G>>().id;
        let GroupData {
            component_infos,
            snippets,
            calculations,
            ..
        } = world.remove_resource::<GroupData<G>>().unwrap();

        let (component_order, to_shader_infos) =
            Self::sort_and_split_component_infos(component_infos);
        Self::init_component_positions(app, group_id, component_order);
        let infos = Self::process_to_shader_infos(app, to_shader_infos);

        let shader_settings = ShaderSettings {
            infos,
            calculations,
            snippets,
        };
        load_shader_to_pipeline(app, shader_settings, group_id);
    }

    fn sort_and_split_component_infos(
        mut component_infos: Vec<ComponentInfo>,
    ) -> (Vec<ComponentOrder>, Vec<ToComponentShaderInfo>) {
        component_infos.sort_by_key(|c| c.order.sort);
        component_infos
            .into_iter()
            .map(|i| (i.order, i.to_shader_info))
            .unzip()
    }

    fn init_component_positions(app: &mut App, group_id: usize, components: Vec<ComponentOrder>) {
        let positions = components
            .into_iter()
            .enumerate()
            .map(|(i, info)| (info.id, i as u8))
            .collect();
        GlobalGroupInfos::register_component_positions(app, group_id, positions);
    }

    fn process_to_shader_infos(
        app: &mut App,
        to_shader_info: Vec<ToComponentShaderInfo>,
    ) -> Vec<ComponentShaderInfo> {
        let wgsl_type_infos = app.world().resource::<WgslTypeInfos>();
        to_shader_info
            .into_iter()
            .map(
                |ToComponentShaderInfo {
                     function_name,
                     to_render_data,
                 }| {
                    let render_data =
                        to_render_data.map(|ToRenderDataShaderInfo { binding, to_wgsl }| {
                            RenderDataShaderInfo {
                                binding,
                                wgsl: to_wgsl(&wgsl_type_infos),
                            }
                        });
                    ComponentShaderInfo {
                        function_name,
                        render_data,
                    }
                },
            )
            .collect()
    }
}

impl<G> Default for GroupData<G> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
            component_infos: default(),
            calculations: vec![Calculation {
                name: "vertex".to_string(),
                wgsl_type: "VertexOut".to_string(),
            }],
            snippets: default(),
        }
    }
}

pub struct CuttleGroupBuilder<'a, G: CuttleGroup> {
    pub(crate) group: GroupData<G>,
    pub(crate) app: &'a mut App,
}

impl<'a, G: CuttleGroup> CuttleGroupBuilder<'a, G> {
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
    /// # use cuttle::groups::{CuttleGroup, CuttleGroupBuilderAppExt};
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
    /// # use cuttle::groups::{CuttleGroup, CuttleGroupBuilderAppExt};
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
        let binding = init_render_data(self.app, C::to_render_data);
        self.register_component_manual::<C>(
            sort,
            Some(ToRenderDataShaderInfo {
                binding,
                to_wgsl: C::wgsl_type,
            }),
        )
    }

    pub fn wrapper_component<C: CuttleWrapperComponent>(
        &mut self,
        sort: impl Into<u32>,
    ) -> &mut Self {
        let binding = init_render_data(self.app, C::to_render_data);
        self.register_component_manual::<C>(
            sort,
            Some(ToRenderDataShaderInfo {
                binding,
                to_wgsl: C::wgsl_type,
            }),
        )
    }

    /// Registers a marker component to work with this group.
    ///
    /// ```
    /// # use bevy::core_pipeline::core_2d::Transparent2d;
    /// # use bevy::prelude::{App, Component, Reflect};
    /// # use bevy::render::render_resource::ShaderType;
    /// # use cuttle::groups::{CuttleGroup, CuttleGroupBuilderAppExt};
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
        self.register_component_manual::<C>(sort, None)
    }

    pub fn register_component_manual<C: Component + Typed>(
        &mut self,
        sort: impl Into<u32>,
        to_render_data: Option<ToRenderDataShaderInfo>,
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

impl<'a, G: CuttleGroup> Drop for CuttleGroupBuilder<'a, G> {
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
        CuttleGroupBuilder { group, app: self }
    }
}

#[derive(Resource)]
pub(crate) struct GroupIdStore<G> {
    pub id: usize,
    phantom_data: PhantomData<G>,
}

impl<G> FromWorld for GroupIdStore<G> {
    fn from_world(world: &mut World) -> Self {
        let mut global = world.resource_mut::<GlobalGroupInfos>();
        let id = global.group_count;
        global.group_count += 1;
        global.component_positions.push(default());
        Self {
            id,
            phantom_data: PhantomData,
        }
    }
}

struct GroupPlugin<G>(PhantomData<G>);

impl<G> GroupPlugin<G> {
    fn new() -> Self {
        Self(PhantomData)
    }
}

impl<G: CuttleGroup> Plugin for GroupPlugin<G> {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<GlobalGroupInfos>() {
            app.init_resource::<InitGroupFns>();
            let infos = GlobalGroupInfos::new(app);
            app.insert_resource(infos);
        }

        app.init_resource::<GroupData<G>>();
        app.init_resource::<GroupIdStore<G>>();

        app.register_required_components::<G, CuttleIndices>();
        app.world_mut()
            .register_component_hooks::<G>()
            .on_add(on_add_group_marker_initialize_indices_group_id::<G>);

        // app.sub_app_mut(RenderApp)
        //    .add_plugins(render_group_plugin::<G>);

        app.world_mut()
            .resource_mut::<InitGroupFns>()
            .push(GroupData::<G>::init);
    }
}
