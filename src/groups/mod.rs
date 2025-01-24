use crate::calculations::Calculation;
use crate::components::initialization::ComponentOrder;
use crate::components::{ComponentInfo, ComponentPosition};
use crate::indices::{on_add_group_marker_initialize_indices_group_id, CuttleIndices};
use crate::pipeline::SortedCuttlePhaseItem;
use crate::shader::wgsl_struct::WgslTypeInfos;
use crate::shader::{
    load_shader_to_pipeline, AddSnippet, ComponentShaderInfo, RenderDataShaderInfo, ShaderSettings,
    ToComponentShaderInfo, ToRenderDataShaderInfo,
};
use bevy::prelude::*;
use global::{GlobalGroupInfos, InitGroupFns};
use std::marker::PhantomData;

pub mod builder;
pub mod global;

pub trait CuttleGroup: Component + Default {
    type Phase: SortedCuttlePhaseItem;
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
            .map(|(i, info)| {
                (
                    info.id,
                    ComponentPosition::new(i as u8, info.extension_override),
                )
            })
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
                                wgsl: to_wgsl(wgsl_type_infos),
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
