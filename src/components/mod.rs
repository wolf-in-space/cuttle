use self::{
    buffer::SdfBuffer,
    extract::{extract_sdf_comp, CompOffsets},
};
use crate::{
    flag::{BitPosition, Comp, Flag},
    shader::{CompShaderInfo, CompShaderInfos},
    utils::GetOrInitResourceWorldExt,
    ComdfExtractSet, ComdfPostUpdateSet,
};
use bevy::{prelude::*, render::RenderApp};

pub mod buffer;
pub mod colors;
pub mod extract;

pub fn plugin(app: &mut App) {
    app.add_plugins(extract::plugin)
        .init_resource::<CompOffsets>();
}

pub trait RegisterSdfRenderCompAppExt {
    fn register_sdf_render_comp<C: RenderSdfComponent>(&mut self) -> &mut Self;
}

impl RegisterSdfRenderCompAppExt for App {
    fn register_sdf_render_comp<C: RenderSdfComponent>(&mut self) -> &mut Self {
        let world = self.world_mut();
        let mut infos = world.resource_or_init::<CompShaderInfos>();
        let bit_index = infos.register(C::shader_info());
        world.insert_resource(BitPosition::<C>::new(bit_index));

        self.add_systems(
            PostUpdate,
            C::set_flag_bit.in_set(ComdfPostUpdateSet::BuildFlag),
        );
        self.sub_app_mut(RenderApp).add_systems(
            ExtractSchedule,
            extract_sdf_comp::<C>.in_set(ComdfExtractSet::Extract),
        );

        self
    }
}

pub trait RenderSdfComponent: Sized + Component + Clone {
    fn set_flag_bit(
        mut query: Query<&mut Flag<Comp>, With<Self>>,
        comp_bit: Res<BitPosition<Self>>,
    ) {
        query.iter_mut().for_each(|mut flag| {
            flag.set(comp_bit.position);
        });
    }

    fn shader_info() -> CompShaderInfo;

    fn push_to_buffer(&self, render: &mut SdfBuffer);
}
