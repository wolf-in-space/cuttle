use crate::{operations::Operations, shader::lines::Lines, ComdfPostUpdateSet};
use bevy::{prelude::*, utils::HashSet};
use itertools::Itertools;
use std::{array::from_fn, marker::PhantomData};

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        update_sdf_flags.in_set(ComdfPostUpdateSet::UpdateFlags),
    )
    .init_resource::<FlagsRegistry>()
    .add_event::<NewSdfFlags>();
}

#[derive(
    Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Hash, Deref, DerefMut,
)]
pub struct SdfFlags(Vec<(Flag<Op>, Flag<Comp>)>);

impl SdfFlags {
    pub fn map_to_lines<FN: Fn(&Flag<Comp>) -> Lines>(&self, func: FN) -> Lines {
        self.iter().map(|(_, f)| f).map(func).collect()
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct FlagsRegistry(HashSet<SdfFlags>);

fn update_sdf_flags(
    mut hosts: Query<
        (&Operations, &Flag<Comp>, &mut SdfFlags),
        Or<(Changed<Operations>, Changed<Flag<Comp>>)>,
    >,
    targets: Query<&Flag<Comp>>,
    mut registry: ResMut<FlagsRegistry>,
    mut new_sdf: EventWriter<NewSdfFlags>,
) {
    for (operations, flag, mut flags) in hosts.iter_mut() {
        flags.clear();
        flags.push((Flag::default(), *flag));
        for (target, info) in operations.iter().sorted_by_key(|(_, i)| i.order) {
            let Ok(flag) = targets.get(*target) else {
                error!("Operations Component held an Entry for Entity {target:?} which no longer exists / has the CompFlag Component");
                continue;
            };
            flags.push((info.operation, *flag));
        }
        if !registry.contains(&flags.clone()) {
            registry.insert(flags.clone());
            new_sdf.send(NewSdfFlags(flags.clone()));
        }
    }
}

#[derive(Event, Deref, Debug)]
pub struct NewSdfFlags(pub SdfFlags);

#[derive(Resource)]
pub struct BitPosition<M> {
    pub position: u8,
    marker: PhantomData<M>,
}

impl<M> BitPosition<M> {
    pub fn new(position: u8) -> Self {
        Self {
            position,
            marker: PhantomData,
        }
    }

    pub fn as_flag<T>(&self) -> Flag<T> {
        Flag::<T>::new(self.position as u64)
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Comp;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Op;

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Component)]
pub struct Flag<M> {
    bits: u64,
    marker: PhantomData<M>,
}

impl<M> Flag<M> {
    pub fn new(bits: u64) -> Self {
        Self {
            bits,
            marker: PhantomData,
        }
    }

    pub fn as_str(&self) -> String {
        self.bits().to_string()
    }

    pub fn bits(&self) -> u64 {
        self.bits
    }

    pub const SIZE: usize = 65;

    pub fn set(&mut self, position: u8) {
        self.bits |= 1 << position;
    }

    pub fn iter_indices_of_set_bits(&self) -> SetFlagBitPositionsIterator {
        SetFlagBitPositionsIterator(self.bits)
    }
}

pub struct SetFlagBitPositionsIterator(u64);
impl Iterator for SetFlagBitPositionsIterator {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            let next = self.0.trailing_zeros();
            self.0 &= self.0 - 1;
            Some(next as u8)
        }
    }
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct FlagStorage<T, const C: usize> {
    #[deref]
    pub(crate) storage: [T; C],
    pub(crate) count: u8,
}

impl<T, const C: usize> FlagStorage<T, C> {
    pub fn register(&mut self, value: T) -> u8 {
        let i = self.count;
        self.count += 1;
        self.storage[i as usize] = value;
        i
    }
}

impl<T: Default, const C: usize> Default for FlagStorage<T, C> {
    fn default() -> Self {
        Self {
            storage: from_fn(|_| T::default()),
            count: 1,
        }
    }
}

#[cfg(test)]
mod test_iter_indices_of_set_bits {
    use super::Flag;
    use crate::flag::Comp;

    #[test]
    fn empty() {
        let mut iter = Flag::<Comp>::new(0).iter_indices_of_set_bits();
        assert_eq!(iter.next(), None)
    }

    #[test]
    fn three() {
        let mut iter = Flag::<Comp>::new(0b111).iter_indices_of_set_bits();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn lots() {
        let mut iter = Flag::<Comp>::new(0b1100101100).iter_indices_of_set_bits();
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), Some(8));
        assert_eq!(iter.next(), Some(9));
        assert_eq!(iter.next(), None);
    }
}
