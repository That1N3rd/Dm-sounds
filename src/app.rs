//! The egui user interface. Layout mirrors the Character Forge: a header bar
//! with the title and the primary action, a row of tabs with completion
//! marks, a big parchment card holding a heading, a lead paragraph and a grid
//! of smaller cards. The bottom dock is the "now playing" tray.

use crate::audio::Engine;
use crate::bank::{self, Kind, SoundDef};
use crate::scenes::{self, SavedState, Scene};
use crate::theme::{
    self, ACCENT, ACCENT_HOVER, ACCENT_TEXT, BORDER, CARD, GREEN, MUTED, TEXT, TEXT_SOFT,
};
use egui::{Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, Sense, Stroke, Vec2};
use std::time::{Duration, Instant};

#[derive(Clone, PartialEq, Eq)]
enum Tab {
    Scenes,
    Category(usize),
    Credits,
}

pub struct App {
    engine: Engine,
    tab: Tab,
    builtin: Vec<Scene>,
    state: SavedState,
    active_scene: Option<String>,
    new_scene_name: String,
    search: String,
    menu_open: bool,
    pending_delete: Option<usize>,
    dirty: bool,
    last_save: Instant,
}

const DEFAULT_LEVEL: f32 = 0.7;
const CARD_H: f32 = 182.0;
const SCENE_H: f32 = 196.0;

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let state = scenes::load_state();
        let mut engine = Engine::new();
        engine.master = state.master.clamp(0.0, 1.0);
        if state.resume_on_start && !state.last_mix.is_empty() {
            engine.apply_mix(&state.last_mix);
        }
        Self {
            engine,
            tab: Tab::Scenes,
            builtin: scenes::builtin(),
            state,
            active_scene: None,
            new_scene_name: String::new(),
            search: String::new(),
            menu_open: false,
            pending_delete: None,
            dirty: false,
            last_save: Instant::now(),
        }
    }

    fn touch(&mut self) {
        self.state.last_mix = self
            .engine
            .playing()
            .into_iter()
            .map(|(id, l)| (id.to_owned(), l))
            .collect();
        self.state.master = self.engine.master;
        self.dirty = true;
    }

    fn persist_if_needed(&mut self, force: bool) {
        if self.dirty && (force || self.last_save.elapsed() > Duration::from_secs(2)) {
            scenes::save_state(&self.state);
            self.dirty = false;
            self.last_save = Instant::now();
        }
    }

    fn play_scene(&mut self, scene: &Scene) {
        self.engine.apply_mix(&scene.mix);
        self.active_scene = Some(scene.name.clone());
        self.touch();
    }

    // ----- building blocks -------------------------------------------------

    fn accent_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
        let b = egui::Button::new(RichText::new(text).font(theme::ui(15.0)).color(TEXT))
            .fill(if enabled {
                ACCENT
            } else {
                ACCENT.gamma_multiply(0.45)
            })
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(0.0, 34.0));
        let r = ui.add_enabled(enabled, b);
        if r.hovered() && enabled {
            ui.painter().rect_filled(
                r.rect,
                CornerRadius::same(8),
                ACCENT_HOVER.gamma_multiply(0.25),
            );
        }
        r
    }

    fn ghost_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
        ui.add(
            egui::Button::new(RichText::new(text).font(theme::ui(14.0)).color(TEXT_SOFT))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0_f32, BORDER))
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(0.0, 32.0)),
        )
    }

    fn check_circle(ui: &mut egui::Ui, on: bool) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
        let c = rect.center();
        if on {
            ui.painter().circle_filled(c, 9.0, GREEN);
            // Painted rather than a glyph: the UI font has no check mark.
            let ink = Stroke::new(2.0_f32, Color32::from_rgb(0x12, 0x2a, 0x18));
            ui.painter()
                .line_segment([c + egui::vec2(-4.0, 0.0), c + egui::vec2(-1.0, 3.0)], ink);
            ui.painter()
                .line_segment([c + egui::vec2(-1.0, 3.0), c + egui::vec2(4.5, -3.5)], ink);
        } else {
            ui.painter()
                .circle_stroke(c, 8.5, Stroke::new(1.5_f32, BORDER));
        }
    }

    // ----- top chrome --------------------------------------------------------

    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header")
            .frame(
                Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(Margin::symmetric(18, 12)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚔").font(theme::ui(24.0)).color(TEXT_SOFT));
                    ui.label(
                        RichText::new("DM Sounds")
                            .font(theme::title(27.0))
                            .color(ACCENT_TEXT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if Self::ghost_button(ui, "☰").clicked() {
                            self.menu_open = !self.menu_open;
                        }
                        let playing = self.engine.any_playing();
                        if Self::accent_button(ui, "■ Stop all", playing).clicked() {
                            self.engine.stop_all();
                            self.active_scene = None;
                            self.touch();
                        }
                        ui.add_space(6.0);
                        let mut master = self.engine.master;
                        let s = ui.add_sized(
                            [150.0, 24.0],
                            egui::Slider::new(&mut master, 0.0..=1.0).show_value(false),
                        );
                        if s.changed() {
                            self.engine.master = master;
                            self.touch();
                        }
                        ui.label(
                            RichText::new("Master")
                                .font(theme::ui(15.0))
                                .color(TEXT_SOFT),
                        );
                    });
                });
                if self.menu_open {
                    ui.add_space(6.0);
                    Frame::new()
                        .fill(CARD)
                        .stroke(Stroke::new(1.0_f32, BORDER))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(12))
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                if ui
                                    .checkbox(
                                        &mut self.state.resume_on_start,
                                        "Resume last mix on launch",
                                    )
                                    .changed()
                                {
                                    self.dirty = true;
                                }
                                ui.add_space(12.0);
                                if Self::ghost_button(ui, "Credits & licences").clicked() {
                                    self.tab = Tab::Credits;
                                    self.menu_open = false;
                                }
                                if let Some(p) = scenes::state_path() {
                                    ui.add_space(12.0);
                                    ui.label(
                                        RichText::new(format!("Saved to {}", p.display()))
                                            .font(theme::ui(12.0))
                                            .color(MUTED),
                                    );
                                }
                            });
                        });
                }
                if let Some(err) = self.engine.error.clone() {
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(err)
                            .font(theme::body(15.0))
                            .color(ACCENT_TEXT),
                    );
                }
            });
    }

    fn tabs(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabs")
            .frame(
                Frame::new()
                    .fill(theme::BG)
                    .stroke(Stroke::new(1.0_f32, theme::BORDER_SOFT))
                    .inner_margin(Margin {
                        left: 10,
                        right: 10,
                        top: 6,
                        bottom: 0,
                    }),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
                    let mut items: Vec<(Tab, String, Option<bool>)> = vec![(
                        Tab::Scenes,
                        "Scenes".into(),
                        Some(self.active_scene.is_some()),
                    )];
                    for (i, c) in bank::CATEGORIES.iter().enumerate() {
                        let on = bank::in_category(c.name).any(|s| self.engine.is_playing(s.id));
                        items.push((Tab::Category(i), c.name.to_owned(), Some(on)));
                    }
                    items.push((Tab::Credits, "Credits".into(), None));
                    for (tab, label, mark) in items {
                        // Pre-measure so the row breaks before a tab that
                        // would not fit: a framed widget cannot wrap itself.
                        let text_w = ui.fonts(|f| {
                            f.layout_no_wrap(label.clone(), theme::ui(15.5), TEXT)
                                .size()
                                .x
                        });
                        let tab_w = text_w + 18.0 + if mark.is_some() { 24.0 } else { 0.0 };
                        if ui.available_size_before_wrap().x < tab_w {
                            ui.end_row();
                        }
                        let selected = self.tab == tab;
                        let text_col = if selected { TEXT } else { TEXT_SOFT };
                        let resp = ui
                            .scope(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                Frame::new()
                                    .inner_margin(Margin::symmetric(9, 10))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if let Some(on) = mark {
                                                Self::check_circle(ui, on);
                                            }
                                            ui.label(
                                                RichText::new(label)
                                                    .font(theme::ui(15.5))
                                                    .color(text_col),
                                            );
                                        });
                                    })
                            })
                            .response;
                        let resp = ui.interact(resp.rect, resp.id.with("tab"), Sense::click());
                        if resp.hovered() {
                            ui.painter().rect_filled(
                                resp.rect,
                                CornerRadius::same(6),
                                Color32::from_white_alpha(8),
                            );
                        }
                        if selected {
                            let r = resp.rect;
                            ui.painter().rect_filled(
                                egui::Rect::from_min_max(
                                    egui::pos2(r.left(), r.bottom() - 3.0),
                                    egui::pos2(r.right(), r.bottom()),
                                ),
                                CornerRadius {
                                    nw: 2,
                                    ne: 2,
                                    sw: 0,
                                    se: 0,
                                },
                                ACCENT,
                            );
                        }
                        if resp.clicked() {
                            self.tab = tab;
                            self.pending_delete = None;
                        }
                    }
                });
            });
    }

    // ----- bottom dock -------------------------------------------------------

    fn dock(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("dock")
            .frame(
                Frame::new()
                    .fill(theme::PANEL)
                    .stroke(Stroke::new(1.0_f32, theme::BORDER_SOFT))
                    .inner_margin(Margin::symmetric(18, 12)),
            )
            .show(ctx, |ui| {
                let playing = self.engine.playing();
                if playing.is_empty() {
                    ui.label(
                        RichText::new(
                            "Nothing playing. Pick a scene, or layer sounds from the tabs above.",
                        )
                        .font(theme::italic(16.0))
                        .color(MUTED),
                    );
                    return;
                }
                let title = match &self.active_scene {
                    Some(s) => format!("Now playing · {s}"),
                    None => "Now playing".to_owned(),
                };
                ui.label(
                    RichText::new(title)
                        .font(theme::title(15.0))
                        .color(ACCENT_TEXT),
                );
                ui.add_space(2.0);

                // Chips are laid out in fixed-width rows by hand: a wrapped
                // horizontal layout can't predict a Frame's width before it
                // is drawn, so it would overflow the panel instead of wrapping.
                let defs: Vec<(&'static SoundDef, f32)> = playing
                    .iter()
                    .filter_map(|(id, l)| bank::find(id).map(|d| (d, *l)))
                    .collect();
                let chip_w = 270.0;
                let gap = 8.0;
                let cols = ((ui.available_width() + gap) / (chip_w + gap))
                    .floor()
                    .max(1.0) as usize;
                for row in defs.chunks(cols) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for &(def, level) in row {
                            ui.allocate_ui_with_layout(
                                Vec2::new(chip_w, 32.0),
                                Layout::left_to_right(Align::Center),
                                |ui| self.chip(ui, def, level, chip_w),
                            );
                        }
                    });
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let edit = egui::TextEdit::singleline(&mut self.new_scene_name)
                        .hint_text("Name this mix…")
                        .font(theme::body(16.0))
                        .desired_width(220.0);
                    let r = ui.add(edit);
                    let submit = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let name = self.new_scene_name.trim().to_owned();
                    if (Self::accent_button(ui, "Save as scene", !name.is_empty()).clicked()
                        || submit)
                        && !name.is_empty()
                    {
                        let mix: Vec<(String, f32)> = playing
                            .iter()
                            .map(|(id, l)| ((*id).to_owned(), *l))
                            .collect();
                        if let Some(existing) =
                            self.state.scenes.iter_mut().find(|s| s.name == name)
                        {
                            existing.mix = mix;
                        } else {
                            self.state.scenes.push(Scene {
                                name: name.clone(),
                                desc: "Your own mix.".into(),
                                mix,
                            });
                        }
                        self.active_scene = Some(name);
                        self.new_scene_name.clear();
                        self.dirty = true;
                    }
                });
            });
    }

    /// One "now playing" chip: name, mini level slider and a remove button.
    fn chip(&mut self, ui: &mut egui::Ui, def: &'static SoundDef, level: f32, chip_w: f32) {
        Frame::new()
            .fill(CARD)
            .stroke(Stroke::new(1.0_f32, BORDER))
            .corner_radius(CornerRadius::same(16))
            .inner_margin(Margin::symmetric(10, 4))
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(chip_w - 20.0, 24.0));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.add_sized(
                        [118.0, 20.0],
                        egui::Label::new(RichText::new(def.name).font(theme::ui(13.5)).color(TEXT))
                            .truncate(),
                    );
                    let mut l = level;
                    let s = ui.add_sized(
                        [80.0, 16.0],
                        egui::Slider::new(&mut l, 0.0..=1.0).show_value(false),
                    );
                    let x = ui.add(
                        egui::Button::new(RichText::new("×").font(theme::ui(16.0)).color(MUTED))
                            .frame(false),
                    );
                    if s.changed() {
                        self.engine.set_level(def.id, l);
                        self.active_scene = None;
                        self.touch();
                    }
                    if x.clicked() {
                        self.engine.stop(def.id);
                        self.active_scene = None;
                        self.touch();
                    }
                });
            });
    }

    fn page<R>(
        ui: &mut egui::Ui,
        heading: &str,
        lead: &str,
        body: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        Frame::new()
            .fill(CARD)
            .stroke(Stroke::new(1.0_f32, BORDER))
            .corner_radius(CornerRadius::same(14))
            .inner_margin(Margin::same(22))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(heading)
                        .font(theme::heading(30.0))
                        .color(TEXT),
                );
                ui.add_space(4.0);
                ui.label(RichText::new(lead).font(theme::body(18.5)).color(TEXT_SOFT));
                ui.add_space(14.0);
                body(ui)
            })
            .inner
    }

    /// Lay `items` out in equal-width columns and call `draw` for each inside
    /// a bordered card. Width-driven, so the grid reflows with the window.
    fn grid<T: Copy>(
        ui: &mut egui::Ui,
        items: &[T],
        min_w: f32,
        h: f32,
        mut draw: impl FnMut(&mut egui::Ui, T),
    ) {
        let gap = 12.0;
        let avail = ui.available_width();
        let cols = ((avail + gap) / (min_w + gap)).floor().max(1.0) as usize;
        let w = ((avail - gap * (cols as f32 - 1.0)) / cols as f32).floor();
        for row in items.chunks(cols) {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for &it in row {
                    // A fixed-size vertical child: the row is horizontal, so
                    // without this the card's own content would flow sideways,
                    // and the fixed height is what the bottom-aligned button
                    // row anchors to.
                    ui.allocate_ui_with_layout(
                        Vec2::new(w, h),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.set_min_size(Vec2::new(w, h));
                            draw(ui, it);
                        },
                    );
                }
            });
            ui.add_space(gap);
        }
    }

    fn sound_card(&mut self, ui: &mut egui::Ui, def: &'static SoundDef) {
        let playing = self.engine.is_playing(def.id);
        let stroke = if playing {
            Stroke::new(1.5_f32, ACCENT)
        } else {
            Stroke::new(1.0_f32, BORDER)
        };
        Frame::new()
            .fill(if playing {
                Color32::from_rgb(0x2a, 0x22, 0x1e)
            } else {
                theme::PANEL
            })
            .stroke(stroke)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(CARD_H - 28.0);
                ui.label(
                    RichText::new(def.name)
                        .font(theme::heading(16.5))
                        .color(TEXT),
                );
                let kind = match def.kind {
                    Kind::Loop => "LOOP",
                    Kind::OneShot => "ONE-SHOT",
                };
                ui.add(
                    egui::Label::new(
                        RichText::new(format!(
                            "{kind} · {} · {}",
                            def.license.to_uppercase(),
                            def.author.to_uppercase()
                        ))
                        .font(theme::ui(11.0))
                        .color(MUTED),
                    )
                    .truncate(),
                );
                if !def.desc.is_empty() {
                    ui.add_space(2.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(def.desc)
                                .font(theme::body(15.5))
                                .color(TEXT_SOFT),
                        )
                        .wrap(),
                    );
                }
                ui.add_space(8.0);
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.horizontal(|ui| match def.kind {
                        Kind::Loop => {
                            let label = if playing { "■ Stop" } else { "▶ Play" };
                            if Self::accent_button(ui, label, true).clicked() {
                                self.engine.toggle(def, DEFAULT_LEVEL);
                                self.active_scene = None;
                                self.touch();
                            }
                            let mut level = self.engine.level(def.id).unwrap_or(DEFAULT_LEVEL);
                            let w = (ui.available_width() - 4.0).max(60.0);
                            let s = ui.add_sized(
                                [w, 24.0],
                                egui::Slider::new(&mut level, 0.0..=1.0).show_value(false),
                            );
                            if s.changed() {
                                if playing {
                                    self.engine.set_level(def.id, level);
                                } else {
                                    self.engine.play(def, level);
                                }
                                self.active_scene = None;
                                self.touch();
                            }
                        }
                        Kind::OneShot => {
                            if Self::accent_button(ui, "▶ Fire", true).clicked() {
                                self.engine.play(def, 1.0);
                            }
                            ui.label(
                                RichText::new("plays once")
                                    .font(theme::italic(14.0))
                                    .color(MUTED),
                            );
                        }
                    });
                });
            });
    }

    fn scene_card(&mut self, ui: &mut egui::Ui, scene_ref: (bool, usize)) {
        let (custom, idx) = scene_ref;
        let scene = if custom {
            self.state.scenes[idx].clone()
        } else {
            self.builtin[idx].clone()
        };
        let active = self.active_scene.as_deref() == Some(scene.name.as_str());
        let stroke = if active {
            Stroke::new(1.5_f32, ACCENT)
        } else {
            Stroke::new(1.0_f32, BORDER)
        };
        Frame::new()
            .fill(if active {
                Color32::from_rgb(0x2a, 0x22, 0x1e)
            } else {
                theme::PANEL
            })
            .stroke(stroke)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(SCENE_H - 28.0);
                ui.label(
                    RichText::new(&scene.name)
                        .font(theme::heading(17.0))
                        .color(TEXT),
                );
                let names: Vec<&str> = scene
                    .mix
                    .iter()
                    .filter_map(|(id, _)| bank::find(id).map(|d| d.name))
                    .collect();
                ui.add(
                    egui::Label::new(
                        RichText::new(names.join(" · ").to_uppercase())
                            .font(theme::ui(11.0))
                            .color(MUTED),
                    )
                    .truncate(),
                );
                if !scene.desc.is_empty() {
                    ui.add_space(2.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(&scene.desc)
                                .font(theme::body(15.5))
                                .color(TEXT_SOFT),
                        )
                        .wrap(),
                    );
                }
                ui.add_space(8.0);
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.horizontal(|ui| {
                        let label = if active { "■ Stop" } else { "▶ Play scene" };
                        if Self::accent_button(ui, label, true).clicked() {
                            if active {
                                self.engine.stop_all();
                                self.active_scene = None;
                                self.touch();
                            } else {
                                self.play_scene(&scene);
                            }
                        }
                        if custom {
                            if self.pending_delete == Some(idx) {
                                if Self::ghost_button(ui, "Really delete").clicked() {
                                    self.state.scenes.remove(idx);
                                    self.pending_delete = None;
                                    self.dirty = true;
                                }
                            } else if Self::ghost_button(ui, "Delete").clicked() {
                                self.pending_delete = Some(idx);
                            }
                        }
                    });
                });
            });
    }

    fn scenes_tab(&mut self, ui: &mut egui::Ui) {
        Self::page(
            ui,
            "Choose a Scene",
            "A scene is a ready-made mix of loops. Playing one crossfades out whatever is sounding now. \
             Tweak levels in the tray below, then save the result as a scene of your own.",
            |ui| {
                let items: Vec<(bool, usize)> = (0..self.builtin.len()).map(|i| (false, i)).collect();
                Self::grid(ui, &items, 290.0, SCENE_H, |ui, it| self.scene_card(ui, it));
                if !self.state.scenes.is_empty() {
                    ui.add_space(6.0);
                    ui.label(RichText::new("Your Scenes").font(theme::heading(22.0)).color(TEXT));
                    ui.add_space(10.0);
                    let items: Vec<(bool, usize)> = (0..self.state.scenes.len()).map(|i| (true, i)).collect();
                    Self::grid(ui, &items, 290.0, SCENE_H, |ui, it| self.scene_card(ui, it));
                }
            },
        );
    }

    fn category_tab(&mut self, ui: &mut egui::Ui, idx: usize) {
        let cat = &bank::CATEGORIES[idx];
        Self::page(ui, cat.name, cat.desc, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text("Filter sounds…")
                        .font(theme::body(16.0))
                        .desired_width(240.0),
                );
                if !self.search.is_empty() && Self::ghost_button(ui, "Clear").clicked() {
                    self.search.clear();
                }
            });
            ui.add_space(12.0);
            let q = self.search.to_lowercase();
            let items: Vec<&'static SoundDef> = bank::in_category(cat.name)
                .filter(|s| {
                    q.is_empty()
                        || s.name.to_lowercase().contains(&q)
                        || s.desc.to_lowercase().contains(&q)
                })
                .collect();
            if items.is_empty() {
                ui.label(
                    RichText::new("No sounds match.")
                        .font(theme::italic(16.0))
                        .color(MUTED),
                );
            }
            Self::grid(ui, &items, 290.0, CARD_H, |ui, def| {
                self.sound_card(ui, def)
            });
        });
    }

    fn credits_tab(&mut self, ui: &mut egui::Ui) {
        Self::page(
            ui,
            "Credits & Licences",
            "Every sound here is free and openly licensed. CC0 and public-domain recordings need no credit; \
             the CC-BY ones are credited below as their licence asks. Click a title to visit its source page.",
            |ui| {
                for cat in bank::CATEGORIES {
                    ui.label(RichText::new(cat.name).font(theme::heading(18.0)).color(ACCENT_TEXT));
                    ui.add_space(4.0);
                    for s in bank::in_category(cat.name) {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            ui.hyperlink_to(RichText::new(s.name).font(theme::body(17.0)).color(TEXT), s.source);
                            ui.label(
                                RichText::new(format!("— {} · {}", s.author, s.license))
                                    .font(theme::body(16.0))
                                    .color(MUTED),
                            );
                        });
                    }
                    ui.add_space(12.0);
                }
                ui.label(RichText::new("Fonts").font(theme::heading(18.0)).color(ACCENT_TEXT));
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Cinzel by Natanael Gama and EB Garamond by Georg Duffner & Octavio Pardo, both under the SIL Open Font License.")
                        .font(theme::body(16.0))
                        .color(TEXT_SOFT),
                );
                ui.add_space(12.0);
                ui.label(RichText::new("App").font(theme::heading(18.0)).color(ACCENT_TEXT));
                ui.add_space(4.0);
                ui.label(
                    RichText::new("DM Sounds is written in Rust with egui and rodio. Esc stops everything.")
                        .font(theme::body(16.0))
                        .color(TEXT_SOFT),
                );
            },
        );
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        if self.engine.tick(dt) {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.engine.any_playing() {
            self.engine.stop_all();
            self.active_scene = None;
            self.touch();
        }

        self.header(ctx);
        self.tabs(ctx);
        self.dock(ctx);

        egui::CentralPanel::default()
            .frame(Frame::new().fill(theme::BG).inner_margin(Margin::same(16)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match self.tab.clone() {
                            Tab::Scenes => self.scenes_tab(ui),
                            Tab::Category(i) => self.category_tab(ui, i),
                            Tab::Credits => self.credits_tab(ui),
                        }
                        ui.add_space(8.0);
                    });
            });

        self.persist_if_needed(false);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.touch();
        self.persist_if_needed(true);
    }
}
