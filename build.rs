use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() {
    // Tell Cargo to re-run this script if any file in the assets folder changes
    println!("cargo:rerun-if-changed=assets");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("asset_map.rs");
    let mut f = File::create(&dest_path).unwrap();

    // Start generating the static slice code
    writeln!(f, "static ASSET_MAP: &[(&str, &[u8])] = &[").unwrap();

    // Read the assets directory
    if let Ok(entries) = fs::read_dir("assets") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let name = path.file_stem().unwrap().to_str().unwrap();
                // Format the inclusion macro pointing to the assets directory
                writeln!(
                    f,
                    "    (\"{}\", include_bytes!(\"../../../assets/{}\")),",
                    name, file_name
                ).unwrap();
            }
        }
    }

    writeln!(f, "];").unwrap();
}
