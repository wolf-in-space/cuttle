use crate::components::initialization::ComponentOrder;
use crate::configs::ConfigId;
use crate::internal_prelude::*;
use crate::shader::ToComponentShaderInfo;
use bevy_app::{App, Plugin};
use buffer::BufferPlugin;

pub mod arena;
pub mod buffer;
pub mod initialization;

pub struct CompPlugin;
impl Plugin for CompPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ConfigComponents>()
            .register_type::<Sort>()
            .register_type::<Positions>()
            .register_type::<ExtensionIndexOverride>()
            .add_plugins(BufferPlugin);
    }
}

#[derive(Debug, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ConfigComponents(Vec<Entity>);

#[derive(Debug, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct Sort(u32);

#[derive(Debug, Clone, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct Positions(pub Vec<Option<u8>>);

#[derive(Debug, Component, Reflect, Deref)]
#[reflect(Component)]
pub struct ExtensionIndexOverride(pub u8);

pub fn sort_components(mut configs: Query<&mut ConfigComponents>, components_sort: Query<&Sort>) {
    for mut components in &mut configs {
        components.sort_by_cached_key(|&entity| components_sort.get(entity).unwrap().0)
    }
}

pub fn init_component_positions(
    configs: Query<(&ConfigId, &ConfigComponents)>,
    mut components: Query<&mut Positions>,
) -> Option<()> {
    let config_count = configs.iter().count();

    for mut positions in &mut components {
        positions.resize(config_count, None);
    }

    for (id, comps) in &configs {
        for (i, &entity) in comps.iter().enumerate() {
            *components.get_mut(entity).ok()?.get_mut(id.0)? = Some(i as u8);
        }
    }

    Some(())
}

#[derive(Debug, Reflect)]
pub struct ComponentInfo {
    pub(crate) order: ComponentOrder,
    pub(crate) to_shader_info: ToComponentShaderInfo,
}

#[cfg(test)]
mod tests {
    /*
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
    */
}
