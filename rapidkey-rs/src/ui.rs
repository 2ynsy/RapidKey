use crate::models::{AppState, Mode};
use crate::theme;
use crate::utils::map_egui_to_rdev;
use crossbeam_channel::Sender;
use eframe::egui;
use std::sync::atomic::Ordering;
use std::time::Duration;

pub struct RapidKeyUI {
    pub state: AppState,
    pub tx_start: Sender<()>,
    pub tx_stop: Sender<()>,
}

impl RapidKeyUI {
    pub fn new(state: AppState, tx_start: Sender<()>, tx_stop: Sender<()>) -> Self {
        Self { state, tx_start, tx_stop }
    }

    fn render_header(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(egui::RichText::new("⚡ RAPIDKEY PRO").size(32.0).strong().color(theme::TEXT_TITLE));
            ui.label(egui::RichText::new("PRECISION PERFORMANCE ENGINE").size(11.0).color(theme::TEXT_SUB).extra_letter_spacing(1.5));
        });
        ui.add_space(theme::HEADER_BOTTOM);
    }

    fn render_target_key_section(&self, ui: &mut egui::Ui, capturing: bool, key_name: &str) {
        render_section(ui, "TARGET KEY", |ui| {
            let btn_text = if capturing { "CAPTURING..." } else { key_name };
            let btn_fill = if capturing { theme::PRIMARY } else { theme::BG_SURFACE };
            
            let key_btn = egui::Button::new(egui::RichText::new(btn_text).size(22.0).strong().color(egui::Color32::WHITE))
                .fill(btn_fill)
                .min_size(egui::vec2(360.0, 60.0));
                
            if ui.add(key_btn).clicked() {
                self.state.capturing_key.store(true, Ordering::SeqCst);
            }
        });
    }

    fn render_configuration_section(&self, ui: &mut egui::Ui, cps: u32, mode: Mode, limit: u32) {
        render_section(ui, "CONFIGURATION", |ui| {
            egui::Grid::new("config_grid")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    // Speed
                    ui.label("Speed (CPS):");
                    let mut val = cps;
                    let slider = egui::Slider::new(&mut val, 1..=120).show_value(true).trailing_fill(true);
                    if ui.add_sized([220.0, 20.0], slider).changed() {
                        *self.state.cps.lock().unwrap() = val;
                    }
                    ui.end_row();

                    // Mode
                    ui.label("Action Mode:");
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let mut m = mode;
                        if ui.radio_value(&mut m, Mode::Toggle, "Toggle").clicked() ||
                           ui.radio_value(&mut m, Mode::Hold, "Hold").clicked() ||
                           ui.radio_value(&mut m, Mode::Count, "Burst").clicked() {
                            *self.state.mode.lock().unwrap() = m;
                        }
                    });
                    ui.end_row();

                    // Burst Limit
                    let opacity = if mode == Mode::Count { 1.0 } else { 0.2 };
                    ui.label(egui::RichText::new("Burst Limit:").color(theme::TEXT_TITLE.gamma_multiply(opacity)));
                    
                    ui.add_enabled_ui(mode == Mode::Count, |ui| {
                        let mut l = limit;
                        if ui.add(egui::DragValue::new(&mut l).range(1..=1000000)).changed() {
                            *self.state.repeat_count.lock().unwrap() = l;
                        }
                    });
                    ui.end_row();
                });
        });
    }

    fn render_master_controls(&self, ui: &mut egui::Ui, is_running: bool) {
        let main_text = if is_running { "⬛ TERMINATE ENGINE" } else { "▶ INITIALIZE ENGINE" };
        let main_color = if is_running { theme::DANGER } else { theme::PRIMARY };
        
        if ui.add(egui::Button::new(egui::RichText::new(main_text).size(20.0).strong().color(egui::Color32::WHITE))
            .fill(main_color)
            .min_size(egui::vec2(380.0, 64.0))).clicked() {
            self.state.toggle();
            // Still sync with channels for reliability
            if self.state.is_running() {
                let _ = self.tx_start.send(());
            } else {
                let _ = self.tx_stop.send(());
            }
        }
    }
}

