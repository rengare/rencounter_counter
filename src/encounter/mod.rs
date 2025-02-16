use core::panic;
use image::{DynamicImage, RgbImage};
use ocrs::{ImageSource, OcrEngine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::thread;
use std::time::Duration;
use xcap::Window;

pub const APP_NAME: &str = "pokemmo";
pub const JAVA: &str = "java";
const ENCOUNTER_DETECT_FRAMES: i32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Init,
    Encounter,
    Walk,
    Pause,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode_str = match self {
            Mode::Init => "Init, Press S to start.",
            Mode::Encounter => "Encounter",
            Mode::Walk => "Walk",
            Mode::Pause => "Pause",
        };
        write!(f, "{}", mode_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Toggle {
    Exp,
    Runaway,
    Safari,
}

impl Toggle {
    fn to_num(&self) -> u64 {
        match self {
            Toggle::Exp => 2000,
            Toggle::Runaway => 800,
            Toggle::Safari => 200,
        }
    }
}

impl std::fmt::Display for Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let toggle_str = match self {
            Toggle::Exp => "Exp Mode",
            Toggle::Runaway => "Runaway Mode",
            Toggle::Safari => "Safari Mode",
        };
        write!(f, "{}", toggle_str)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncounterState {
    pub encounters: u32,
    pub last_encounter: Vec<String>,
    pub mode: Mode,
    pub mon_stats: HashMap<String, u32>,
    pub lure_on: bool,
    pub toggle: Toggle,
    pub debug: bool,
}

impl Default for EncounterState {
    fn default() -> Self {
        Self {
            encounters: 0,
            last_encounter: vec![],
            mode: Mode::Init,
            mon_stats: HashMap::new(),
            lure_on: false,
            toggle: Toggle::Runaway,
            debug: false,
        }
    }
}

pub fn game_exist(w: &&Window) -> bool {
    let name = w.app_name().to_lowercase();
    let title = w.title().to_lowercase();
    [APP_NAME, JAVA].contains(&name.as_str()) || [APP_NAME, JAVA].contains(&title.as_str())
}

pub fn get_current_working_dir() -> (String, String) {
    match (std::env::current_exe(), std::env::current_dir()) {
        (Ok(exe_path), Ok(path)) => (
            exe_path.parent().unwrap().display().to_string(),
            path.display().to_string(),
        ),
        _ => panic!("can't find current directory"),
    }
}

pub fn load_state() -> Result<EncounterState, Box<dyn Error>> {
    let state_json = fs::read_to_string("state.json")?;
    let state = serde_json::from_str(&state_json)?;
    Ok(state)
}

pub fn save_state(state: &EncounterState) -> Result<(), Box<dyn Error>> {
    let state_json = serde_json::to_string(state)?;
    fs::write("state.json", state_json)?;
    Ok(())
}

fn get_mons(engine: &OcrEngine, data: RgbImage) -> Result<(Vec<String>, bool), Box<dyn Error>> {
    let img = ImageSource::from_bytes(data.as_raw(), data.dimensions())?;
    let ocr_input = engine.prepare_input(img)?;
    let word_rects = engine.detect_words(&ocr_input)?;
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    let mut mons = Vec::new();
    let mut lure_on = false;

    for line in line_texts
        .iter()
        .flatten()
        .map(|l| l.to_string().to_lowercase())
    {
        if line.contains("lure") {
            lure_on = true;
        }

        if line.contains("lv.") || line.contains("nv.") || line.contains("niv.") {
            line.split_whitespace()
                .collect::<Vec<_>>()
                .windows(2)
                .filter(|w| (w[1] == "lv." || w[1] == "nv." || w[1] == "niv.") && w[0].len() > 1)
                .for_each(|w| mons.push(w[0].to_string()));
        }
    }

    Ok((mons, lure_on))
}

fn capture_screen(debug: bool, window: &Window) -> Result<RgbImage, Box<dyn Error>> {
    let get_image = |w: &Window| {
        let factor = 0.5;

        let img = w.capture_image()?;
        let img = DynamicImage::ImageRgba8(img)
            .crop(0, 0, w.width(), (w.height() as f32 * factor) as u32)
            .grayscale()
            .to_rgb8();

        if debug {
            img.save("debug.png")?;
        }

        Ok(img)
    };

    if let Some(w) = Window::all().unwrap().iter().find(game_exist) {
        get_image(w)
    } else {
        get_image(window)
    }
}

pub fn encounter_process(
    engine: &OcrEngine,
    state: &mut EncounterState,
    window: &Window,
) -> Result<(), Box<dyn Error>> {
    if matches!(state.mode, Mode::Init | Mode::Pause) {
        return Ok(());
    }

    let mut mode_detect = Vec::with_capacity(ENCOUNTER_DETECT_FRAMES as usize);
    for _ in 1..=ENCOUNTER_DETECT_FRAMES {
        let buffer = capture_screen(state.debug, window)?;
        let mons = get_mons(engine, buffer)?;
        mode_detect.push(mons);
        thread::sleep(Duration::from_millis(state.toggle.to_num()));
    }

    match state.mode {
        Mode::Encounter => {
            if mode_detect.iter().all(|(m, _)| m.is_empty()) {
                state.mode = Mode::Walk;
                state.lure_on = mode_detect.first().map_or(false, |(_, lure)| *lure);
            }
        }
        Mode::Walk => {
            if let Some((mons, is_lure)) = mode_detect
                .iter()
                .filter(|(m, _)| !m.is_empty())
                .max_by_key(|(m, _)| m.len())
            {
                state.encounters += mons.len() as u32;
                state.last_encounter = mons.clone();
                state.mode = Mode::Encounter;
                state.lure_on = *is_lure;

                for mon in mons {
                    *state.mon_stats.entry(mon.clone()).or_insert(0) += 1;
                }
            }
        }
        _ => {}
    }

    save_state(state)?;
    Ok(())
}
