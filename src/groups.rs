use crate::bounding::{make_compute_aabb_system, InitBoundingFn};
use crate::calculations::Calculation;
use crate::components::buffer::BufferEntity;
use crate::components::initialization::{
    init_component, init_components_for_group, init_zst_component, CuttleComponent,
    CuttleRenderDataFrom, CuttleZstComponent, InitComponentInfo, RegisterCuttleComponent,
};
use crate::indices::{
    build_set_flag_index, on_add_group_marker_initialize_indices_group_id, CuttleIndices,
};
use crate::pipeline::SortedCuttlePhaseItem;
use crate::shader::{load_shader_to_pipeline, AddSnippet, ShaderSettings};
use bevy::prelude::*;
use bevy::render::sync_world::RenderEntity;
use bevy::render::RenderApp;
use bevy::utils::TypeIdMap;
use std::{any::TypeId, marker::PhantomData};

pub trait CuttleGroup: Component + Default {
    type Phase: SortedCuttlePhaseItem;
}

pub type InitGroupFn = fn(&mut App);
#[derive(Resource, Deref, DerefMut, Default)]
pub struct InitGroupFns(Vec<InitGroupFn>);

pub type InitObserversFn = fn(&mut App, positions: Vec<Option<u8>>);
pub type InitExtractFn = fn(&mut App, positions: Vec<Option<u8>>);

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

    pub fn register_component<C: Component>(&mut self, group_id: usize, pos: u8) {
        let id = TypeId::of::<C>();
        self.component_positions[group_id].insert(id, pos);
        self.component_observer_inits.insert(id, |app, positions| {
            app.add_observer(build_set_flag_index::<true, OnAdd, C>(positions.clone()));
            app.add_observer(build_set_flag_index::<false, OnRemove, C>(positions));
        });
    }
}

#[derive(Resource)]
pub(crate) struct GroupData<G> {
    pub init_comp_fns: Vec<InitComponentInfo>,
    pub calculations: Vec<Calculation>,
    pub snippets: Vec<AddSnippet>,
    pub marker: PhantomData<G>,
}

impl<G> Default for GroupData<G> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
            init_comp_fns: default(),
            calculations: vec![Calculation {
                name: "vertex".to_string(),
                wgsl_type: "VertexOut".to_string(),
            }],
            snippets: default(),
        }
    }
}

pub struct CuttleGroupBuilder<'a, G> {
    pub(crate) group: &'a mut GroupData<G>,
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

    /// Registers a zst (zero sized type / struct with no data) component
    /// to affect any entity of this Group that it is added to
    ///
    /// ```
    /// # use bevy::prelude::{Component, Reflect};
    /// # use bevy::render::render_resource::ShaderType;
    /// # use cuttle::components::initialization::{CuttleComponent, CuttleZstComponent};
    /// # use cuttle::prelude::DISTANCE_POS;
    ///
    /// #[derive(Component, Reflect, ShaderType, Clone, Debug)]
    /// struct MyZstComponent;
    ///
    /// impl CuttleZstComponent for MyZstComponent {
    ///     const SORT: u32 = DISTANCE_POS + 500;
    /// }
    /// ```
    ///
    /// Example wgsl code for MyZstComponent:
    /// ```wgsl
    /// fn my_zst_component() {
    ///     distance *= 2.0;
    /// }
    /// ```
    pub fn zst_component<C: CuttleZstComponent>(&mut self) -> &mut Self {
        self.group.init_comp_fns.push(InitComponentInfo {
            sort: C::SORT,
            init_bounding: None,
            init_fn: init_zst_component::<C, G>,
        });
        self
    }

    /// Registers a component to affect any entity of this Group that it is added to
    ///
    /// ```
    /// # use bevy::prelude::{Component, Reflect};
    /// # use bevy::render::render_resource::ShaderType;
    /// # use cuttle::components::initialization::CuttleComponent;
    /// # use cuttle::prelude::DISTANCE_POS;
    ///
    /// #[derive(Component, Reflect, ShaderType, Clone, Debug)]
    /// struct MyComponent {
    ///     value: f32,
    /// }
    ///
    /// impl CuttleComponent for MyComponent {
    ///     type RenderData = Self;
    ///     const SORT: u32 = DISTANCE_POS + 500;
    /// }
    /// ```
    ///
    /// Example wgsl code for MyComponent:
    /// ```wgsl
    /// fn my_component(input: MyComponent) {
    ///     distance += input.value;
    /// }
    /// ```
    pub fn component<C: CuttleComponent>(&'a mut self) -> &'a mut Self {
        self.component_with(C::registration_data())
    }

    /// Specify all the data for the component manually. Useful to
    /// evade the orphan rule.
    /// see [`component`](CuttleGroupBuilder::component) for more info.
    pub fn component_with<C: Component, R: CuttleRenderDataFrom<C>>(
        &mut self,
        data: RegisterCuttleComponent<C, R>,
    ) -> &mut Self {
        let init_bounding = data.affect_bounds_fn.map(|func| {
            let result: InitBoundingFn = Box::new(move |app: &mut App| {
                app.add_systems(
                    PostUpdate,
                    make_compute_aabb_system(func, data.affect_bounds)
                        .ambiguous_with_all()
                        .in_set(data.affect_bounds),
                );
            });
            result
        });
        self.group.init_comp_fns.push(InitComponentInfo {
            sort: data.sort,
            init_bounding,
            init_fn: init_component::<C, R, G>,
        });
        self
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
        CuttleGroupBuilder {
            group: self.world_mut().resource_mut::<GroupData<G>>().into_inner(),
        }
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
            .push(init_group::<G>);
    }
}

fn init_group<G: CuttleGroup>(app: &mut App) {
    let world = app.world_mut();
    let group_id = world.resource::<GroupIdStore<G>>().id;
    let GroupData {
        init_comp_fns,
        snippets,
        calculations,
        ..
    } = world.remove_resource::<GroupData<G>>().unwrap();
    let infos = init_components_for_group(app, init_comp_fns, group_id);
    let shader_settings = ShaderSettings {
        infos,
        calculations,
        snippets,
    };
    load_shader_to_pipeline(app, shader_settings, group_id);
}
