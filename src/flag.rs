use crate::{operations::Operations, ComdfPostUpdateSet};
use bevy::{prelude::*, utils::HashSet};
use fixedbitset::FixedBitSet;
use itertools::Itertools;
use std::{array::from_fn, fmt::Debug, marker::PhantomData};

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        update_sdf_flags.in_set(ComdfPostUpdateSet::UpdateFlags),
    )
    .init_resource::<FlagsRegistry>()
    .add_event::<NewSdfFlags>();
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Hash)]
pub struct SdfFlags {
    pub flag: CompFlag,
    pub operations: Vec<(OpFlag, CompFlag)>,
}

impl SdfFlags {
    pub fn iter_comps(&self) -> impl Iterator<Item = &CompFlag> {
        [&self.flag]
            .into_iter()
            .chain(self.operations.iter().map(|(_, f)| f))
    }

    pub fn iter_unique_comps(&self) -> impl Iterator<Item = &CompFlag> {
        self.iter_comps().sorted().dedup()
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct FlagsRegistry(HashSet<SdfFlags>);

fn update_sdf_flags(
    mut hosts: Query<
        (&Operations, &CompFlag, &mut SdfFlags),
        Or<(Changed<Operations>, Changed<CompFlag>)>,
    >,
    targets: Query<&CompFlag>,
    mut registry: ResMut<FlagsRegistry>,
    mut new_sdf: EventWriter<NewSdfFlags>,
) {
    for (operations, flag, mut flags) in hosts.iter_mut() {
        flags.operations.clear();
        flags.flag = flag.clone();
        for (target, info) in operations.iter().sorted_by_key(|(_, i)| i.order) {
            let Ok(flag) = targets.get(*target) else {
                error!("Operations Component held an Entry for Entity {target:?} which no longer exists / has the CompFlag Component");
                continue;
            };
            flags
                .operations
                .push((info.operation.clone(), flag.clone()));
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
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Component, Deref, DerefMut)]
pub struct CompFlag(pub FixedBitSet);

impl Default for CompFlag {
    fn default() -> Self {
        Self(FixedBitSet::with_capacity(64))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Component, Deref, DerefMut)]
pub struct OpFlag(pub FixedBitSet);

impl Default for OpFlag {
    fn default() -> Self {
        Self(FixedBitSet::with_capacity(64))
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
            count: 0,
        }
    }
}
