// Generate PI, adapted (stolen) from https://github.com/itchyny/pihex-rs
// INSERT MIT LICENSE HERE

use std::{sync::mpsc, thread};

use crate::{ArbitraryFixed, SCALING_FACTOR};

impl ArbitraryFixed {
    pub fn gen_pi() -> Self {
        let mut ret: ArbitraryFixed = Default::default();

        ret.data[0] |= 3;
        for i in 0..(SCALING_FACTOR / 16) {
            ret = ret.lshiftword(16);
            ret.data[0] |= pihex(4 * i as u64) as u32;
        }
        ret = ret.lshiftword(SCALING_FACTOR % 16);
        ret.data[0] |=
            (pihex(4 * (SCALING_FACTOR / 16) as u64) >> (-(SCALING_FACTOR as i32) & 0x0F)) as u32;

        ret
    }

    pub fn gen_ln_2() -> Self {
        let mut ret: ArbitraryFixed = Default::default();

        for i in 0..(SCALING_FACTOR / 16) {
            ret = ret.lshiftword(16);
            ret.data[0] |= logbin(16 * i as u64) as u32;
        }
        ret = ret.lshiftword(SCALING_FACTOR % 16);
        ret.data[0] |=
            (logbin(16 * (SCALING_FACTOR / 16) as u64) >> (-(SCALING_FACTOR as i32) & 0x0F)) as u32;

        ret
    }
}

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
        thread::spawn(move || tx.send(l * series_sum_pi(d, j, k)).unwrap());
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

pub fn logbin(d: u64) -> u16 {
    let fraction: f64 = series_sum_log_2(d);
    (0..16)
        .scan(fraction, |x, k| {
            *x = (*x - x.floor()) * 2.0;
            Some((x.floor() as u16) << (15 - k))
        })
        .fold(0, |s, t| s + &t)
}

fn series_sum_log_2(d: u64) -> f64 {
    let fraction1: f64 = (1..d + 1)
        .map(|i| pow_mod(2, d - i, i) as f64 / i as f64)
        .fold(0.0, |x, y| (x + y).fract());
    let fraction2: f64 = (d + 1..)
        .map(|i| 2.0_f64.powi(-((i - d) as i32)) / (i as f64))
        .take_while(|&x| x > 1e-13_f64)
        .sum();
    fraction1 + fraction2
}

fn series_sum_pi(d: u64, j: u64, k: u64) -> f64 {
    let fraction1: f64 = (0..(2 * d + 2) / 5)
        .map(|i| {
            (if i % 2 == 0 { 1.0 } else { -1.0 }) * pow_mod(4, 2 * d - 3 - 5 * i, j * i + k) as f64
                / (j * i + k) as f64
        })
        .fold(0.0, |x, y| (x + y).fract());
    let fraction2: f64 = ((2 * d + 2) / 5..)
        .map(|i| -(-4.0_f64).powi(-((5 * i + 3 - 2 * d) as i32)) / ((j * i + k) as f64))
        .take_while(|&x| x.abs() > 1e-13_f64)
        .sum();
    fraction1 + fraction2
}

fn pow_mod(n: u64, m: u64, d: u64) -> u64 {
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
    fn test_pi() {
        let a = ArbitraryFixed::gen_pi();
        assert_eq!(f32::from(a), std::f32::consts::PI);
    }

    #[test]
    fn test_ln_2() {
        let a = ArbitraryFixed::gen_ln_2();
        assert_eq!(f32::from(a), 2.0f32.ln());
    }
}
