//! Scenes are named mixes: a list of (sound id, level). Built-in scenes ship
//! in `assets/scenes.json`; the user's own scenes and last session are saved
//! as JSON in the OS config directory so they survive restarts.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Scene {
    pub name: String,
    #[serde(default)]
    pub desc: String,
    pub mix: Vec<(String, f32)>,
}

#[derive(Deserialize)]
struct SceneFile {
    scenes: Vec<Scene>,
}

pub fn builtin() -> Vec<Scene> {
    let file: SceneFile = serde_json::from_str(include_str!("../assets/scenes.json"))
        .expect("assets/scenes.json is valid");
    file.scenes
}

#[derive(Serialize, Deserialize)]
pub struct SavedState {
    #[serde(default = "default_master")]
    pub master: f32,
    #[serde(default)]
    pub scenes: Vec<Scene>,
    #[serde(default)]
    pub last_mix: Vec<(String, f32)>,
    #[serde(default)]
    pub resume_on_start: bool,
}

fn default_master() -> f32 {
    0.8
}

// Hand-written so a first launch (no state file yet) starts at a sensible
// master level; a derived Default would give 0.0 and a silent app.
impl Default for SavedState {
    fn default() -> Self {
        Self {
            master: default_master(),
            scenes: Vec::new(),
            last_mix: Vec::new(),
            resume_on_start: false,
        }
    }
}

pub fn state_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("dm-sounds").join("state.json"))
}

pub fn load_state() -> SavedState {
    state_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_state(state: &SavedState) {
    if let Some(p) = state_path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(state) {
            let _ = std::fs::write(p, json);
        }
    }
}
