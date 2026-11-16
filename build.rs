//! Generates `bank.rs` from `assets/sounds/manifest.json`.
//!
//! Why a build script: the whole sound bank is embedded into the executable
//! with `include_bytes!`, so the app is a single file on Windows with nothing
//! to install or locate at runtime. The manifest stays human-editable JSON,
//! and this script turns it into static Rust data at compile time.
use serde::Deserialize;
use std::{env, fs, path::Path};

#[derive(Deserialize)]
struct Manifest {
    categories: Vec<Category>,
    sounds: Vec<Sound>,
}

#[derive(Deserialize)]
struct Category {
    name: String,
    desc: String,
}

#[derive(Deserialize)]
struct Sound {
    id: String,
    name: String,
    category: String,
    kind: String,
    file: String,
    author: String,
    license: String,
    source: String,
    #[serde(default)]
    desc: String,
}

fn lit(s: &str) -> String {
    format!("{:?}", s)
}

fn main() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let sounds_dir = Path::new(&root).join("assets").join("sounds");
    let manifest_path = sounds_dir.join("manifest.json");
    println!("cargo:rerun-if-changed={}", manifest_path.display());
    println!("cargo:rerun-if-changed={}", sounds_dir.display());

    let text = fs::read_to_string(&manifest_path).expect("read assets/sounds/manifest.json");
    let m: Manifest = serde_json::from_str(&text).expect("parse manifest.json");

    let mut out = String::new();
    out.push_str("pub static CATEGORIES: &[Category] = &[\n");
    for c in &m.categories {
        out.push_str(&format!(
            "    Category {{ name: {}, desc: {} }},\n",
            lit(&c.name),
            lit(&c.desc)
        ));
    }
    out.push_str("];\n\n");

    out.push_str("pub static SOUNDS: &[SoundDef] = &[\n");
    for s in &m.sounds {
        let path = sounds_dir.join(&s.file);
        assert!(
            path.exists(),
            "manifest references missing file {}",
            path.display()
        );
        assert!(
            m.categories.iter().any(|c| c.name == s.category),
            "sound {} uses unknown category {}",
            s.id,
            s.category
        );
        let kind = match s.kind.as_str() {
            "loop" => "Kind::Loop",
            "oneshot" => "Kind::OneShot",
            other => panic!("sound {} has unknown kind {other}", s.id),
        };
        out.push_str(&format!(
            "    SoundDef {{ id: {}, name: {}, category: {}, kind: {}, bytes: include_bytes!({}), author: {}, license: {}, source: {}, desc: {} }},\n",
            lit(&s.id), lit(&s.name), lit(&s.category), kind,
            lit(&path.to_string_lossy()),
            lit(&s.author), lit(&s.license), lit(&s.source), lit(&s.desc)
        ));
    }
    out.push_str("];\n");

    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("bank.rs");
    fs::write(dest, out).unwrap();
}
