/*
Blazingly fast, eliminates an entire class of memory saftey bugs,
very sale of this software contributes towards Chritian Porter's legal fund.
*/

use bytemuck::{Pod, Zeroable};
use num_traits::{Num, One, Zero};
use std::{
    cmp::Ordering,
    num::ParseIntError,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
    sync::mpsc,
    thread,
};

pub const SIZE: usize = 8;
pub const SCALING_FACTOR: usize = 132;

#[derive(Copy, Clone, Default, PartialEq, Eq, Ord, Pod, Zeroable)]
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
                res[idx + 1 + SCALING_FACTOR / 32]
                    << ((32 as usize).wrapping_sub(SCALING_FACTOR) % 32)
            } else {
                0
            }) | ((res[idx + (SCALING_FACTOR / 32)]) >> (SCALING_FACTOR % 32));
        }

        match a_negative != b_negative {
            true => -ret,
            false => ret,
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
            res = {
                let mut ret: [u32; SIZE * 2] = Default::default();
                ret[0] = res[0] << 1;

                for i in 1..(SIZE * 2) {
                    ret[i] = (res[i] << 1) | (res[i - 1] >> 31);
                }

                ret
            };

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

            d = {
                let mut ret: [u32; SIZE * 2] = Default::default();
                ret[2 * SIZE - 1] = d[2 * SIZE - 1] >> 1;

                for i in (0..(2 * SIZE - 1)).rev() {
                    ret[i] = (d[i + 1] << 31) | (d[i] >> 1);
                }

                ret
            }
        }

        let mut r: ArbitraryFixed = Default::default();
        for idx in 0..SIZE {
            r.data[idx] = (if (SCALING_FACTOR & 0x1F) > 0 {
                res[idx + SIZE - 1 - (SCALING_FACTOR / 32)] >> ((-(SCALING_FACTOR as isize)) & 0x1F)
            } else {
                0
            }) | ((res[idx + SIZE - (SCALING_FACTOR / 32)])
                << (SCALING_FACTOR & 0x1F));
        }

        match a_negative != b_negative {
            true => -r,
            false => r,
        }
    }
}

impl Rem for ArbitraryFixed {
    type Output = ArbitraryFixed;

    fn rem(self, rhs: ArbitraryFixed) -> Self::Output {
        let mut a = self / rhs;

        a.data[SCALING_FACTOR / 32] &= 0xFFFFFFFF << (SCALING_FACTOR % 32);
        for i in 0..(SCALING_FACTOR / 32) {
            a.data[i] = 0;
        }

        self - a * rhs
    }
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

        return true;
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

