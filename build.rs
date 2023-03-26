use arbitrary_fixed::ArbitraryFixed;

// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/generate_consts.py");

}
