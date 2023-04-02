use crate::{ArbitraryFixed, SIZE};

impl ArbitraryFixed {
    pub fn lshift1(&self) -> Self {
        let mut ret: ArbitraryFixed = Default::default();
        ret.data[0] = self.data[0] << 1;

        for i in 1..(SIZE) {
            ret.data[i] = (self.data[i] << 1) | (self.data[i - 1] >> 31);
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

    pub fn lshiftword(&self, n: usize) -> Self {
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

    pub fn rshiftword(&self, n: usize) -> Self {
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

    pub fn lshift(&self, n: usize) -> Self {
        if n == 0 {
            return self.clone();
        }
        let mut ret: ArbitraryFixed = Default::default();

        for i in 0..(n / 32) {
            ret.data[i] = 0;
        }
        ret.data[n / 32] = self.data[0] << (n & 0x1F);

        for i in (n / 32 + 1)..(SIZE) {
            ret.data[i] = (self.data[i - n / 32] << (n & 0x1F))
                | ((((n & 0x1F) != 0) as u32)
                    * (self.data[i - 1 - n / 32] >> ((-(n as isize)) & 0x1F)));
        }

        ret
    }

    pub fn rshift(&self, n: usize) -> Self {
        if n == 0 {
            return self.clone();
        }
        let mut ret: ArbitraryFixed = Default::default();
        for i in 0..(n / 32) {
            ret.data[SIZE - i - 1] = 0;
        }

        ret.data[SIZE - 1 - (n / 32)] = self.data[SIZE - 1] >> (n & 0x1F);

        for i in (0..=(SIZE - (n / 32) - 2)).rev() {
            ret.data[i] = ((((n & 0x1F) != 0) as u32)
                * (self.data[i + 1 + n / 32] << ((-(n as isize)) & 0x1F)))
                | (self.data[i + n / 32] >> (n & 0x1F));
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_lshiftword() {
        let fa: f32 = 3.1;
        let a: ArbitraryFixed = fa.into();
        assert_eq!(
            a.lshiftword(24),
            ArbitraryFixed::from((2.0f32).powi(24) * fa)
        );
    }

    #[test]
    fn test_rshiftword() {
        let fa: f32 = 3.1;
        let a: ArbitraryFixed = fa.into();
        assert_eq!(
            a.rshiftword(24),
            ArbitraryFixed::from((2.0f32).powi(-24) * fa)
        );
    }

    #[test]
    fn test_lshift() {
        let fa: f32 = 3.141592;
        let a: ArbitraryFixed = fa.into();
        assert_eq!(a.lshift(32), ArbitraryFixed::from((2.0f32).powi(32) * fa));
    }

    #[test]
    fn test_rshift() {
        let a: ArbitraryFixed = ArbitraryFixed::gen_pi();

        assert_eq!(f32::from(a.rshift(32)), (2.0f32).powi(-32) * f32::from(a));
    }
}