impl eframe::App for RapidKeyUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_running = self.state.is_running();
        let capturing = self.state.capturing_key.load(Ordering::SeqCst);
        let status_msg = self.state.debug_msg.lock().unwrap().clone();

        if capturing {
            ctx.input(|i| {
                for e in &i.events {
                    if let egui::Event::Key { key, pressed: true, .. } = e {
                        if *key != egui::Key::F8 && *key != egui::Key::Escape {
                            let rk = map_egui_to_rdev(*key);
                            self.state.set_target_key(rk);
                        }
                    }
                }
            });
        }

        let key_name = self.state.target_key_name.lock().unwrap().clone();
        let cps = *self.state.cps.lock().unwrap();
        let mode = *self.state.mode.lock().unwrap();
        let limit = *self.state.repeat_count.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0; 

            ui.add_space(theme::PADDING_PAGE);
            
            ui.vertical_centered(|ui| {
                self.render_header(ui);

                self.render_target_key_section(ui, capturing, &key_name);
                ui.add_space(theme::GAP_SECTION);

                self.render_configuration_section(ui, cps, mode, limit);
                ui.add_space(theme::GAP_SECTION * 1.5);

                self.render_master_controls(ui, is_running);
                ui.add_space(theme::GAP_SECTION * 1.5);

                render_stats(ui, &self.state);
                
                ui.add_space(theme::PADDING_PAGE);
                ui.label(egui::RichText::new(format!("STATUS: {} | F8 TO TOGGLE", status_msg.to_uppercase()))
                    .size(10.0)
                    .color(theme::TEXT_SUB)
                    .extra_letter_spacing(1.0));
            });
            ctx.request_repaint_after(Duration::from_millis(16));
        });
    }
}

fn render_section<R>(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui) -> R) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(title).size(10.0).strong().color(theme::PRIMARY_LIGHT).extra_letter_spacing(1.0));
        });
        ui.add_space(4.0);
        let frame = egui::Frame::NONE
            .fill(theme::BG_MAIN)
            .inner_margin(theme::CARD_PADDING)
            .stroke(egui::Stroke::new(1.0, theme::BG_SURFACE))
            .corner_radius(egui::CornerRadius::same(12));
        
        frame.show(ui, |ui| {
            ui.set_width(380.0);
            add_contents(ui);
        });
    });
}

fn render_stats(ui: &mut egui::Ui, state: &AppState) {
    let total = state.total_presses.load(Ordering::Relaxed);
    let cps = state.measured_cps.load(Ordering::Relaxed);
    let uptime = state.elapsed_time.lock().unwrap().as_secs();
    let current_session = if let Some(start) = *state.start_time.lock().unwrap() { start.elapsed().as_secs() } else { 0 };

    egui::Frame::NONE
        .fill(theme::BG_SURFACE)
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(380.0);
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.columns(3, |cols| {
                cols[0].vertical_centered(|ui| {
                    ui.label(egui::RichText::new(format!("{}", total)).size(24.0).strong().color(egui::Color32::WHITE));
                    ui.label(egui::RichText::new("TOTAL").size(8.0).color(theme::TEXT_SUB));
                });
                cols[1].vertical_centered(|ui| {
                    let color = if cps > 0 { theme::SUCCESS } else { theme::TEXT_SUB };
                    ui.label(egui::RichText::new(format!("{}", cps)).size(24.0).strong().color(color));
                    ui.label(egui::RichText::new("REAL CPS").size(8.0).color(theme::TEXT_SUB));
                });
                cols[2].vertical_centered(|ui| {
                    ui.label(egui::RichText::new(format!("{}s", uptime + current_session)).size(24.0).strong().color(egui::Color32::WHITE));
                    ui.label(egui::RichText::new("UPTIME").size(8.0).color(theme::TEXT_SUB));
                });
            });
        });
}
