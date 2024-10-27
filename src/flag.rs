use crate::components::SdfCompInfos;
use bevy::prelude::*;

pub struct FlagPlugin;
impl Plugin for FlagPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        app.world_mut()
            .resource_scope(|world, infos: Mut<SdfCompInfos>| {
                infos.iter().enumerate().for_each(|(i, info)| {
                    (info.insert_arena)(world, i as u8);
                });
            });
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Flag(pub(crate) u32);

impl Flag {
    pub fn set(&mut self, position: u8) {
        self.0 |= 1 << position;
    }

    pub fn unset(&mut self, position: u8) {
        self.0 &= !(1 << position);
    }
}

impl IntoIterator for Flag {
    type IntoIter = FlagSetBitsIndexIterator;
    type Item = usize;

    fn into_iter(self) -> Self::IntoIter {
        FlagSetBitsIndexIterator { flag: self.0 }
    }
}

pub struct FlagSetBitsIndexIterator {
    flag: u32,
}

impl Iterator for FlagSetBitsIndexIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.flag == 0 {
            None
        } else {
            let index = self.flag.trailing_zeros();
            self.flag &= self.flag - 1;
            Some(index as usize)
        }
    }
}
