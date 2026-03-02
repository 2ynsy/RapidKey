use eframe::egui;
use enigo::{Enigo, Key, Keyboard, Settings, Direction};
use rdev::{listen, EventType, Key as RdevKey};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};
use crossbeam_channel::{unbounded, Receiver, Sender};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Mode {
    Toggle,
    Hold,
    Count,
}

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
            target_key_name: Arc::new(Mutex::new("クリックして設定".to_string())),
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
    
    // Command channels
    let (tx_start, rx_start) = unbounded::<()>();
    let (tx_stop, rx_stop) = unbounded::<()>();

    // Engine Thread
    let state_engine = Arc::new(app_state.clone());
    thread::spawn(move || {
        let mut enigo = Enigo::new(&Settings::default()).expect("Failed to initialize Enigo");
        let mut last_fire = Instant::now();
        let mut presses_in_last_sec = Vec::new();

        loop {
            // Check for explicit commands
            while let Ok(_) = rx_start.try_recv() {
                state_engine.is_running.store(true, Ordering::SeqCst);
                *state_engine.start_time.lock().unwrap() = Some(Instant::now());
            }
            while let Ok(_) = rx_stop.try_recv() {
                state_engine.is_running.store(false, Ordering::SeqCst);
                let start = *state_engine.start_time.lock().unwrap();
                if let Some(s) = start {
                    *state_engine.elapsed_time.lock().unwrap() += s.elapsed();
                }
                *state_engine.start_time.lock().unwrap() = None;
            }

            let is_running = state_engine.is_running.load(Ordering::SeqCst);
            
            if is_running {
                let target_opt = *state_engine.target_key.lock().unwrap();
                let cps = *state_engine.cps.lock().unwrap();
                let mode = *state_engine.mode.lock().unwrap();
                let repeat_limit = *state_engine.repeat_count.lock().unwrap();
                let total = state_engine.total_presses.load(Ordering::SeqCst);

                if let Some(rdev_key) = target_opt {
                    let interval = Duration::from_micros(1_000_000 / cps as u64);
                    
                    if last_fire.elapsed() >= interval {
                        if let Some(enigo_key) = map_rdev_to_enigo(rdev_key) {
                            // High performance clicking
                            let _ = enigo.key(enigo_key, Direction::Click);
                            
                            state_engine.total_presses.fetch_add(1, Ordering::SeqCst);
                            presses_in_last_sec.push(Instant::now());
                            last_fire = Instant::now();

                            if mode == Mode::Count && total + 1 >= repeat_limit as u64 {
                                state_engine.is_running.store(false, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }

            // Cleanup old presses for CPS measurement
            let now = Instant::now();
            presses_in_last_sec.retain(|&t| now.duration_since(t) <= Duration::from_secs(1));
            state_engine.measured_cps.store(presses_in_last_sec.len() as u64, Ordering::SeqCst);

            // CPU friendly sleep
            thread::sleep(Duration::from_millis(1));
        }
    });

    // Global Key Listener (hooks)
    let state_listener = app_state.clone();
    let tx_s = tx_start.clone();
    let tx_p = tx_stop.clone();
    thread::spawn(move || {
        if let Err(error) = listen(move |event| {
            if let EventType::KeyPress(key) = event.event_type {
                // Global Toggle: F8
                if key == RdevKey::F8 {
                    let current = state_listener.is_running.load(Ordering::SeqCst);
                    if current {
                        let _ = tx_p.send(());
                    } else {
                        let _ = tx_s.send(());
                    }
                }

                // Capture key if in capture mode
                if state_listener.capturing_key.load(Ordering::SeqCst) {
                    if key != RdevKey::F8 {
                        *state_listener.target_key.lock().unwrap() = Some(key);
                        let name = format!("{:?}", key).to_uppercase();
                        *state_listener.target_key_name.lock().unwrap() = name;
                        state_listener.capturing_key.store(false, Ordering::SeqCst);
                    }
                }
            }
        }) {
            eprintln!("Error listening to keys: {:?}", error);
        }
    });

    // UI RUN
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([460.0, 680.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "RapidKey Rust ⚡",
        options,
        Box::new(|_cc| {
            // Apply custom dark/premium style
            let mut visuals = egui::Visuals::dark();
            visuals.window_rounding = 12.0.into();
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(13, 15, 24); // BG
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(19, 22, 43);      // SURFACE
            visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(108, 99, 255)); // ACCENT
            _cc.egui_ctx.set_visuals(visuals);
            
            Ok(Box::new(RapidKeyUI::new(app_state, tx_start, tx_stop)))
        }),
    )
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
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(egui::RichText::new("⚡ RapidKey").size(32.0).strong().color(egui::Color32::from_rgb(179, 174, 255)));
                ui.label(egui::RichText::new("Windows Key Rapid-fire Tool (Rust ver)").color(egui::Color32::from_rgb(120, 128, 168)));
                ui.add_space(20.0);
            });

            // TARGET KEY CARD
            ui.group(|ui| {
                ui.set_width(420.0);
                ui.label(egui::RichText::new("📌 ターゲットキー").strong().size(11.0).color(egui::Color32::from_rgb(120, 128, 168)));
                ui.add_space(8.0);
                
                let text = if capturing { 
                    "⌨ キーを押してください...".to_string() 
                } else { 
                    self.state.target_key_name.lock().unwrap().clone() 
                };

                let color = if capturing { egui::Color32::from_rgb(108, 99, 255) } else { egui::Color32::WHITE };
                
                if ui.add(egui::Button::new(egui::RichText::new(text).size(20.0).strong().color(color)).min_size(egui::vec2(400.0, 60.0))).clicked() {
                    self.state.capturing_key.store(true, Ordering::SeqCst);
                }
                
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                     ui.label("よく使う:");
                     if ui.button("Z").clicked() { self.set_key(RdevKey::KeyZ, "KeyZ"); }
                     if ui.button("X").clicked() { self.set_key(RdevKey::KeyX, "KeyX"); }
                     if ui.button("Space").clicked() { self.set_key(RdevKey::Space, "Space"); }
                });
            });

            ui.add_space(15.0);

            // SETTINGS CARD
            ui.group(|ui| {
                ui.set_width(420.0);
                ui.label(egui::RichText::new("⚙️ 設定").strong().size(11.0).color(egui::Color32::from_rgb(120, 128, 168)));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let mut cps = *self.state.cps.lock().unwrap();
                    ui.label("速度:");
                    if ui.add(egui::Slider::new(&mut cps, 1..=120).text("CPS")).changed() {
                        *self.state.cps.lock().unwrap() = cps;
                    }
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    let mut mode = *self.state.mode.lock().unwrap();
                    ui.label("モード:");
                    ui.radio_value(&mut mode, Mode::Toggle, "トグル");
                    ui.radio_value(&mut mode, Mode::Hold, "ホールド");
                    ui.radio_value(&mut mode, Mode::Count, "回数指定");
                    *self.state.mode.lock().unwrap() = mode;
                });

                if *self.state.mode.lock().unwrap() == Mode::Count {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("連打回数:");
                        let mut count = *self.state.repeat_count.lock().unwrap();
                        if ui.add(egui::DragValue::new(&mut count).range(1..=999999)).changed() {
                            *self.state.repeat_count.lock().unwrap() = count;
                        }
                    });
                }
                
                ui.add_space(10.0);
                ui.label(egui::RichText::new("F8 で開始・停止").italics().color(egui::Color32::GRAY));
            });

            ui.add_space(20.0);

            // MAIN BUTTON
            let btn_text = if is_running { "⬛ 停止" } else { "▶ 連打開始" };
            let btn_color = if is_running { egui::Color32::from_rgb(255, 94, 122) } else { egui::Color32::from_rgb(108, 99, 255) };
            
            if ui.add(egui::Button::new(egui::RichText::new(btn_text).size(22.0).strong().color(egui::Color32::WHITE))
                .fill(btn_color)
                .min_size(egui::vec2(420.0, 60.0))).clicked() 
            {
                if is_running {
                    let _ = self.tx_stop.send(());
                } else {
                    let _ = self.tx_start.send(());
                }
            }

            ui.add_space(20.0);

            // STATS CARD
            ui.group(|ui| {
                ui.set_width(420.0);
                ui.label(egui::RichText::new("📊 統計").strong().size(11.0).color(egui::Color32::from_rgb(120, 128, 168)));
                ui.add_space(10.0);

                ui.columns(3, |columns| {
                    columns[0].vertical_centered(|ui| {
                        ui.label(egui::RichText::new(format!("{}", self.state.total_presses.load(Ordering::Relaxed))).size(24.0).strong());
                        ui.label("TOTAL");
                    });
                    columns[1].vertical_centered(|ui| {
                        ui.label(egui::RichText::new(format!("{}", self.state.measured_cps.load(Ordering::Relaxed))).size(24.0).strong());
                        ui.label("実測CPS");
                    });
                    columns[2].vertical_centered(|ui| {
                        let total_elapsed = *self.state.elapsed_time.lock().unwrap();
                        let current_session = if let Some(start) = *self.state.start_time.lock().unwrap() {
                            start.elapsed()
                        } else {
                            Duration::ZERO
                        };
                        let elapsed = total_elapsed + current_session;
                        ui.label(egui::RichText::new(format!("{:.1} s", elapsed.as_secs_f32())).size(24.0).strong());
                        ui.label("経過時間");
                    });
                });
            });

            // Update UI periodically during run
            ctx.request_repaint_after(Duration::from_millis(16)); // ~60fps stats
        });
    }
}

