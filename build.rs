use std::fs::File;
use std::io::{BufWriter, Write};

use arbitrary_fixed::ArbitraryFixed;

fn factorial(num: u128) -> u128 {
    (1..=num).product()
}

fn arb_to_array(a: ArbitraryFixed) -> String {
    a.data
        .iter()
        .map(|k| format!("    {:#010X}", k))
        .collect::<Vec<_>>()
        .join(",\n")
}

fn gen_table(len: usize, f: fn(usize) -> ArbitraryFixed) -> String {
    (0..len)
        .map(|i| format!("  {{\n{}\n  }}", arb_to_array(f(i))))
        .collect::<Vec<_>>()
        .join(",\n")
}

// Example custom build script.
fn main() -> std::io::Result<()> {
    // Tell Cargo that if the given file changes, to rerun this build script.
    //println!("cargo:rerun-if-changed=src/generate_consts.py");

    let mut f = BufWriter::new(File::create("target/constants.glsl")?);

    writeln!(f, "const uint SIZE = {};", arbitrary_fixed::SIZE)?;
    writeln!(
        f,
        "const uint SCALING_FACTOR = {};\n",
        arbitrary_fixed::SCALING_FACTOR
    )?;

    const TRIG_PRECISION: usize = 6;
    const LOG_PRECISION: usize = 6;
    const INV_SQRT_SIZE: usize = 100;

    writeln!(f, "const uint TRIG_PRECISION = {TRIG_PRECISION};")?;
    writeln!(f, "const uint LOG_PRECISION = {LOG_PRECISION};")?;
    writeln!(f, "const uint INV_SQRT_SIZE = {INV_SQRT_SIZE};")?;

    writeln!(f, "const uint[TRIG_PRECISION][SIZE] sin_table = {{")?;
    writeln!(
        f,
        "{}",
        gen_table(TRIG_PRECISION, |i| {
            let adj = TRIG_PRECISION - i;
            ArbitraryFixed::from(1 - (2 * ((adj % 2) as i128)))
                / ArbitraryFixed::from(factorial(2 * adj as u128))
        })
    )?;
    writeln!(f, "}};\n")?;

    writeln!(f, "const uint[LOG_PRECISION][SIZE] log_table = {{")?;
    writeln!(
        f,
        "{}",
        gen_table(LOG_PRECISION, |i| {
            let adj = LOG_PRECISION - i - 1;
            ArbitraryFixed::from(2u32) / ArbitraryFixed::from(2 * (adj as u32) + 1)
        })
    )?;
    writeln!(f, "}};\n")?;

    writeln!(f, "const uint[INV_SQRT_SIZE][SIZE] inv_sqrt_table = {{")?;
    writeln!(
        f,
        "{}",
        gen_table(INV_SQRT_SIZE, |i| {
            ArbitraryFixed::from(1u32) / ArbitraryFixed::from((i + 1) as u32).sqrt()
        })
    )?;
    writeln!(f, "}};\n")?;

    writeln!(f, "const uint[SIZE] FIX_2_PI = {{")?;
    writeln!(f, "{}", arb_to_array(ArbitraryFixed::gen_pi().lshift1()))?;
    writeln!(f, "}};\n")?;

    writeln!(f, "const uint[SIZE] FIX_PI = {{")?;
    writeln!(f, "{}", arb_to_array(ArbitraryFixed::gen_pi()))?;
    writeln!(f, "}};\n")?;

    writeln!(f, "const uint[SIZE] FIX_PI_2 = {{")?;
    writeln!(f, "{}", arb_to_array(ArbitraryFixed::gen_pi().rshift1()))?;
    writeln!(f, "}};\n")?;

    writeln!(f, "const uint[SIZE] FIX_LN_2 = {{")?;
    writeln!(f, "{}", arb_to_array(ArbitraryFixed::gen_ln_2()))?;
    writeln!(f, "}};")?;

    writeln!(f, "const uint[SIZE] FIX_ZERO = {{")?;
    writeln!(f, "{}", arb_to_array(ArbitraryFixed::from(0u32)))?;
    writeln!(f, "}};")?;

    writeln!(f, "const uint[SIZE] FIX_ONE = {{")?;
    writeln!(f, "{}", arb_to_array(ArbitraryFixed::from(1u32)))?;
    writeln!(f, "}};")?;

    writeln!(f, "const uint[SIZE] FIX_NEG_ONE = {{")?;
    writeln!(f, "{}", arb_to_array(ArbitraryFixed::from(-1i128)))?;
    writeln!(f, "}};")?;
    Ok(())
}
