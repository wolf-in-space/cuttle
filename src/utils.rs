use bevy::ecs::{
    system::Resource,
    world::{Mut, World},
};

pub(crate) trait GetOrInitResourceWorldExt {
    fn resource_or_init<R: Resource + Default>(&mut self) -> Mut<R>;
}

impl GetOrInitResourceWorldExt for World {
    fn resource_or_init<R: Resource + Default>(&mut self) -> Mut<R> {
        if !self.contains_resource::<R>() {
            self.init_resource::<R>();
        }
        self.resource_mut::<R>()
    }
}
