use rdev::Key as RdevKey;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Toggle,
    Hold,
    Count,
}

#[derive(Clone)]
pub struct AppState {
    pub target_key: Arc<Mutex<Option<RdevKey>>>,
    pub target_key_name: Arc<Mutex<String>>,
    pub cps: Arc<Mutex<u32>>,
    pub mode: Arc<Mutex<Mode>>,
    pub repeat_count: Arc<Mutex<u32>>,
    pub is_running: Arc<AtomicBool>,
    pub total_presses: Arc<AtomicU64>,
    pub measured_cps: Arc<AtomicU64>,
    pub elapsed_time: Arc<Mutex<Duration>>,
    pub start_time: Arc<Mutex<Option<Instant>>>,
    pub capturing_key: Arc<AtomicBool>,
    pub debug_msg: Arc<Mutex<String>>,
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
            debug_msg: Arc::new(Mutex::new("Ready".to_string())),
        }
    }
}

impl AppState {
    pub fn log(&self, msg: &str) {
        if let Ok(mut log) = self.debug_msg.lock() {
            *log = msg.to_string();
        }
    }

    pub fn get_target_key(&self) -> Option<RdevKey> {
        *self.target_key.lock().unwrap()
    }
    
    pub fn set_target_key(&self, key: RdevKey) {
        *self.target_key.lock().unwrap() = Some(key);
        *self.target_key_name.lock().unwrap() = format!("{:?}", key).to_uppercase();
        self.capturing_key.store(false, Ordering::SeqCst);
        self.log(&format!("Key Set: {:?}", key));
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn toggle(&self) {
        if self.is_running() {
            self.stop();
            self.log("Engine Stopped (Hotkey)");
        } else {
            self.start();
            self.log("Engine Started (Hotkey)");
        }
    }

    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            let s = self.start_time.lock().unwrap().take();
            if let Some(st) = s {
                *self.elapsed_time.lock().unwrap() += st.elapsed();
            }
        }
    }

    pub fn start(&self) {
        if !self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(true, Ordering::SeqCst);
            *self.start_time.lock().unwrap() = Some(Instant::now());
        }
    }
}
