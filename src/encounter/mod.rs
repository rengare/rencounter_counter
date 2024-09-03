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

const ENCOUNTER_DETECT_FRAMES: i32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Init,
    Encounter,
    Walk,
    Pause,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Init => write!(f, "Init, Press S to start."),
            Mode::Encounter => write!(f, "Encounter"),
            Mode::Walk => write!(f, "Walk"),
            Mode::Pause => write!(f, "Pause"),
        }
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
        match self {
            Toggle::Exp => write!(f, "Exp Mode"),
            Toggle::Runaway => write!(f, "Runaway Mode"),
            Toggle::Safari => write!(f, "Safari Mode"),
        }
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
    return name == APP_NAME || title == APP_NAME || name == JAVA || title == JAVA;
}

pub fn get_current_working_dir() -> (String, String) {
    let exe_path = std::env::current_exe();

    let path = std::env::current_dir();
    if exe_path.is_err() || path.is_err() {
        panic!("can't find currenct directory");
    } else {
        (
            exe_path.unwrap().parent().unwrap().display().to_string(),
            path.unwrap().display().to_string(),
        )
    }
}

pub fn load_state() -> Result<EncounterState, Box<dyn Error>> {
    let state_json = fs::read_to_string("state.json")?;
    let state: EncounterState = serde_json::from_str(&state_json)?;
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

    let mut mons: Vec<String> = vec![];
    let mut lure_on = false;

    // Process each line, streamlined handling of None values and text processing
    line_texts
        .iter()
        .flatten()
        .filter(|l| l.to_string().len() > 0)
        .map(|line| line.to_string().to_lowercase())
        .for_each(|l| {
            // Check if 'lure' is in the line
            if l.contains("lure") {
                lure_on = true;
            }

            if l.contains("lv.") || l.contains("nv.") {
                // Efficiently find and collect monster names without collecting into Vec
                let words = l.split_whitespace().collect::<Vec<_>>();
                words
                    .windows(2)
                    .filter(|w| (w[1] == "lv." || w[1] == "nv.") && w[0].len() > 1)
                    .for_each(|w| {
                        mons.push(w[0].to_string());
                    });
            }
        });

    Ok((mons, lure_on))
}

fn capture_screen(debug: bool) -> Result<RgbImage, Box<dyn Error>> {
    if let Some(window) = Window::all().into_iter().flatten().find(|w| game_exist(&w)) {
        let img = window.capture_image()?;
        let img = DynamicImage::ImageRgba8(img)
            .crop(
                0,
                0,
                window.width() as u32,
                (window.height() as f32 * 0.4) as u32,
            )
            .grayscale()
            .to_rgb8();

        if debug {
            let _ = img.save("debug.png");
        }
        return Ok(img);
    } else {
        Err("Game not found".into())
    }
}

pub fn encounter_process(
    engine: &OcrEngine,
    state: &mut EncounterState,
) -> Result<(), Box<dyn Error>> {
    if state.mode == Mode::Init || state.mode == Mode::Pause {
        return Ok(());
    }

    let mut mode_detect: Vec<(Vec<String>, bool)> = vec![];
    if state.mode != Mode::Pause {
        for _ in 1..=ENCOUNTER_DETECT_FRAMES {
            let buffer = capture_screen(state.debug)?;
            let mons = get_mons(engine, buffer)?;
            mode_detect.push(mons);
            thread::sleep(Duration::from_millis(state.toggle.to_num()));
        }
    }

    match state.mode {
        Mode::Encounter => {
            if mode_detect.iter().all(|(m, _)| m.is_empty()) {
                state.mode = Mode::Walk;
                if let Some(lure) = mode_detect.first() {
                    state.lure_on = lure.1;
                }
            }
        }
        Mode::Walk => {
            if mode_detect.iter().any(|(m, _)| !m.is_empty()) {
                let mut mons: Vec<String> = vec![];
                let mut is_lure = false;

                for (m, lure) in mode_detect.iter() {
                    if !m.is_empty() && m.len() >= mons.len() {
                        mons = m.clone();
                        is_lure = *lure;
                    }
                }

                state.encounters += mons.len() as u32;
                state.last_encounter = mons.clone();
                state.mode = Mode::Encounter;
                state.lure_on = is_lure;

                mons.iter().for_each(|m| {
                    let count = state.mon_stats.entry(m.clone()).or_insert(0);
                    *count += 1;
                });
            }
        }
        _ => {}
    }

    save_state(state)?;
    Ok(())
}
