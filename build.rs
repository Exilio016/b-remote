use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() {
    // Tell Cargo to re-run this script if any file in the assets folder changes
    println!("cargo:rerun-if-changed=assets");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("asset_map.rs");
    let mut f = File::create(&dest_path).unwrap();

    // Start generating the static slice code
    writeln!(f, "static ASSET_MAP: &[(&str, &[u8])] = &[").unwrap();

    // Read the assets directory
    let assets_dir = Path::new(&manifest_dir).join("assets");
    if let Ok(entries) = fs::read_dir(&assets_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_stem().unwrap().to_str().unwrap();
                // Use absolute path for include_bytes!
                let abs_path = path.canonicalize().unwrap();
                writeln!(
                    f,
                    "    (\"{}\", include_bytes!(\"{}\")),",
                    name, abs_path.display()
                ).unwrap();
            }
        }
    }

    writeln!(f, "];").unwrap();
}
