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
    type Item = usize;
    type IntoIter = FlagSetBitsIndexIterator;

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
