use crate::bounding::{make_compute_aabb_system, InitBoundingFn};
use crate::components::buffer::BufferEntity;
use crate::components::initialization::{
    init_component, init_components, InitComponentInfo, RegisterSdfComponent, SdfComponent,
    SdfRenderDataFrom,
};
use crate::pipeline::extract::extract_group_marker;
use crate::shader::{load_shader_to_pipeline, ShaderSettings};
use crate::{calculations::Calculation, shader::snippets::AddSnippet, SdfInternals};
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::sync_component::SyncComponentPlugin;
use bevy::render::sync_world::RenderEntity;
use bevy::render::RenderApp;
use bevy::utils::HashMap;
use std::{any::TypeId, marker::PhantomData};

pub trait SdfGroup: Component {
    // type Phase: RenderPhase;
}

#[derive(Resource)]
pub struct GlobalGroupInfos {
    pub group_count: u32,
    pub component_bindings: HashMap<TypeId, u32>,
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
            buffer_entity: RenderEntity::from(id),
        }
    }
}

#[derive(Resource)]
pub(crate) struct GroupData<G> {
    pub id: GroupId,
    pub init_comp_fns: Vec<InitComponentInfo>,
    pub calculations: Vec<Calculation>,
    pub snippets: Vec<AddSnippet>,
    pub marker: PhantomData<G>,
}

impl<G> FromWorld for GroupData<G> {
    fn from_world(world: &mut World) -> Self {
        let mut global = world.resource_mut::<GlobalGroupInfos>();
        let id = global.group_count;
        global.group_count += 1;
        Self {
            id: GroupId(id),
            marker: PhantomData,
            init_comp_fns: default(),
            calculations: default(),
            snippets: default(),
        }
    }
}

#[derive(Copy, Clone, Deref, DerefMut, Debug, Hash, Eq, PartialEq)]
pub struct GroupId(pub(crate) u32);

pub struct GroupBuilder<'a, G> {
    pub(crate) group: &'a mut GroupData<G>,
    pub(crate) global: &'a mut GlobalGroupInfos,
}

impl<'a, G: SdfGroup> GroupBuilder<'a, G> {
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

    pub fn snippet(&mut self, snippet: impl Into<String>) -> &mut Self {
        self.group.snippets.push(AddSnippet::Inline(snippet.into()));
        self
    }

    pub fn snippet_file(&mut self, path: impl Into<String>) -> &mut Self {
        self.group.snippets.push(AddSnippet::File(path.into()));
        self
    }

    pub fn component<C: SdfComponent>(&'a mut self) -> &mut Self {
        self.component_with(C::registration_data())
    }

    pub fn component_with<C: Component, R: SdfRenderDataFrom<C>>(
        &'a mut self,
        data: RegisterSdfComponent<C, R>,
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

pub trait SdfGroupBuilderAppExt {
    fn sdf_group<G: SdfGroup>(&mut self) -> GroupBuilder<G>;
}

type GroupState<'a, G> = SystemState<(ResMut<'a, GlobalGroupInfos>, ResMut<'a, GroupData<G>>)>;
impl SdfGroupBuilderAppExt for App {
    fn sdf_group<G: SdfGroup>(&mut self) -> GroupBuilder<G> {
        if !self.is_plugin_added::<GroupPlugin<G>>() {
            self.add_plugins(GroupPlugin::<G>::new());
        }
        let world = self.world_mut();
        let (global, group) = GroupState::<G>::new(world).get_mut(world);
        GroupBuilder {
            global: global.into_inner(),
            group: group.into_inner(),
        }
    }
}

struct GroupPlugin<G>(PhantomData<G>);

impl<G> GroupPlugin<G> {
    fn new() -> Self {
        Self(PhantomData)
    }
}

impl<G: SdfGroup> Plugin for GroupPlugin<G> {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<GlobalGroupInfos>() {
            let infos = GlobalGroupInfos::new(app);
            app.insert_resource(infos);
        }
        app.init_resource::<GroupData<G>>();
    }

    fn finish(&self, app: &mut App) {
        app.register_required_components::<G, SdfInternals>();
        app.add_plugins(SyncComponentPlugin::<G>::default());
        app.sub_app_mut(RenderApp)
            .add_systems(ExtractSchedule, extract_group_marker::<G>);
        let world = app.world_mut();
        let GroupData {
            init_comp_fns,
            snippets,
            calculations,
            id,
            ..
        } = world.remove_resource::<GroupData<G>>().unwrap();
        let infos = init_components(app, init_comp_fns);
        let shader_settings = ShaderSettings {
            infos,
            calculations,
            snippets,
        };
        load_shader_to_pipeline(app, shader_settings, id);
    }
}
