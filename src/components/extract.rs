use super::{buffer::SdfBuffer, RenderSdfComponent};
use crate::{
    flag::{BitPosition, Comp, Flag, FlagStorage},
    shader::CompShaderInfos,
    ComdfExtractSet::*,
    ComdfPostUpdateSet::*,
};
use bevy::{
    prelude::*,
    render::{Extract, RenderApp},
    utils::HashMap,
};
use itertools::{EitherOrBoth, Itertools};
use std::any::type_name;

pub fn plugin(app: &mut App) {
    app.add_event::<NewBinding>()
        .add_event::<IncreaseSdfBufferSize>()
        .init_resource::<SdfBindings>()
        .init_resource::<CompOffsets>()
        .add_systems(
            PostUpdate,
            (
                assign_bindings.in_set(AssignBindings),
                assign_indices.in_set(AssignIndices),
                CompOffsets::add_new_offsets.after(AssignBindings),
            ),
        );

    app.sub_app_mut(RenderApp)
        .init_resource::<SdfBuffers>()
        .init_resource::<SdfBindings>()
        .add_systems(
            ExtractSchedule,
            (
                prepare_buffers_for_extract.in_set(PrepareExtract),
                extract_bindings.in_set(Extract),
            ),
        );
}

pub fn extract_sdf_comp<Comp: RenderSdfComponent>(
    offsets: Extract<Res<CompOffsets>>,
    flag_bit: Extract<Res<BitPosition<Comp>>>,
    comps: Extract<Query<(&SdfBinding, &SdfBufferIndex, &Comp)>>,
    mut buffers: ResMut<SdfBuffers>,
) {
    let offsets = &offsets[flag_bit.position as usize];
    for (binding, index, comp) in comps.into_iter() {
        let buffer = &mut buffers[binding.0];
        let Some(offset) = offsets.get(binding.0) else {
            error!(
                "Offset during extract not found for: type={}, binding={}, bit={}, offsets_len={}",
                type_name::<Comp>(),
                binding.0,
                index.0,
                offsets.len()
            );
            continue;
        };
        buffer.prep_for_push(index.0, *offset);
        comp.push_to_buffer(buffer);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SdfBuffers(Vec<SdfBuffer>);

fn prepare_buffers_for_extract(
    mut new_bindings: Extract<EventReader<NewBinding>>,
    mut resizes: Extract<EventReader<IncreaseSdfBufferSize>>,
    mut buffers: ResMut<SdfBuffers>,
) {
    for new_binding in new_bindings.read() {
        debug_assert_eq!(buffers.len(), new_binding.binding);
        buffers.push(SdfBuffer::new(new_binding.stride))
    }

    for event in resizes.read() {
        let buffer = &mut buffers[event.binding];
        let new_size = event.new_size * buffer.stride;
        // println!(
        //     "RESIZE from {} to {}, for binding={}",
        //     buffer.buffer.len(),
        //     new_size,
        //     event.binding
        // );
        buffer.buffer.values_mut().resize(new_size, 0);
    }
}

#[derive(Event, Debug)]
pub struct NewBinding {
    binding: usize,
    stride: usize,
    offsets: Vec<(u8, usize)>,
}

pub type CompOffsets = FlagStorage<Vec<usize>, { Flag::<Comp>::SIZE }>;

impl CompOffsets {
    pub fn add_new_offsets(mut this: ResMut<CompOffsets>, mut event: EventReader<NewBinding>) {
        for event in event.read() {
            for (bit, offset) in event.offsets.iter() {
                // println!("bit = {}, off = {}, bind = {}", bit, offset, event.binding);
                let offsets = &mut this[*bit as usize];
                offsets.resize(event.binding + 1, usize::MAX);
                offsets[event.binding] = *offset;
            }
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct SdfBinding(pub usize);

#[derive(Resource, Clone, Default, Deref, DerefMut)]
pub struct SdfBindings(pub HashMap<Flag<Comp>, usize>);

fn extract_bindings(
    main_bindings: Extract<Res<SdfBindings>>,
    mut render_bindings: ResMut<SdfBindings>,
) {
    if main_bindings.is_changed() {
        *render_bindings = main_bindings.clone()
    }
}

pub(crate) fn assign_bindings(
    mut bindings: ResMut<SdfBindings>,
    mut query: Query<(&Flag<Comp>, &mut SdfBinding), Changed<Flag<Comp>>>,
    mut events: EventWriter<NewBinding>,
    infos: Res<CompShaderInfos>,
) {
    for (flag, mut binding) in query.iter_mut() {
        match bindings.get(flag) {
            Some(new_binding) => *binding = SdfBinding(*new_binding),
            None => {
                let new_binding = bindings.len();
                bindings.insert(*flag, new_binding);
                *binding = SdfBinding(new_binding);
                let (stride, offsets) = SdfBuffer::stride_and_offsets_for_flag(flag, &infos);
                events.send(NewBinding {
                    binding: new_binding,
                    offsets,
                    stride,
                });
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Ord)]
pub struct SdfBufferIndex(pub usize);

#[derive(Event, Debug)]
struct IncreaseSdfBufferSize {
    binding: usize,
    new_size: usize,
}

fn assign_indices(
    mut max: Local<Vec<usize>>,
    mut size_events: EventWriter<IncreaseSdfBufferSize>,
    bindings: Res<SdfBindings>,
    mut query: Query<(&mut SdfBufferIndex, &SdfBinding)>,
) {
    let total_bindings = bindings.len();
    let new_max = query.iter_mut().fold(
        vec![0; total_bindings],
        |mut binding_indices, (mut render_index, SdfBinding(bind))| {
            let index = &mut binding_indices[*bind];
            render_index.0 = *index;
            *index += 1;
            binding_indices
        },
    );

    // dbg!(&new_max);

    for (binding, val) in max.iter().zip_longest(new_max.iter()).enumerate() {
        match val {
            EitherOrBoth::Both(prev, new) if new > prev => {
                size_events.send(IncreaseSdfBufferSize {
                    binding,
                    new_size: *new,
                });
            }
            EitherOrBoth::Right(size) => {
                // println!("INCREASE");
                size_events.send(IncreaseSdfBufferSize {
                    binding,
                    new_size: *size,
                });
            }
            _ => (),
        }
    }

    *max = new_max;
}
