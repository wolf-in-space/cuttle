use bevy::render::{
    render_resource::{
        BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingType,
        BufferBindingType, RawBufferVec, ShaderStages,
    },
    renderer::RenderDevice,
};
use itertools::Itertools;

use crate::flag::{CompFlag, OpFlag};

fn bind_group_layout_entry(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
        binding,
        visibility: ShaderStages::FRAGMENT,
    }
}

fn bind_group_layout(
    bindings: impl Iterator<Item = u32>,
    device: &RenderDevice,
) -> BindGroupLayout {
    let entrys = bindings.map(bind_group_layout_entry).collect_vec();
    device.create_bind_group_layout(None, &entrys)
}

fn bind_group_entrys<'a>(entrys: &[(u32, &'a RawBufferVec<u8>)]) -> Vec<BindGroupEntry<'a>> {
    entrys
        .iter()
        .map(|entry| BindGroupEntry {
            binding: entry.0,
            resource: entry
                .1
                .buffer()
                .expect("Bindgroup buffer was not written to")
                .as_entire_binding(),
        })
        .collect()
}

pub fn bind_group(
    entries: &[(u32, &RawBufferVec<u8>)],
    device: &RenderDevice,
) -> (BindGroupLayout, BindGroup) {
    let layout = bind_group_layout(entries.iter().map(|(b, _)| b).copied(), device);
    let entries = bind_group_entrys(entries);
    let bind_group = device.create_bind_group(None, &layout, &entries);
    (layout, bind_group)
}

pub fn flags_to_index_name((op, comp): &(OpFlag, CompFlag)) -> String {
    format!("i_{}_{}", op.0, comp.0)
}
