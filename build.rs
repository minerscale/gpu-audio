// build.rs

use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use arbitrary_fixed::ArbitraryFixed;
use arbitrary_fixed_glsl::write_table;

fn main() -> std::io::Result<()> {
    let in_str = fs::read_to_string("src/cs.rs")?;
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("cs.rs");
    fs::write(
        &dest_path,
        in_str.replace(
            "INCLUDE_PATH",
            &format!(
                "\"{}\", \"{}\"",
                arbitrary_fixed_glsl::include_dir(),
                out_dir.to_str().unwrap()
            ),
        ),
    )?;
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/cs.rs");

    let const_path = Path::new(&out_dir).join("local_consts.glsl");

    let mut f = BufWriter::new(fs::File::create(const_path)?);

    const INV_SQRT_SIZE: usize = 100;

    writeln!(f, "const uint INV_SQRT_SIZE = {INV_SQRT_SIZE};")?;

    write_table(&mut f, "inv_sqrt_table", INV_SQRT_SIZE, |i| {
            ArbitraryFixed::from(1u32) / ArbitraryFixed::from((i + 1) as u32).sqrt()
        })?;

    Ok(())
}