        return true;
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

impl From<u128> for ArbitraryFixed {
    fn from(a: u128) -> Self {
        let mut ret: ArbitraryFixed = Default::default();

        for (ind, r) in ret.data.iter_mut().enumerate() {
            let offset = -(32 * ind as isize) + SCALING_FACTOR as isize;

            match (offset.abs() < 128, offset > 0) {
                (true, true) => *r = (a << offset) as u32,
                (true, false) => *r = (a >> -offset) as u32,
                (false, _) => *r = 0,
            }
        }

        ret
    }
}

impl From<u32> for ArbitraryFixed {
    fn from(a: u32) -> Self {
        let mut ret: ArbitraryFixed = Default::default();

        ret.data[SCALING_FACTOR/32] = a << (SCALING_FACTOR % 32);
        if (SCALING_FACTOR % 32) != 0 {
            ret.data[SCALING_FACTOR/32 + 1] = a >> ((-(SCALING_FACTOR as i32)) & 0x1F);
        }
        
        ret
    }
}


impl From<i128> for ArbitraryFixed {
    fn from(a: i128) -> Self {
        let mut ret: ArbitraryFixed = Default::default();

        let normalised: u128 = a.abs().try_into().unwrap();

        for (ind, r) in ret.data.iter_mut().enumerate() {
            let offset = -(32 * ind as isize) + SCALING_FACTOR as isize;

            match (offset.abs() < 128, offset > 0) {
                (true, true) => *r = (normalised << offset) as u32,
                (true, false) => *r = (normalised >> -offset) as u32,
                (false, _) => *r = 0,
            }
        }

        if a < 0 {
            -ret
        } else {
            ret
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

        let offset = (SCALING_FACTOR as isize) + (exponent as isize - 127 - 23);
        if offset >= 0 {
            ret.data[(offset/32) as usize] = mantissa_complete << (offset % 32);
        }
        if ((SCALING_FACTOR % 32) != 0) && (offset >= -1) {
            ret.data[(offset/32) as usize + 1] = mantissa_complete >> ((-(offset as i32)) & 0x1F);
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

impl std::fmt::Debug for ArbitraryFixed {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("ArbitraryFixed")
            // ...
            .field("data", &format_args!("{:08x?}", self.data))
            // ...
            .finish()
    }
}

impl ArbitraryFixed {
    fn is_negative(&self) -> bool {
        (self.data[SIZE - 1] & 0x80000000) > 0
    }

    pub fn lshift1(&self) -> Self {
        let mut ret: ArbitraryFixed = Default::default();
        ret.data[0] = self.data[0] << 1;

        for i in 1..(SIZE) {
            ret.data[i] = (self.data[i] << 1) | (self.data[i - 1] >> 31);
        }

        ret
    }

    pub fn lshiftword(&self, n: u32) -> Self {
        if n == 0 {
            return self.clone();
        }
        let mut ret: ArbitraryFixed = Default::default();
        ret.data[0] = self.data[0] << n;

        for i in 1..(SIZE) {
            ret.data[i] = (self.data[i] << n) | (self.data[i - 1] >> (32 - n));
        }

        ret
    }

    pub fn rshift1(&self) -> Self {
        let mut ret: ArbitraryFixed = Default::default();
        ret.data[SIZE - 1] = self.data[SIZE - 1] >> 1;

        for i in (0..(SIZE - 1)).rev() {
            ret.data[i] = (self.data[i + 1] << 31) | (self.data[i] >> 1);
        }

        ret
    }

    pub fn rshiftword(&self, n: u32) -> Self {
        if n == 0 {
            return self.clone();
        }
        let mut ret: ArbitraryFixed = Default::default();
        ret.data[SIZE - 1] = self.data[SIZE - 1] >> n;

        for i in (0..(SIZE - 1)).rev() {
            ret.data[i] = (self.data[i + 1] << (32 - n)) | (self.data[i] >> n);
        }

        ret
    }

    // Generate PI, adapted from https://github.com/itchyny/pihex-rs
    // TODO, licensing
    pub fn gen_pi() -> Self {
        fn pihex(d: u64) -> u16 {
            let (tx, rx) = mpsc::channel();
            for &(j, k, l) in &[
                (4, 1, -32.0),
                (4, 3, -1.0),
                (10, 1, 256.0),
                (10, 3, -64.0),
                (10, 5, -4.0),
                (10, 7, -4.0),
                (10, 9, 1.0),
            ] {
                let tx = tx.clone();
                thread::spawn(move || tx.send(l * series_sum(d, j, k)).unwrap());
            }
            drop(tx);
            let fraction: f64 = rx.iter().sum();
            (0..4)
                .scan(fraction, |x, k| {
                    *x = (*x - x.floor()) * 16.0;
                    Some((x.floor() as u16) << (12 - 4 * k))
                })
                .fold(0, |s, t| s + &t)
        }

        fn series_sum(d: u64, j: u64, k: u64) -> f64 {
            let fraction1: f64 = (0..(2 * d + 2) / 5)
                .map(|i| {
                    (if i % 2 == 0 { 1.0 } else { -1.0 })
                        * pow_mod(4, 2 * d - 3 - 5 * i, j * i + k) as f64
                        / (j * i + k) as f64
                })
                .fold(0.0, |x, y| (x + y).fract());
            let fraction2: f64 = ((2 * d + 2) / 5..)
                .map(|i| -(-4.0_f64).powi(-((5 * i + 3 - 2 * d) as i32)) / ((j * i + k) as f64))
                .take_while(|&x| x.abs() > 1e-13_f64)
                .sum();
            fraction1 + fraction2
        }
        let mut ret: ArbitraryFixed = Default::default();

        ret.data[0] |= 3;
        for i in 0..(SCALING_FACTOR / 16) {
            ret = ret.lshiftword(16);
            ret.data[0] |= pihex(4 * i as u64) as u32;
        }
        ret = ret.lshiftword((SCALING_FACTOR % 16) as u32);
        ret.data[0] |=
            (pihex(4 * (SCALING_FACTOR / 16) as u64) >> (-(SCALING_FACTOR as i32) & 0x0F)) as u32;

        ret
    }
}

pub fn pow_mod(n: u64, m: u64, d: u64) -> u64 {
    if n < 100 && d < 400_000_000 {
        // k * k * n < 2^64 - 1
        pow_mod_inner(n, m, d)
    } else {
        pow_mod_inner(n as u128, m as u128, d as u128) as u64
    }
}

fn pow_mod_inner<T>(n: T, m: T, d: T) -> T
where
    T: Copy
        + std::cmp::PartialEq
        + std::ops::Mul<Output = T>
        + std::ops::Div<Output = T>
        + std::ops::Rem<Output = T>
        + std::convert::From<u64>,
{
    if m == 0.into() {
        if d == 1.into() {
            0.into()
        } else {
            1.into()
        }
    } else if m == 1.into() {
        n % d
    } else {
        let k = pow_mod_inner(n, m / 2.into(), d);
        if m % 2.into() == 0.into() {
            (k * k) % d
        } else {
            (k * k * n) % d
        }
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
        assert_eq!(a.lshift1(), ArbitraryFixed::from(fa * 2.0));
    }

    #[test]
    fn test_rshift1() {
        let fa: f32 = 3.1;
        let a: ArbitraryFixed = fa.into();
        assert_eq!(a.rshift1(), ArbitraryFixed::from(fa / 2.0));
    }

    #[test]
    fn test_u32_from() {
        let au: u32 = 151785;
        let a: ArbitraryFixed = au.into();
        assert_eq!(au, f32::from(a) as u32);
    }

    #[test]
    fn test_div() {
        let fa: f32 = 3.0;
        let fb: f32 = 5.3;
        let a: ArbitraryFixed = fa.into();
        let b: ArbitraryFixed = fb.into();
        assert_eq!(f32::from(a / b), (fa / fb));
    }

    #[test]
    fn test_rem() {
        let fa: f32 = 3.0;
        let fb: f32 = 5.3;
        let a: ArbitraryFixed = fa.into();
        let b: ArbitraryFixed = fb.into();
        assert_eq!(f32::from(a % b), (fa % fb));
    }

    #[test]
    fn test_pi() {
        let a = ArbitraryFixed::gen_pi();
        assert_eq!(f32::from(a), std::f32::consts::PI);
    } 

    #[test]
    fn test_from_f32() {
        let fa: f32 = 3.141592;
        let a: ArbitraryFixed = fa.into();
        println!("{:?}", a);
        assert_eq!(f32::from(a), fa);
    }
}
