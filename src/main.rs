use std::error::Error;
use std::fs;
use std::path::PathBuf;

use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;
use rten_imageio::read_image;
use rten_tensor::prelude::*;
extern crate regex;

use regex::Regex;

struct Args {
    image: String,
}

/// Read a file from a path that is relative to the crate root.
fn read_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut abs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    abs_path.push(path);
    fs::read(abs_path)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args {
        image: "test.png".to_string(),
    };

    let detection_model_data = read_file("text-detection.rten")?;
    let rec_model_data = read_file("text-recognition.rten")?;

    let detection_model = Model::load(&detection_model_data)?;
    let recognition_model = Model::load(&rec_model_data)?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;

    loop {
        let mons = get_mons(&engine, &args.image)?;
        println!("encountered {:?}", mons[0]);
    }
}

fn get_mons(engine: &OcrEngine, path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let image = read_image(path)?;
    let ocr_input = engine.prepare_input(image.view())?;
    let word_rects = engine.detect_words(&ocr_input)?;
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    let pokemon_regex = Regex::new(r"(\S+)\s+Lv\.").unwrap();

    let mons: Vec<Vec<String>> = line_texts
        .iter()
        .flatten()
        .map(|l| l.to_string())
        .filter(|l| l.to_string().len() > 1 && l.to_string().contains("Lv."))
        .map(|l| {
            return pokemon_regex
                .captures_iter(&l)
                .map(|cap| {
                    let full_match = cap[1].to_string();
                    let parts: Vec<&str> = full_match.split_whitespace().collect();
                    parts.last().unwrap().to_string()
                })
                .collect::<Vec<String>>();
        })
        .collect();

    Ok(mons)
}
