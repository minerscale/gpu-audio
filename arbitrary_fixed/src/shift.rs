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
}
