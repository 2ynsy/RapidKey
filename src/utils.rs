use crate::theme;
use eframe::egui;
use enigo::Key;
use rdev::Key as RdevKey;

pub fn setup_theme(ctx: &egui::Context) {
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

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let font_paths = [
        "C:\\Windows\\Fonts\\meiryo.ttc",
        "C:\\Windows\\Fonts\\YuGothM.ttc",
        "C:\\Windows\\Fonts\\msgothic.ttc",
    ];
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

pub fn map_rdev_to_enigo(k: RdevKey) -> Option<Key> {
    match k {
        RdevKey::KeyA => Some(Key::Unicode('a')),
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

pub fn map_egui_to_rdev(k: egui::Key) -> RdevKey {
    match k {
        egui::Key::A => RdevKey::KeyA,
        egui::Key::B => RdevKey::KeyB,
        egui::Key::C => RdevKey::KeyC,
        egui::Key::D => RdevKey::KeyD,
        egui::Key::E => RdevKey::KeyE,
        egui::Key::F => RdevKey::KeyF,
        egui::Key::G => RdevKey::KeyG,
        egui::Key::H => RdevKey::KeyH,
        egui::Key::I => RdevKey::KeyI,
        egui::Key::J => RdevKey::KeyJ,
        egui::Key::K => RdevKey::KeyK,
        egui::Key::L => RdevKey::KeyL,
        egui::Key::M => RdevKey::KeyM,
        egui::Key::N => RdevKey::KeyN,
        egui::Key::O => RdevKey::KeyO,
        egui::Key::P => RdevKey::KeyP,
        egui::Key::Q => RdevKey::KeyQ,
        egui::Key::R => RdevKey::KeyR,
        egui::Key::S => RdevKey::KeyS,
        egui::Key::T => RdevKey::KeyT,
        egui::Key::U => RdevKey::KeyU,
        egui::Key::V => RdevKey::KeyV,
        egui::Key::W => RdevKey::KeyW,
        egui::Key::X => RdevKey::KeyX,
        egui::Key::Y => RdevKey::KeyY,
        egui::Key::Z => RdevKey::KeyZ,
        egui::Key::Space => RdevKey::Space,
        egui::Key::Enter => RdevKey::Return,
        _ => RdevKey::Unknown(0),
    }
}
