/*
Blazingly fast, eliminates an entire class of memory saftey bugs,
very sale of this software contributes towards Chritian Porter's legal fund.
*/

use std::{ops::{Add, Mul, Neg, Sub, AddAssign, MulAssign, SubAssign, Div, DivAssign, Rem, RemAssign}, cmp::Ordering, num::ParseIntError};
use bytemuck::{Pod, Zeroable};
use num_traits::{Zero, One, Num};

const SIZE: usize = 8;
const SCALING_FACTOR: usize = 32 * (SIZE/2);

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Ord, Pod, Zeroable)]
#[repr(C)]
pub struct ArbitraryFixed {
    pub data: [u32; SIZE],
}

impl Add for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn add(self, other: ArbitraryFixed) -> Self::Output {
        let mut ret: ArbitraryFixed = Default::default();

        let mut carry_prev = false;
        for ((r, a), b) in ret.data.iter_mut().zip(self.data).zip(other.data) {
            *r = a.wrapping_add(b);
            let carry = *r < a;
            *r = (carry_prev as u32).wrapping_add(*r);
            carry_prev = carry || (carry_prev && (*r == 0));
        }

        ret
    }
}

impl Neg for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn neg(self) -> Self::Output {
        let mut ret: ArbitraryFixed = Default::default();

        let mut carry_prev = true;
        for (r, a) in ret.data.iter_mut().zip(self.data) {
            *r = !a;
            *r = (carry_prev as u32).wrapping_add(*r);
            carry_prev = carry_prev && (*r == 0);
        }

        ret
    }
}

impl Sub for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn sub(self, rhs: ArbitraryFixed) -> Self::Output {
        self + -rhs
    }
}

impl Mul for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn mul(self, other: ArbitraryFixed) -> Self::Output {
        let a_negative = self.is_negative();
        let b_negative = other.is_negative();

        let fix_a = match a_negative {
            true => -self,
            false => self,
        };

        let fix_b = match b_negative {
            true => -other,
            false => other,
        };

        let mut res: [u32; SIZE * 2] = Default::default();

        for (i, a) in fix_a.data.iter().enumerate() {
            let mut carry = 0;
            for (j, b) in fix_b.data.iter().enumerate() {
                let product = (*a as u64) * (*b as u64) + (res[i + j] as u64) + (carry as u64);
                res[i + j] = product as u32;
                carry = (product >> 32) as u32
            }
            res[i + SIZE] = carry;
        }

        let mut ret: ArbitraryFixed = Default::default();

        for (idx, r) in ret.data.iter_mut().enumerate().rev() {
            *r = (if (SCALING_FACTOR % 32) > 0 {
                res[idx + 1 + SCALING_FACTOR / 32] << (((32 as usize).wrapping_sub(SCALING_FACTOR) % 32))
            } else {
                0
            }) | ((res[idx + (SCALING_FACTOR / 32)]) >> (SCALING_FACTOR % 32));
        }

        match a_negative != b_negative {
            true => -ret,
            false => ret
        }
    }
}

impl Div for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn div(self, other: ArbitraryFixed) -> ArbitraryFixed {
        let a_negative = self.is_negative();
        let b_negative = other.is_negative();

        let fix_a = match a_negative {
            true => -self,
            false => self,
        };

        let fix_b = match b_negative {
            true => -other,
            false => other,
        };

        let mut rem: [u32; SIZE * 2] = Default::default();
        let mut d: [u32; SIZE * 2] = Default::default();
        let mut res: [u32; SIZE * 2] = Default::default();
        for i in 0..SIZE {
            rem[i] = fix_a.data[i];
            d[i + SIZE] = fix_b.data[i];
        }

        for _ in 0..(2 * 32 * SIZE + 1) {
            res = ArbitraryFixed::lshift1_double(res);
            
            let rem_d = {
                let mut d_sub: [u32; 2 * SIZE] = Default::default();
                let mut carry_prev = true;
                for (r, &a) in d_sub.iter_mut().zip(d.iter()) {
                    *r = !a;
                    *r = (*r).wrapping_add(carry_prev as u32);
                    carry_prev = carry_prev && (*r == 0);
                }

                let mut ret: [u32; 2 * SIZE] = Default::default();
                let mut carry_prev = false;
                for ((r, &a), &b) in ret.iter_mut().zip(rem.iter()).zip(d_sub.iter()) {
                    *r = a.wrapping_add(b);
                    let carry = *r < a;
                    *r = (*r).wrapping_add(carry_prev as u32);
                    carry_prev = carry || (carry_prev && (*r == 0));
                }

                ret
            };
            //panic!("{:?}", rem_d);
            if (rem_d[2 * SIZE - 1] & 0x80000000) == 0 {
                res[0] |= 1;
                rem = rem_d;
            }

            d = ArbitraryFixed::rshift1_double(d);
        }

        let mut r: ArbitraryFixed = Default::default();
        for idx in 0..SIZE {
            r.data[idx] = (if (SCALING_FACTOR & 0x1F) > 0 {
                res[idx + SIZE - 1 - (SCALING_FACTOR / 32)] >> ((-(SCALING_FACTOR as isize)) & 0x1F)
            } else {
                0
            }) | ((res[idx + SIZE - (SCALING_FACTOR / 32)]) << (SCALING_FACTOR & 0x1F));
        }

        match a_negative != b_negative {
            true => -r,
            false => r
        }
    }
}

