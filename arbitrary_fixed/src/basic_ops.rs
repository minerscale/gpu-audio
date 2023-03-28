use std::ops::Rem;
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::ArbitraryFixed;
use crate::SCALING_FACTOR;
use crate::SIZE;

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
}
