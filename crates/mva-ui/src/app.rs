//! [`MvaUiApp`] — the eframe application struct.

use std::sync::{Arc, Mutex};

use mva_core::PlaybackClock;
use mva_core::effect::EngineEffect;
use mva_core::engine::Engine;
use mva_core::state::PlaybackState;
use mva_renderer::{DrawList, Renderer, Viewport};

use crate::painter::TextureCache;
use crate::panels;

pub struct MvaUiApp {
    engine: Arc<Mutex<Engine>>,
    clock: Box<dyn PlaybackClock>,
    renderer: Renderer,
    seek_pos: f64,
    open_path: String,
    texture_cache: TextureCache,
    on_effect: Box<dyn Fn(EngineEffect)>,
    config_warnings: Vec<String>,
}

impl MvaUiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        engine: Arc<Mutex<Engine>>,
        clock: Box<dyn PlaybackClock>,
        renderer: Renderer,
        on_effect: Box<dyn Fn(EngineEffect)>,
        config_warnings: Vec<String>,
    ) -> Self {
        Self {
            engine,
            clock,
            renderer,
            seek_pos: 0.0,
            open_path: String::new(),
            texture_cache: TextureCache::new(),
            on_effect,
            config_warnings,
        }
    }
}

impl eframe::App for MvaUiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let t = self.clock.position_seconds();

        // --- lock engine for state operations -----------------------
        let (snap, _draw_list, all_effects) = {
            let mut engine = self.engine.lock().unwrap();
            engine.update_position(t);
            let snap = engine.snapshot();

            let rect = ui.max_rect();
            let viewport = Viewport {
                width: rect.width(),
                height: rect.height(),
            };

            let draw_list = match &snap.scene {
                Some(scene) => self.renderer.render(scene, &viewport),
                None => DrawList::empty(),
            };

            let mut commands = Vec::new();

            panels::info::show(ui, &snap);
            panels::viewport::show(ui, &mut self.texture_cache, &draw_list);
            panels::controls::show(ui, &snap, &mut self.seek_pos, &mut commands);
            if snap.state == PlaybackState::Playing {
                self.seek_pos = snap.position;
            }
            panels::settings::show(
                ui,
                &mut commands,
                &snap,
                &mut self.open_path,
                &self.config_warnings,
            );

            let mut all_effects = Vec::new();
            for cmd in commands {
                if let Ok(effects) = engine.handle_command(cmd) {
                    all_effects.extend(effects);
                }
            }

            (snap, draw_list, all_effects)
        }; // engine lock dropped

        // --- dispatch effects outside the lock ---------------------
        for effect in all_effects {
            (self.on_effect)(effect);
        }

        if snap.state == PlaybackState::Playing {
            ui.ctx().request_repaint();
        }
    }
}