impl RapidKeyUI {
    fn set_key(&mut self, key: RdevKey, name: &str) {
        *self.state.target_key.lock().unwrap() = Some(key);
        *self.state.target_key_name.lock().unwrap() = name.to_string();
    }
}

fn map_rdev_to_enigo(key: RdevKey) -> Option<Key> {
    match key {
        RdevKey::KeyA => Some(Key::O), // enigo mapping for standard keys
        RdevKey::KeyB => Some(Key::Unicode('b')),
        RdevKey::KeyC => Some(Key::Unicode('c')),
        RdevKey::KeyD => Some(Key::Unicode('d')),
        RdevKey::KeyE => Some(Key::Unicode('e')),
        RdevKey::KeyF => Some(Key::Unicode('f')),
        RdevKey::KeyG => Some(Key::Unicode('g')),
        RdevKey::KeyH => Some(Key::Unicode('h')),
        RdevKey::KeyI => Some(Key::Unicode('i')),
        RdevKey::KeyJ => Some(Key::Unicode('j')),
        RdevKey::KeyK => Some(Key::Unicode('k')),
        RdevKey::KeyL => Some(Key::Unicode('l')),
        RdevKey::KeyM => Some(Key::Unicode('m')),
        RdevKey::KeyN => Some(Key::Unicode('n')),
        RdevKey::KeyO => Some(Key::Unicode('o')),
        RdevKey::KeyP => Some(Key::Unicode('p')),
        RdevKey::KeyQ => Some(Key::Unicode('q')),
        RdevKey::KeyR => Some(Key::Unicode('r')),
        RdevKey::KeyS => Some(Key::Unicode('s')),
        RdevKey::KeyT => Some(Key::Unicode('t')),
        RdevKey::KeyU => Some(Key::Unicode('u')),
        RdevKey::KeyV => Some(Key::Unicode('v')),
        RdevKey::KeyW => Some(Key::Unicode('w')),
        RdevKey::KeyX => Some(Key::Unicode('x')),
        RdevKey::KeyY => Some(Key::Unicode('y')),
        RdevKey::KeyZ => Some(Key::Unicode('z')),
        RdevKey::Space => Some(Key::Space),
        RdevKey::Return => Some(Key::Return),
        RdevKey::F1 => Some(Key::F1),
        RdevKey::F2 => Some(Key::F2),
        RdevKey::F3 => Some(Key::F3),
        RdevKey::F4 => Some(Key::F4),
        RdevKey::F5 => Some(Key::F5),
        RdevKey::F6 => Some(Key::F6),
        RdevKey::F7 => Some(Key::F7),
        RdevKey::F8 => Some(Key::F8),
        RdevKey::F9 => Some(Key::F9),
        RdevKey::F10 => Some(Key::F10),
        RdevKey::F11 => Some(Key::F11),
        RdevKey::F12 => Some(Key::F12),
        _ => None,
    }
}
