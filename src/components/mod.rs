use self::{
    buffer::SdfBuffer,
    extract::{extract_sdf_comp, CompOffsets},
};
use crate::{
    flag::{BitPosition, CompFlag},
    shader::{CompShaderInfo, CompShaderInfos},
    utils::GetOrInitResourceWorldExt,
    ComdfExtractSet,
};
use bevy::{prelude::*, render::RenderApp};
use std::any::type_name;

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
        world.observe(set_flag_bit::<C, OnAdd, true>);
        world.observe(set_flag_bit::<C, OnRemove, false>);

        self.sub_app_mut(RenderApp).add_systems(
            ExtractSchedule,
            extract_sdf_comp::<C>.in_set(ComdfExtractSet::Extract),
        );

        trace!(
            "Registered comp {}: index={}, {:#?}",
            type_name::<C>(),
            bit_index,
            C::shader_info()
        );

        self
    }
}

pub fn set_flag_bit<COMP: Component, T, const SET: bool>(
    trigger: Trigger<T, COMP>,
    bit: Res<BitPosition<COMP>>,
    mut flags: Query<&mut CompFlag>,
) {
    if let Ok(mut flag) = flags.get_mut(trigger.entity()) {
        flag.set(bit.position as usize, SET)
    }
}

pub trait RenderSdfComponent: Sized + Component + Clone {
    // fn set_flag_bit(mut query: Query<&mut CompFlag, With<Self>>, comp_bit: Res<BitPosition<Self>>) {
    //     query.iter_mut().for_each(|mut flag| {
    //         flag.set(comp_bit.position as usize, true);
    //     });
    // }

    fn shader_info() -> CompShaderInfo;

    fn push_to_buffer(&self, render: &mut SdfBuffer);
}
