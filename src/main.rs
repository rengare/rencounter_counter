extern crate image;
extern crate regex;
extern crate rten_imageio;
extern crate rten_tensor;
extern crate scrap;

use image::*;
use imageproc::*;
use ocrs::{OcrEngine, OcrEngineParams};
use regex::Regex;
use rten::Model;
// use rten_imageio::write_image;
use scrap::{Capturer, Display};
use std::error::Error;
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;

use std::fs;
use std::path::PathBuf;

use rten_imageio::read_image;
use rten_tensor::prelude::*;

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
        capture_screen(&args.image)?;
        //sleep for 1 sec
        thread::sleep(Duration::from_secs(1));

        let mons = get_mons(&engine, &args.image)?;
        println!("encountered {:?}", mons);
    }
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
                    .filter(|w| w.len() > 2 && !pokemon_regex.is_match(w) && !w.contains("Lv."))
                    .for_each(|w| {
                        println!("{}", w);
                        mons.push(w);
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
        // Wait until there's a frame.

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

        // Flip the ARGB image into a BGRA image.

        let mut bitflipped = Vec::with_capacity(w * h * 4);
        let stride = buffer.len() / h;

        for y in 0..h {
            for x in 0..w {
                let i = stride * y + 4 * x;
                bitflipped.extend_from_slice(&[buffer[i + 2], buffer[i + 1], buffer[i], 255]);
            }
        }

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            image::ImageBuffer::from_raw(w as u32, h as u32, Vec::from(&*bitflipped)).unwrap();

        let mut rgba = DynamicImage::ImageRgba8(img).crop(150, 50, w as u32, (h / 2 - 150) as u32);
        // .grayscale();

        rgba.invert();
        rgba.brighten(250);
        rgba.save(path)?;

        return Ok(());
    }
}
