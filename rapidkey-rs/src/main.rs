#![windows_subsystem = "windows"]

use eframe::egui;
use enigo::{Enigo, Key, Keyboard, Settings, Direction};
use rdev::{listen, EventType, Key as RdevKey};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};
use crossbeam_channel::{unbounded, Sender};

// --- 極限まで緊密化したデザインシステム・トークン ---
pub mod theme {
    use eframe::egui::Color32;

    // Standard spacing units
    pub const PADDING_PAGE: f32 = 12.0;    
    pub const GAP_SECTION: f32 = 8.0;     
    pub const CARD_PADDING: f32 = 8.0;    
    pub const HEADER_BOTTOM: f32 = 16.0;   

    // Colors
    pub const BG_MAIN: Color32 = Color32::from_rgb(15, 23, 42);     
    pub const BG_SURFACE: Color32 = Color32::from_rgb(30, 41, 59);  
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(45, 55, 75);

    pub const PRIMARY: Color32 = Color32::from_rgb(124, 58, 237);   
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(167, 139, 250);
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);    
    pub const DANGER: Color32 = Color32::from_rgb(239, 68, 68);     

    pub const TEXT_TITLE: Color32 = Color32::from_rgb(248, 250, 252);
    pub const TEXT_SUB: Color32 = Color32::from_rgb(148, 163, 184); 
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Mode {
    Toggle,
    Hold,
    Count,
}

#[derive(Clone)]
struct AppState {
    target_key: Arc<Mutex<Option<RdevKey>>>,
    target_key_name: Arc<Mutex<String>>,
    cps: Arc<Mutex<u32>>,
    mode: Arc<Mutex<Mode>>,
    repeat_count: Arc<Mutex<u32>>,
    is_running: Arc<AtomicBool>,
    total_presses: Arc<AtomicU64>,
    measured_cps: Arc<AtomicU64>,
    elapsed_time: Arc<Mutex<Duration>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    capturing_key: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            target_key: Arc::new(Mutex::new(None)),
            target_key_name: Arc::new(Mutex::new("Unassigned".to_string())),
            cps: Arc::new(Mutex::new(10)),
            mode: Arc::new(Mutex::new(Mode::Toggle)),
            repeat_count: Arc::new(Mutex::new(100)),
            is_running: Arc::new(AtomicBool::new(false)),
            total_presses: Arc::new(AtomicU64::new(0)),
            measured_cps: Arc::new(AtomicU64::new(0)),
            elapsed_time: Arc::new(Mutex::new(Duration::ZERO)),
            start_time: Arc::new(Mutex::new(None)),
            capturing_key: Arc::new(AtomicBool::new(false)),
        }
    }
}