impl Rem for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn rem(self, rhs: ArbitraryFixed) -> Self::Output {
        let mut a = self / rhs;
    
        a.data[SCALING_FACTOR / 32] &= 0xFFFFFFFF >> (SCALING_FACTOR % 32);
        for i in 0..(SCALING_FACTOR / 32) {
            a.data[i] = 0;
        }

        a
    }
}

impl Zero for ArbitraryFixed {
    fn zero() -> ArbitraryFixed {
        ArbitraryFixed::default()
    }

    fn is_zero(&self) -> bool {
        for i in self.data {
            if i != 0 {
                return false
            }
        }

        return true
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
            return false
        }
        for &i in self.data.iter().skip(1) {
            if i != 0 {
                return false
            }
        }

        return true
    }
}

impl From<ArbitraryFixed> for f32 {
    fn from(a: ArbitraryFixed) -> Self {
        let mut res: f32 = 0.0;

        let a_negative = a.is_negative();

        let mut a = match a_negative {
            true => -a,
            false => a,
        };

        for (ind, r) in a.data.iter_mut().enumerate() {
            let factor = ((32 * ind as isize - SCALING_FACTOR as isize) as f32).exp2();

            if factor.is_finite() {
                res += (*r as f32) * factor;
            }
        }

        match a_negative {
            true => -res,
            false => res,
        }
    }
}

impl From<f32> for ArbitraryFixed {
    fn from(f: f32) -> Self {
        let mut ret: ArbitraryFixed = Default::default();
        let f_int = f.to_bits();
        let sign = (f_int & 0x80000000) > 0;
        let exponent = (f_int & 0x7f800000) >> 23;
        let mantissa_complete = (f_int & 0x007fffff) + (1 << 23);

        for (ind, r) in ret.data.iter_mut().enumerate() {
            let offset =
                -(32 * ind as isize) + SCALING_FACTOR as isize + (exponent as isize - 127) - 23;

            match (offset.abs() < 32, offset > 0) {
                (true, true) => *r = mantissa_complete << offset,
                (true, false) => *r = mantissa_complete >> -offset,
                (false, _) => *r = 0,
            }
        }

        match sign {
            true => -ret,
            false => ret,
        }
    }
}

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
        Ok((u32::from_str_radix(string, radix)? as f32).into())
    }
}

impl ArbitraryFixed {
    fn is_negative(&self) -> bool {
        (self.data[SIZE - 1] & 0x80000000) > 0
    }

    pub fn lshift1(a: [u32; SIZE]) -> [u32; SIZE] {
        let mut ret: [u32; SIZE] = Default::default();
        ret[0] = a[0] << 1;

        for i in 1..(SIZE) {
            ret[i] = (a[i] << 1) | (a[i - 1] >> 31);
        }

        ret
    }

    pub fn rshift1(a: [u32; SIZE]) -> [u32; SIZE] {
        let mut ret: [u32; SIZE] = Default::default();
        ret[SIZE - 1] = a[SIZE - 1] >> 1;

        for i in (0..(SIZE - 1)).rev() {
            ret[i] = (a[i + 1] << 31) | (a[i] >> 1);
        }
        
        ret
    }

    fn lshift1_double(a: [u32; SIZE * 2]) -> [u32; SIZE * 2] {
        let mut ret: [u32; SIZE * 2] = Default::default();
        ret[0] = a[0] << 1;

        for i in 1..(SIZE * 2) {
            ret[i] = (a[i] << 1) | (a[i - 1] >> 31);
        }

        ret
    }

    fn rshift1_double(a: [u32; SIZE * 2]) -> [u32; SIZE * 2] {
        let mut ret: [u32; SIZE * 2] = Default::default();
        ret[2*SIZE - 1] = a[2*SIZE - 1] >> 1;

        for i in (0..(2 * SIZE - 1)).rev() {
            ret[i] = (a[i + 1] << 31) | (a[i] >> 1);
        }
        
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let (fa, fb): (f32, f32) = (3.0, 5.3);
        let (a, b): (ArbitraryFixed, ArbitraryFixed) = (fa.into(), fb.into());
        assert_eq!(a + b, (fa + fb).into());
    }

    #[test]
    fn test_sub() {
        let (fa, fb): (f32, f32) = (3.0, 5.3);
        let (a, b): (ArbitraryFixed, ArbitraryFixed) = (fa.into(), fb.into());
        assert_eq!(a - b, (fa - fb).into());
    }

    #[test]
    fn test_mul() {
        let (fa, fb): (f32, f32) = (3.0, 5.3);
        let (a, b): (ArbitraryFixed, ArbitraryFixed) = (fa.into(), fb.into());
        assert_eq!(a * b, (fa * fb).into());
    }

    #[test]
    fn test_lshift1() {
        let fa: f32 = 3.1;
        let a: ArbitraryFixed = fa.into();
        assert_eq!(ArbitraryFixed::lshift1(a.data), ArbitraryFixed::from(fa * 2.0).data);
    }

    #[test]
    fn test_rshift1() {
        let fa: f32 = 3.1;
        let a: ArbitraryFixed = fa.into();
        assert_eq!(ArbitraryFixed::rshift1(a.data), ArbitraryFixed::from(fa / 2.0).data);
    }

    #[test]
    fn test_div() {
        let fa: f32 = 3.0;
        let fb: f32 = 5.3;
        let a: ArbitraryFixed = fa.into();
        let b: ArbitraryFixed = fb.into();
        assert_eq!(f32::from(a / b), (fa / fb));
    }
}
