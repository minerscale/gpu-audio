use std::fs::File;
use std::io::{BufWriter, Write};

use arbitrary_fixed::ArbitraryFixed;

fn factorial(num: u128) -> u128 {
    (1..=num).product()
}

// Example custom build script.
fn main() -> std::io::Result<()> {
    // Tell Cargo that if the given file changes, to rerun this build script.
    //println!("cargo:rerun-if-changed=src/generate_consts.py");

    // Need to generate taylor series constants, spit them out as glsl-style arrays
    // Multidimensional array?
    // e.g. const uint[SIZE][] taylor_table = {
    //  {blah, blahdy, bla, blews, aont...},
    //  {blah, blahdy, bla, blews, aont...}
    //}

    let mut f = BufWriter::new(File::create("target/constants.glsl")?);

    writeln!(f, "const uint SIZE = {};", arbitrary_fixed::SIZE)?;
    writeln!(
        f,
        "const uint SCALING_FACTOR = {};\n",
        arbitrary_fixed::SCALING_FACTOR
    )?;

    const TRIG_PRECISION: usize = 6;

    writeln!(f, "const uint TRIG_PRECISION = {TRIG_PRECISION};")?;
    writeln!(f, "const uint[TRIG_PRECISION][SIZE] taylor_table = {{")?;
    writeln!(
        f,
        "{}",
        (1..=TRIG_PRECISION)
            .rev()
            .map(|i| {
                let a = ArbitraryFixed::from(1 - (2 * (i % 2) as i128))
                    / ArbitraryFixed::from(factorial(2 * i as u128));
                println!("{}", f32::from(a));
                format!(
                    "    {{\n{}\n    }}",
                    (a).data
                        .iter()
                        .map(|k| format!("        {:#010X}", k))
                        .collect::<Vec<_>>()
                        .join(",\n")
                )
            })
            .collect::<Vec<_>>()
            .join(",\n")
    )?;
    writeln!(f, "}};\n")?;

    writeln!(f, "const uint[SIZE] FIX_PI = {{")?;

    writeln!(
        f,
        "{}",
        ArbitraryFixed::gen_pi()
            .data
            .iter()
            .map(|k| format!("    {:#010X}", k))
            .collect::<Vec<_>>()
            .join(",\n")
    )?;
    writeln!(f, "}};")?;

    Ok(())
}
