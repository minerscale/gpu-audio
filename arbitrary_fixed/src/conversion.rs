use crate::{ArbitraryFixed, SCALING_FACTOR};

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

impl From<ArbitraryFixed> for u32 {
    fn from(a: ArbitraryFixed) -> Self {
        a.rshift(SCALING_FACTOR).data[0]
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

        ret.data[SCALING_FACTOR / 32] = a << (SCALING_FACTOR % 32);
        if (SCALING_FACTOR % 32) != 0 {
            ret.data[SCALING_FACTOR / 32 + 1] = a >> ((-(SCALING_FACTOR as i32)) & 0x1F);
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
            ret.data[(offset / 32) as usize] = mantissa_complete << (offset % 32);
        }
        if ((offset % 32) != 0) && (offset >= -1) {
            ret.data[(offset / 32) as usize + 1] = mantissa_complete >> ((-(offset as i32)) & 0x1F);
        }

        match sign {
            true => -ret,
            false => ret,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_from() {
        let au: u32 = 15178538;
        let a: ArbitraryFixed = au.into();
        assert_eq!(au, f32::from(a) as u32);
    }

    #[test]
    fn test_from_f32() {
        let fa: f32 = 3.141592;
        let a: ArbitraryFixed = fa.into();
        println!("{:?}", a);
        assert_eq!(f32::from(a), fa);
    }

    #[test]
    fn test_to_u32() {
        let fa: f32 = 316.141592;
        let a: ArbitraryFixed = fa.into();
        println!("{:?}", a);
        assert_eq!(u32::from(a), fa as u32);
    }
}
