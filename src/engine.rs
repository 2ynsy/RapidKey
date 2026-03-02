use crate::models::{AppState, Mode};
use crate::utils::map_rdev_to_enigo;
use crossbeam_channel::{Receiver, Sender};
use enigo::{Direction, Enigo, Keyboard, Settings};
use rdev::{listen, EventType, Key as RdevKey};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

pub fn spawn_fire_engine(app_state: AppState, rx_start: Receiver<()>, rx_stop: Receiver<()>) {
    thread::spawn(move || {
        let mut enigo = Enigo::new(&Settings::default()).expect("Init Error");
        let mut last_fire = Instant::now();
        let mut history = Vec::new();
        loop {
            while let Ok(_) = rx_start.try_recv() {
                app_state.start();
            }
            while let Ok(_) = rx_stop.try_recv() {
                app_state.stop();
            }

            if app_state.is_running() {
                let target = app_state.get_target_key();
                let cps = *app_state.cps.lock().unwrap();
                let mode = *app_state.mode.lock().unwrap();
                let limit = *app_state.repeat_count.lock().unwrap();
                let current_total = app_state.total_presses.load(Ordering::SeqCst);

                if let Some(rk) = target {
                    let interval = Duration::from_micros(1_000_000 / cps as u64);
                    if last_fire.elapsed() >= interval {
                        if let Some(ek) = map_rdev_to_enigo(rk) {
                            let _ = enigo.key(ek, Direction::Press);
                            thread::sleep(Duration::from_millis(1));
                            let _ = enigo.key(ek, Direction::Release);
                            app_state.total_presses.fetch_add(1, Ordering::SeqCst);
                            history.push(Instant::now());
                            last_fire = Instant::now();
                            if mode == Mode::Count && current_total + 1 >= limit as u64 {
                                app_state.stop();
                            }
                        }
                    }
                }
            }
            
            let now = Instant::now();
            history.retain(|&t| now.duration_since(t) <= Duration::from_secs(1));
            app_state.measured_cps.store(history.len() as u64, Ordering::SeqCst);
            thread::sleep(Duration::from_micros(100));
        }
    });
}

pub fn spawn_event_listener(app_state: AppState, tx_start: Sender<()>, tx_stop: Sender<()>) {
    thread::spawn(move || {
        app_state.log("Global Listener Active");
        
        // rdev::listen is blocking and provides its own message loop on Windows
        let engine_state = app_state.clone();
        let result = listen(move |e| {
            if let EventType::KeyPress(key) = e.event_type {
                // Log all keys briefly to help debug if it's hearing anything
                engine_state.log(&format!("Key: {:?}", key));
                
                // Use F8 or F9 as toggle keys
                if key == RdevKey::F8 || key == RdevKey::F9 {
                    engine_state.toggle();
                    
                    // Direct wake-up for the fire engine
                    if engine_state.is_running() {
                        let _ = tx_start.send(());
                    } else {
                        let _ = tx_stop.send(());
                    }
                }
                
                // Capture target key logic
                if engine_state.capturing_key.load(Ordering::SeqCst) 
                    && key != RdevKey::F8 
                    && key != RdevKey::F9
                    && key != RdevKey::Escape 
                {
                    engine_state.set_target_key(key);
                }
            }
        });

        if let Err(e) = result {
            app_state.log(&format!("Listener Error: {:?}", e));
        }
    });
}
