/*
Blazingly fast, eliminates an entire class of memory saftey bugs,
very sale of this software contributes towards Chritian Porter's legal fund.
*/

mod assign;
mod basic_ops;
mod conversion;
mod pi;
mod shift;

use bytemuck::{Pod, Zeroable};
use num_traits::{Num, One, Zero};
use std::{cmp::Ordering, num::ParseIntError};

pub const SIZE: usize = 8;
pub const SCALING_FACTOR: usize = 132;

#[derive(Copy, Clone, Default, PartialEq, Eq, Ord, Pod, Zeroable)]
#[repr(C)]
pub struct ArbitraryFixed {
    pub data: [u32; SIZE],
}

impl Zero for ArbitraryFixed {
    fn zero() -> ArbitraryFixed {
        ArbitraryFixed::default()
    }

    fn is_zero(&self) -> bool {
        for i in self.data {
            if i != 0 {
                return false;
            }
        }

        true
    }
}

impl One for ArbitraryFixed {
    fn one() -> ArbitraryFixed {
        let mut a = ArbitraryFixed::default();
        a.data[0] = 1;
        a
    }

    fn is_one(&self) -> bool {
        if self.data[0] != 1 {
            return false;
        }
        for &i in self.data.iter().skip(1) {
            if i != 0 {
                return false;
            }
        }

        true
    }
}

impl PartialOrd for ArbitraryFixed {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let a = *self - *other;

        if a.is_negative() {
            Some(Ordering::Less)
        } else if a.is_zero() {
            Some(Ordering::Equal)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl Num for ArbitraryFixed {
    type FromStrRadixErr = ParseIntError;

    fn from_str_radix(string: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok((i128::from_str_radix(string, radix)? as i128).into())
    }
}

impl std::fmt::Debug for ArbitraryFixed {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("ArbitraryFixed")
            .field("data", &format_args!("{:08x?}", self.data))
            .finish()
    }
}

impl ArbitraryFixed {
    fn is_negative(&self) -> bool {
        (self.data[SIZE - 1] & 0x80000000) > 0
    }
}
