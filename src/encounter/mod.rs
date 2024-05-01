use image::{DynamicImage, ImageBuffer, Rgba};
use ocrs::OcrEngine;
use regex::Regex;
use rten_imageio::read_image;
use rten_tensor::prelude::*;
use scrap::{Capturer, Display};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;

const CAPTURED_SCREEN_PATH: &str = "test.png";
const SLEEP_TIME_MS: u64 = 200;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Init,
    Encounter,
    Walk,
    Pause,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Init => write!(f, "Init"),
            Mode::Encounter => write!(f, "Encounter"),
            Mode::Walk => write!(f, "Walk"),
            Mode::Pause => write!(f, "Pause"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncounterState {
    pub encounters: u32,
    pub last_encounter: Vec<String>,
    pub mode: Mode,
}

impl Default for EncounterState {
    fn default() -> Self {
        Self {
            encounters: 0,
            last_encounter: vec![],
            mode: Mode::Init,
        }
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

fn get_mons(engine: &OcrEngine, path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let image = read_image(path)?;
    let ocr_input = engine.prepare_input(image.view())?;
    let word_rects = engine.detect_words(&ocr_input)?;
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    let pokemon_regex = Regex::new(r"[0-9\s]").unwrap();

    let mut mons: Vec<String> = vec![];
    line_texts.iter().for_each(|line| {
        line.iter()
            .filter(|l| l.to_string().contains("Lv."))
            .for_each(|l| {
                l.words()
                    .map(|w| w.to_string())
                    .filter(|w| w.chars().next().unwrap().is_uppercase())
                    .map(|w| w.to_lowercase())
                    .filter(|w| {
                        w.len() > 3
                            && !pokemon_regex.is_match(w)
                            && (!w.contains("lv.") || !w.contains("llv."))
                    })
                    .for_each(|w| {
                        mons.push(w.replace("lv.", "").replace("llv.", "").to_string());
                    });
            });
    });
    Ok(mons)
}

fn capture_screen(path: &str) -> Result<(), Box<dyn Error>> {
    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 60;

    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());

    loop {
        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    // Keep spinning.
                    thread::sleep(one_frame);
                    continue;
                } else {
                    panic!("Error: {}", error);
                }
            }
        };

        let mut bitflipped = Vec::with_capacity(w * h * 4);
        let mut stride = 0;

        #[cfg(target_os = "macos")]
        {
            stride = w * 4;
        }

        #[cfg(not(target_os = "macos"))]
        {
            stride = buffer.len() / h;
        }

        for y in 0..h {
            for x in 0..w {
                let i = stride * y + 4 * x;
                bitflipped.extend_from_slice(&[buffer[i + 2], buffer[i + 1], buffer[i], 255]);
            }
        }

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            image::ImageBuffer::from_raw(w as u32, h as u32, Vec::from(&*bitflipped)).unwrap();

        let rgba = DynamicImage::ImageRgba8(img)
            .crop(150, 50, w as u32, (h / 2 - 150) as u32)
            .grayscale();

        rgba.brighten(250);
        rgba.save(path)?;

        return Ok(());
    }
}

pub fn encounter_process(
    engine: &OcrEngine,
    state: &mut EncounterState,
) -> Result<(), Box<dyn Error>> {
    if state.mode == Mode::Init || state.mode == Mode::Pause {
        return Ok(());
    }

    let mut mode_detect: Vec<Vec<String>> = vec![];
    let mut mons: Vec<String> = vec![];
    if state.mode != Mode::Pause {
        for _ in 0..2 {
            capture_screen(CAPTURED_SCREEN_PATH)?;
            mons = get_mons(engine, CAPTURED_SCREEN_PATH)?;
            thread::sleep(Duration::from_millis(SLEEP_TIME_MS));
            mode_detect.push(mons.clone());
        }
    }

    match state.mode {
        Mode::Encounter => {
            if mode_detect.iter().all(|m| m.is_empty()) {
                state.mode = Mode::Walk;
            }
        }
        Mode::Walk => {
            if !mons.is_empty() {
                state.encounters += mons.len() as u32;
                state.last_encounter = mons;
                state.mode = Mode::Encounter;
            }
        }
        _ => {}
    }

    save_state(state)?;
    Ok(())
}
