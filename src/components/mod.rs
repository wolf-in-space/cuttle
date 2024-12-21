use crate::extensions::Extension;
use crate::groups::CuttleGroup;
use crate::CuttleFlags;
use arena::IndexArena;
use bevy::prelude::*;
use buffer::BufferPlugin;

pub mod arena;
pub mod buffer;
pub mod initialization;

pub struct CompPlugin;
impl Plugin for CompPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BufferPlugin);
    }
}

pub(crate) const fn build_set_flag_bit<C: Component, G: CuttleGroup, T, const SET: bool>(
    pos: u8,
) -> impl FnMut(
    Trigger<T, C>,
    ResMut<IndexArena<C>>,
    Query<&mut CuttleFlags, Or<(With<G>, With<Extension<G>>)>>,
) {
    move |trigger, mut arena, mut flags| {
        if let Ok(mut flag) = flags.get_mut(trigger.entity()) {
            if SET {
                flag.flag.set(pos);
                flag.indices.insert(pos, arena.get());
            } else {
                flag.flag.unset(pos);
                let id = flag.indices.remove(&pos).unwrap();
                arena.release(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::components::arena::IndexArena;
    use crate::components::build_set_flag_bit;
    use crate::prelude::{Extension, Sdf, UiSdf};
    use crate::CuttleFlags;
    use bevy::prelude::{Component, OnAdd, World};

    #[derive(Component)]
    struct Comp<const C: u8>;

    #[test]
    fn set_flags_multiple_groups() {
        let mut world = World::new();

        world.init_resource::<IndexArena<Comp<0>>>();
        world.init_resource::<IndexArena<Comp<1>>>();
        world.init_resource::<IndexArena<Comp<2>>>();
        world.init_resource::<IndexArena<Comp<3>>>();

        world.add_observer(build_set_flag_bit::<Comp<0>, Sdf, OnAdd, true>(0));
        world.add_observer(build_set_flag_bit::<Comp<1>, Sdf, OnAdd, true>(1));
        world.add_observer(build_set_flag_bit::<Comp<2>, Sdf, OnAdd, true>(2));
        world.add_observer(build_set_flag_bit::<Comp<3>, Sdf, OnAdd, true>(3));
        world.add_observer(build_set_flag_bit::<Comp<1>, UiSdf, OnAdd, true>(1));
        world.add_observer(build_set_flag_bit::<Comp<3>, UiSdf, OnAdd, true>(3));

        let ent1 = world.spawn((CuttleFlags::default(), Comp::<0>)).id();
        let ent2 = world.spawn((Sdf, CuttleFlags::default(), Comp::<0>)).id();
        let ent3 = world.spawn((UiSdf, CuttleFlags::default(), Comp::<0>)).id();
        let ent4 = world
            .spawn((
                Sdf,
                CuttleFlags::default(),
                Comp::<0>,
                Comp::<1>,
                Comp::<2>,
                Comp::<3>,
            ))
            .id();
        let ent5 = world
            .spawn((
                UiSdf,
                CuttleFlags::default(),
                Comp::<0>,
                Comp::<1>,
                Comp::<2>,
                Comp::<3>,
            ))
            .id();

        assert_eq!(world.get::<CuttleFlags>(ent1).unwrap().flag.0, 0b0);
        assert_eq!(world.get::<CuttleFlags>(ent2).unwrap().flag.0, 0b1);
        assert_eq!(world.get::<CuttleFlags>(ent3).unwrap().flag.0, 0b0);
        assert_eq!(world.get::<CuttleFlags>(ent4).unwrap().flag.0, 0b1111);
        assert_eq!(world.get::<CuttleFlags>(ent5).unwrap().flag.0, 0b1010);

        let ent1 = world.spawn((CuttleFlags::default(), Comp::<0>)).id();
        let ent2 = world
            .spawn((
                Extension::<Sdf>::new(ent2),
                CuttleFlags::default(),
                Comp::<0>,
            ))
            .id();
        let ent3 = world
            .spawn((
                Extension::<UiSdf>::new(ent3),
                CuttleFlags::default(),
                Comp::<0>,
            ))
            .id();
        let ent4 = world
            .spawn((
                Extension::<Sdf>::new(ent4),
                CuttleFlags::default(),
                Comp::<0>,
                Comp::<1>,
                Comp::<2>,
                Comp::<3>,
            ))
            .id();
        let ent5 = world
            .spawn((
                Extension::<UiSdf>::new(ent5),
                CuttleFlags::default(),
                Comp::<0>,
                Comp::<1>,
                Comp::<2>,
                Comp::<3>,
            ))
            .id();

        assert_eq!(world.get::<CuttleFlags>(ent1).unwrap().flag.0, 0b0);
        assert_eq!(world.get::<CuttleFlags>(ent2).unwrap().flag.0, 0b1);
        assert_eq!(world.get::<CuttleFlags>(ent3).unwrap().flag.0, 0b0);
        assert_eq!(world.get::<CuttleFlags>(ent4).unwrap().flag.0, 0b1111);
        assert_eq!(world.get::<CuttleFlags>(ent5).unwrap().flag.0, 0b1010);

        assert_eq!(world.resource::<IndexArena<Comp<0>>>().max, 4);
        assert_eq!(world.resource::<IndexArena<Comp<1>>>().max, 4);
        assert_eq!(world.resource::<IndexArena<Comp<2>>>().max, 2);
        assert_eq!(world.resource::<IndexArena<Comp<3>>>().max, 4);
    }
}
