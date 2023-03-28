use crate::ArbitraryFixed;
use std::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};

impl AddAssign for ArbitraryFixed {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl SubAssign for ArbitraryFixed {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl MulAssign for ArbitraryFixed {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}

impl DivAssign for ArbitraryFixed {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl RemAssign for ArbitraryFixed {
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}
