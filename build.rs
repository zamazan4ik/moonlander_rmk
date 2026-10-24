use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{env, io};

use const_gen::*;
use xz2::read::XzEncoder;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::copy("memory.x", out.join("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rerun-if-changed=vial.json");
    generate_vial_config(&out);
}

fn generate_vial_config(out: &Path) {
    let vial_path = Path::new("vial.json");
    let mut content = String::new();
    File::open(vial_path)
        .and_then(|mut f| f.read_to_string(&mut content).map(|_| ()))
        .unwrap_or_else(|e: io::Error| panic!("cannot read vial.json: {e}"));

    let definition = json::parse(&content).expect("vial.json is not valid JSON");
    validate_moonlander_layout(&definition);
    let vial_cfg = json::stringify(definition);
    let mut compressed: Vec<u8> = Vec::new();
    XzEncoder::new(vial_cfg.as_bytes(), 6)
        .read_to_end(&mut compressed)
        .expect("failed to xz-compress vial.json");

    // Stable UID for this firmware; Vial identifies the keyboard by this.
    let keyboard_id: Vec<u8> = vec![0x4D, 0x6F, 0x6F, 0x6E, 0x4D, 0x6B, 0x31, 0x01];

    let declarations = [
        const_declaration!(pub VIAL_KEYBOARD_DEF = compressed),
        const_declaration!(pub VIAL_KEYBOARD_ID = keyboard_id),
    ]
    .map(|s| format!("#[allow(clippy::redundant_static_lifetimes)]\n{s}"))
    .join("\n");
    fs::write(out.join("config_generated.rs"), declarations).unwrap();
}

/// Reject missing, duplicate and phantom switches before embedding the Vial definition.
fn validate_moonlander_layout(definition: &json::JsonValue) {
    assert_eq!(definition["matrix"]["rows"].as_usize(), Some(12));
    assert_eq!(definition["matrix"]["cols"].as_usize(), Some(7));
    let mut seen = [[false; 7]; 12];
    for line in definition["layouts"]["keymap"].members() {
        for item in line.members() {
            let Some(label) = item.as_str() else {
                continue;
            };
            let (row, col) = label.split_once(',').expect("invalid Vial coordinate");
            let row: usize = row.parse().expect("invalid Vial row");
            let col: usize = col.parse().expect("invalid Vial column");
            assert!(row < 12 && col < 7, "Vial coordinate out of range");
            assert!(!seen[row][col], "duplicate Vial coordinate: {label}");
            seen[row][col] = true;
        }
    }
    for (row, columns) in seen.iter().enumerate() {
        for (col, &present) in columns.iter().enumerate() {
            let physical = match row {
                0..=2 | 6..=8 => true,
                3 => col <= 5,
                4 => col <= 4,
                5 => col <= 3,
                9 => col >= 1,
                10 => col >= 2,
                11 => col >= 3,
                _ => false,
            };
            assert_eq!(present, physical, "Vial switch mismatch at ({row},{col})");
        }
    }
}
