use crate::bounding::make_compute_aabb_sytem;
use crate::bounding::BoundingSet;
use crate::calculations::Calculation;
use crate::calculations::Calculations;
use crate::components::set_flag_bit;
use crate::components::SdfCompInfos;
use crate::pipeline::extract::extract_sdf_comp;
use crate::shader::snippets::AddSnippet;
use crate::shader::snippets::AddSnippets;
use crate::utils::GetOrInitResourceWorldExt;
use bevy::prelude::*;
use bevy::reflect::Typed;
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::ShaderSize;
use bevy::render::RenderApp;
use std::any::type_name;
use std::fmt::Debug;
use std::marker::PhantomData;

pub trait SdfAppExt {
    fn sdf<C: Component>(&mut self) -> SdfBuilder<C, C>;
    fn add_sdf_shader(&mut self, shader: impl Into<String>);
    fn add_sdf_shader_file(&mut self, shader: impl Into<String>);
    fn add_sdf_calculation(&mut self, name: impl Into<String>, wgsl_type: impl Into<String>);
}

impl SdfAppExt for App {
    fn sdf<C: Component>(&mut self) -> SdfBuilder<C, C> {
        SdfBuilder {
            app: self,
            marker: PhantomData::<(C, C)>,
        }
    }

    fn add_sdf_shader(&mut self, shader: impl Into<String>) {
        self.world_mut()
            .resource_or_init::<AddSnippets>()
            .push(AddSnippet::Inline(shader.into()));
    }

    fn add_sdf_shader_file(&mut self, path: impl Into<String>) {
        self.world_mut()
            .resource_or_init::<AddSnippets>()
            .push(AddSnippet::File(path.into()));
    }

    fn add_sdf_calculation(&mut self, name: impl Into<String>, wgsl_type: impl Into<String>) {
        self.world_mut()
            .resource_or_init::<Calculations>()
            .push(Calculation {
                name: name.into(),
                wgsl_type: wgsl_type.into(),
            })
    }
}

pub trait SdfRenderData: ShaderSize + Typed + WriteInto + Default + Debug {}
impl<T: ShaderSize + Typed + WriteInto + Default + Debug> SdfRenderData for T {}

pub struct SdfBuilder<'a, C: Component, G> {
    app: &'a mut App,
    marker: PhantomData<(C, G)>,
}

impl<'app, C: Component, G> SdfBuilder<'app, C, G> {
    pub fn render_data<NewG: SdfRenderData>(self) -> SdfBuilder<'app, C, NewG> {
        SdfBuilder {
            app: self.app,
            marker: PhantomData,
        }
    }

    pub fn affect_bounds(self, set: BoundingSet, func: fn(&C) -> f32) -> Self {
        self.app.add_systems(
            PostUpdate,
            make_compute_aabb_sytem(func, set)
                .ambiguous_with_all()
                .in_set(set),
        );
        self
    }
}

impl<'app, C: Component + IntoRenderData<G>, G: SdfRenderData> SdfBuilder<'app, C, G> {
    pub fn register(self, order: u32) -> &'app mut App {
        let world = self.app.world_mut();

        world.resource_or_init::<SdfCompInfos>().add::<C, G>(order);

        world.add_observer(set_flag_bit::<C, OnAdd, true>);
        world.add_observer(set_flag_bit::<C, OnRemove, false>);

        self.app
            .sub_app_mut(RenderApp)
            .add_systems(ExtractSchedule, extract_sdf_comp::<C, G>);

        trace!(
            "Registered comp {} with render data {}: order={}",
            type_name::<C>(),
            type_name::<G>(),
            order
        );

        self.app
    }
}

pub trait IntoRenderData<G>: Sync + Send + 'static {
    fn into_render_data(input: &Self) -> G;
}

impl<C: Clone + SdfRenderData> IntoRenderData<C> for C {
    #[inline(always)]
    fn into_render_data(input: &C) -> C {
        input.clone()
    }
}