fn main() -> eframe::Result {
    let app_state = AppState::default();
    let (tx_start, rx_start) = unbounded::<()>();
    let (tx_stop, rx_stop) = unbounded::<()>();

    let state_engine = app_state.clone();
    thread::spawn(move || {
        let mut enigo = Enigo::new(&Settings::default()).expect("Init Error");
        let mut last_fire = Instant::now();
        let mut history = Vec::new();
        loop {
            while let Ok(_) = rx_start.try_recv() {
                if !state_engine.is_running.load(Ordering::SeqCst) {
                    state_engine.is_running.store(true, Ordering::SeqCst);
                    *state_engine.start_time.lock().unwrap() = Some(Instant::now());
                }
            }
            while let Ok(_) = rx_stop.try_recv() {
                if state_engine.is_running.load(Ordering::SeqCst) {
                    state_engine.is_running.store(false, Ordering::SeqCst);
                    let s = state_engine.start_time.lock().unwrap().take();
                    if let Some(st) = s { *state_engine.elapsed_time.lock().unwrap() += st.elapsed(); }
                }
            }
            if state_engine.is_running.load(Ordering::SeqCst) {
                let target = *state_engine.target_key.lock().unwrap();
                let cps = *state_engine.cps.lock().unwrap();
                let mode = *state_engine.mode.lock().unwrap();
                let limit = *state_engine.repeat_count.lock().unwrap();
                let current_total = state_engine.total_presses.load(Ordering::SeqCst);
                if let Some(rk) = target {
                    let interval = Duration::from_micros(1_000_000 / cps as u64);
                    if last_fire.elapsed() >= interval {
                        if let Some(ek) = map_rdev_to_enigo(rk) {
                            let _ = enigo.key(ek, Direction::Press);
                            thread::sleep(Duration::from_millis(1));
                            let _ = enigo.key(ek, Direction::Release);
                            state_engine.total_presses.fetch_add(1, Ordering::SeqCst);
                            history.push(Instant::now());
                            last_fire = Instant::now();
                            if mode == Mode::Count && current_total + 1 >= limit as u64 {
                                state_engine.is_running.store(false, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
            let now = Instant::now();
            history.retain(|&t| now.duration_since(t) <= Duration::from_secs(1));
            state_engine.measured_cps.store(history.len() as u64, Ordering::SeqCst);
            thread::sleep(Duration::from_micros(100));
        }
    });

    let state_listener = app_state.clone();
    let tx_s = tx_start.clone();
    let tx_p = tx_stop.clone();
    thread::spawn(move || {
        listen(move |e| {
            if let EventType::KeyPress(key) = e.event_type {
                if key == RdevKey::F8 {
                    if state_listener.is_running.load(Ordering::SeqCst) { let _ = tx_p.send(()); }
                    else { let _ = tx_s.send(()); }
                }
                if state_listener.capturing_key.load(Ordering::SeqCst) && key != RdevKey::F8 && key != RdevKey::Escape {
                    *state_listener.target_key.lock().unwrap() = Some(key);
                    *state_listener.target_key_name.lock().unwrap() = format!("{:?}", key).to_uppercase();
                    state_listener.capturing_key.store(false, Ordering::SeqCst);
                }
            }
        }).unwrap();
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([460.0, 580.0]) // Window height reduced to match tight UI
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "RapidKey Pro ⚡",
        options,
        Box::new(|cc| {
            setup_theme(&cc.egui_ctx);
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(RapidKeyUI::new(app_state, tx_start, tx_stop)))
        }),
    )
}

fn setup_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = theme::BG_MAIN;
    visuals.window_fill = theme::BG_MAIN;
    visuals.widgets.inactive.bg_fill = theme::BG_SURFACE;
    visuals.widgets.hovered.bg_fill = theme::BG_CARD_HOVER;
    visuals.widgets.active.bg_fill = theme::BG_CARD_HOVER;
    visuals.widgets.noninteractive.bg_fill = theme::BG_MAIN;
    
    let cr = egui::CornerRadius::same(12);
    visuals.widgets.inactive.corner_radius = cr;
    visuals.widgets.hovered.corner_radius = cr;
    visuals.widgets.active.corner_radius = cr;
    visuals.widgets.noninteractive.corner_radius = cr;

    visuals.selection.bg_fill = theme::PRIMARY;
    ctx.set_visuals(visuals);
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let font_paths = ["C:\\Windows\\Fonts\\meiryo.ttc", "C:\\Windows\\Fonts\\YuGothM.ttc", "C:\\Windows\\Fonts\\msgothic.ttc"];
    for path in font_paths {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert("ds_font".to_owned(), egui::FontData::from_owned(data).into());
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "ds_font".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("ds_font".to_owned());
            break;
        }
    }
    ctx.set_fonts(fonts);
}

struct RapidKeyUI {
    state: AppState,
    tx_start: Sender<()>,
    tx_stop: Sender<()>,
}

impl RapidKeyUI {
    fn new(state: AppState, tx_start: Sender<()>, tx_stop: Sender<()>) -> Self {
        Self { state, tx_start, tx_stop }
    }
}

impl eframe::App for RapidKeyUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_running = self.state.is_running.load(Ordering::SeqCst);
        let capturing = self.state.capturing_key.load(Ordering::SeqCst);

        if capturing {
            ctx.input(|i| {
                for e in &i.events {
                    if let egui::Event::Key { key, pressed: true, .. } = e {
                        if *key != egui::Key::F8 && *key != egui::Key::Escape {
                            let rk = map_egui_to_rdev(*key);
                            *self.state.target_key.lock().unwrap() = Some(rk);
                            *self.state.target_key_name.lock().unwrap() = format!("{:?}", rk).to_uppercase();
                            self.state.capturing_key.store(false, Ordering::SeqCst);
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
            // Force strict item spacing globally in update
            ui.spacing_mut().item_spacing.y = 0.0; 

            ui.add_space(theme::PADDING_PAGE);
            
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;

                // HEADER
                ui.label(egui::RichText::new("⚡ RAPIDKEY PRO").size(32.0).strong().color(theme::TEXT_TITLE));
                ui.label(egui::RichText::new("PRECISION PERFORMANCE ENGINE").size(11.0).color(theme::TEXT_SUB).extra_letter_spacing(1.5));
                
                ui.add_space(theme::HEADER_BOTTOM);

                // SECTION: Target Key
                render_section(ui, "TARGET KEY", |ui| {
                    let btn_text = if capturing { "CAPTURING..." } else { &key_name };
                    let btn_fill = if capturing { theme::PRIMARY } else { theme::BG_SURFACE };
                    
                    let key_btn = egui::Button::new(egui::RichText::new(btn_text).size(22.0).strong().color(egui::Color32::WHITE))
                        .fill(btn_fill)
                        .min_size(egui::vec2(360.0, 60.0));
                        
                    if ui.add(key_btn).clicked() {
                        self.state.capturing_key.store(true, Ordering::SeqCst);
                    }
                });

                ui.add_space(theme::GAP_SECTION);

                // SECTION: Settings (GRID FOR ABSOLUTE SPACING CONTROL)
                render_section(ui, "CONFIGURATION", |ui| {
                    egui::Grid::new("config_grid")
                        .num_columns(2)
                        .spacing([12.0, 8.0]) // Horizontal gap 12px, Vertical gap 8px
                        .show(ui, |ui| {
                            // Row 1: Speed
                            ui.label("Speed (CPS):");
                            let mut val = cps;
                            let slider = egui::Slider::new(&mut val, 1..=120).show_value(true).trailing_fill(true);
                            if ui.add_sized([220.0, 20.0], slider).changed() {
                                *self.state.cps.lock().unwrap() = val;
                            }
                            ui.end_row();

                            // Row 2: Mode
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

                            // Row 3: Burst Limit (Reservations for no layout-shift)
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

                ui.add_space(theme::GAP_SECTION * 1.5);

                // MASTER CONTROL
                let main_text = if is_running { "⬛ TERMINATE ENGINE" } else { "▶ INITIALIZE ENGINE" };
                let main_color = if is_running { theme::DANGER } else { theme::PRIMARY };
                
                if ui.add(egui::Button::new(egui::RichText::new(main_text).size(20.0).strong().color(egui::Color32::WHITE))
                    .fill(main_color)
                    .min_size(egui::vec2(380.0, 64.0))).clicked() {
                    if is_running { let _ = self.tx_stop.send(()); } else { let _ = self.tx_start.send(()); }
                }

                ui.add_space(theme::GAP_SECTION * 1.5);

                // ANALYTICS PANEL
                render_stats(ui, &self.state);
                
                ui.add_space(theme::PADDING_PAGE);
                ui.label(egui::RichText::new("HOTKEY: F8 TO TOGGLE").size(10.0).color(theme::TEXT_SUB));
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

fn map_rdev_to_enigo(k: RdevKey) -> Option<Key> {
    match k {
        RdevKey::KeyA => Some(Key::Unicode('a')), RdevKey::KeyB => Some(Key::Unicode('b')),
        RdevKey::KeyC => Some(Key::Unicode('c')), RdevKey::KeyD => Some(Key::Unicode('d')),
        RdevKey::KeyE => Some(Key::Unicode('e')), RdevKey::KeyF => Some(Key::Unicode('f')),
        RdevKey::KeyG => Some(Key::Unicode('g')), RdevKey::KeyH => Some(Key::Unicode('h')),
        RdevKey::KeyI => Some(Key::Unicode('i')), RdevKey::KeyJ => Some(Key::Unicode('j')),
        RdevKey::KeyK => Some(Key::Unicode('k')), RdevKey::KeyL => Some(Key::Unicode('l')),
        RdevKey::KeyM => Some(Key::Unicode('m')), RdevKey::KeyN => Some(Key::Unicode('n')),
        RdevKey::KeyO => Some(Key::Unicode('o')), RdevKey::KeyP => Some(Key::Unicode('p')),
        RdevKey::KeyQ => Some(Key::Unicode('q')), RdevKey::KeyR => Some(Key::Unicode('r')),
        RdevKey::KeyS => Some(Key::Unicode('s')), RdevKey::KeyT => Some(Key::Unicode('t')),
        RdevKey::KeyU => Some(Key::Unicode('u')), RdevKey::KeyV => Some(Key::Unicode('v')),
        RdevKey::KeyW => Some(Key::Unicode('w')), RdevKey::KeyX => Some(Key::Unicode('x')),
        RdevKey::KeyY => Some(Key::Unicode('y')), RdevKey::KeyZ => Some(Key::Unicode('z')),
        RdevKey::Space => Some(Key::Space), RdevKey::Return => Some(Key::Return),
        RdevKey::F1 => Some(Key::F1), RdevKey::F2 => Some(Key::F2),
        RdevKey::F3 => Some(Key::F3), RdevKey::F4 => Some(Key::F4),
        RdevKey::F5 => Some(Key::F5), RdevKey::F6 => Some(Key::F6),
        RdevKey::F7 => Some(Key::F7), RdevKey::F8 => Some(Key::F8),
        RdevKey::F9 => Some(Key::F9), RdevKey::F10 => Some(Key::F10),
        RdevKey::F11 => Some(Key::F11), RdevKey::F12 => Some(Key::F12),
        _ => None,
    }
}

fn map_egui_to_rdev(k: egui::Key) -> RdevKey {
    match k {
        egui::Key::A => RdevKey::KeyA, egui::Key::B => RdevKey::KeyB,
        egui::Key::C => RdevKey::KeyC, egui::Key::D => RdevKey::KeyD,
        egui::Key::E => RdevKey::KeyE, egui::Key::F => RdevKey::KeyF,
        egui::Key::G => RdevKey::KeyG, egui::Key::H => RdevKey::KeyH,
        egui::Key::I => RdevKey::KeyI, egui::Key::J => RdevKey::KeyJ,
        egui::Key::K => RdevKey::KeyK, egui::Key::L => RdevKey::KeyL,
        egui::Key::M => RdevKey::KeyM, egui::Key::N => RdevKey::KeyN,
        egui::Key::O => RdevKey::KeyO, egui::Key::P => RdevKey::KeyP,
        egui::Key::Q => RdevKey::KeyQ, egui::Key::R => RdevKey::KeyR,
        egui::Key::S => RdevKey::KeyS, egui::Key::T => RdevKey::KeyT,
        egui::Key::U => RdevKey::KeyU, egui::Key::V => RdevKey::KeyV,
        egui::Key::W => RdevKey::KeyW, egui::Key::X => RdevKey::KeyX,
        egui::Key::Y => RdevKey::KeyY, egui::Key::Z => RdevKey::KeyZ,
        egui::Key::Space => RdevKey::Space, egui::Key::Enter => RdevKey::Return,
        _ => RdevKey::Unknown(0),
    }
}
