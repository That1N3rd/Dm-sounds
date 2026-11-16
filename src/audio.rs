//! Playback engine built on rodio.
//!
//! Design: every looping sound gets its own `Sink` so it can be mixed,
//! faded and stopped independently. Fades are driven from the UI thread each
//! frame (`tick`), which keeps the engine tiny and avoids a separate audio
//! control thread. Sources are decoded straight from the embedded bytes
//! (`Cursor<&'static [u8]>`), so nothing is copied into memory at play time.

use crate::bank::{self, Kind, SoundDef};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::io::Cursor;

/// Seconds to go from silent to full volume when starting or stopping.
const FADE_SECS: f32 = 1.5;

struct Channel {
    sink: Sink,
    /// Volume currently applied to the sink (before master).
    current: f32,
    /// Volume we are fading towards. `0.0` means "stopping".
    target: f32,
    /// The user's slider value for this sound (the level once faded in).
    level: f32,
}

pub struct Engine {
    // Dropping the stream closes the device, so it must outlive the sinks.
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    channels: HashMap<&'static str, Channel>,
    pub master: f32,
    pub error: Option<String>,
}

impl Engine {
    pub fn new() -> Self {
        let (stream, handle, error) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h), None),
            Err(e) => (None, None, Some(format!("No audio output device: {e}"))),
        };
        Self {
            _stream: stream,
            handle,
            channels: HashMap::new(),
            master: 0.8,
            error,
        }
    }

    pub fn is_playing(&self, id: &str) -> bool {
        self.channels
            .get(id)
            .map(|c| c.target > 0.0)
            .unwrap_or(false)
    }

    pub fn level(&self, id: &str) -> Option<f32> {
        self.channels
            .get(id)
            .filter(|c| c.target > 0.0)
            .map(|c| c.level)
    }

    /// Ids of every loop currently sounding, with their slider level.
    pub fn playing(&self) -> Vec<(&'static str, f32)> {
        let mut v: Vec<_> = self
            .channels
            .iter()
            .filter(|(_, c)| c.target > 0.0)
            .map(|(id, c)| (*id, c.level))
            .collect();
        v.sort_by(|a, b| a.0.cmp(b.0));
        v
    }

    pub fn any_playing(&self) -> bool {
        self.channels.values().any(|c| c.target > 0.0)
    }

    /// Start (or re-target) a loop at the given level. Loops that are already
    /// fading out are simply faded back in, which is what makes quick
    /// toggling feel smooth instead of restarting the file.
    pub fn play(&mut self, def: &'static SoundDef, level: f32) {
        let level = level.clamp(0.0, 1.0);
        if def.kind == Kind::OneShot {
            self.fire(def, level);
            return;
        }
        if let Some(ch) = self.channels.get_mut(def.id) {
            ch.level = level;
            ch.target = level;
            return;
        }
        let Some(handle) = &self.handle else { return };
        let Ok(sink) = Sink::try_new(handle) else {
            return;
        };
        match Decoder::new_looped(Cursor::new(def.bytes)) {
            Ok(src) => {
                sink.set_volume(0.0);
                sink.append(src);
            }
            Err(e) => {
                self.error = Some(format!("Could not decode {}: {e}", def.name));
                return;
            }
        }
        self.channels.insert(
            def.id,
            Channel {
                sink,
                current: 0.0,
                target: level,
                level,
            },
        );
    }

    /// Play a one-shot once at the given gain and forget about it.
    fn fire(&mut self, def: &'static SoundDef, level: f32) {
        let Some(handle) = &self.handle else { return };
        let Ok(sink) = Sink::try_new(handle) else {
            return;
        };
        if let Ok(src) = Decoder::new(Cursor::new(def.bytes)) {
            sink.set_volume(level * self.master);
            sink.append(src.convert_samples::<f32>());
            sink.detach();
        }
    }

    pub fn set_level(&mut self, id: &str, level: f32) {
        if let Some(ch) = self.channels.get_mut(id) {
            ch.level = level.clamp(0.0, 1.0);
            if ch.target > 0.0 {
                ch.target = ch.level;
            }
        }
    }

    pub fn stop(&mut self, id: &str) {
        if let Some(ch) = self.channels.get_mut(id) {
            ch.target = 0.0;
        }
    }

    pub fn toggle(&mut self, def: &'static SoundDef, default_level: f32) {
        if self.is_playing(def.id) {
            self.stop(def.id);
        } else {
            let level = self
                .channels
                .get(def.id)
                .map(|c| c.level)
                .unwrap_or(default_level);
            self.play(def, level);
        }
    }

    pub fn stop_all(&mut self) {
        for ch in self.channels.values_mut() {
            ch.target = 0.0;
        }
    }

    /// Crossfade to a whole mix: everything not in `mix` fades out, everything
    /// in it fades to its level.
    pub fn apply_mix(&mut self, mix: &[(String, f32)]) {
        self.stop_all();
        for (id, level) in mix {
            if let Some(def) = bank::find(id) {
                self.play(def, *level);
            }
        }
    }

    /// Advance fades. Returns true while any channel is still moving, so the
    /// UI knows to keep repainting.
    pub fn tick(&mut self, dt: f32) -> bool {
        let step = dt / FADE_SECS;
        let master = self.master;
        let mut busy = false;
        self.channels.retain(|_, ch| {
            if (ch.current - ch.target).abs() > 0.0005 {
                busy = true;
                if ch.current < ch.target {
                    ch.current = (ch.current + step).min(ch.target);
                } else {
                    ch.current = (ch.current - step).max(ch.target);
                }
            } else {
                ch.current = ch.target;
            }
            // Perceptual curve: sliders feel linear when gain is squared.
            ch.sink.set_volume(ch.current * ch.current * master);
            if ch.target <= 0.0 && ch.current <= 0.0 {
                ch.sink.stop();
                false
            } else {
                true
            }
        });
        busy
    }
}
