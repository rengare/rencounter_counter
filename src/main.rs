extern crate image;
extern crate regex;
extern crate rten_imageio;
extern crate rten_tensor;
extern crate scrap;

mod encounter;
mod tui;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use encounter::{encounter_process, load_state, save_state, EncounterState, Mode};
use ocrs::{OcrEngine, OcrEngineParams};
use ratatui::{
    layout::Alignment,
    prelude::Stylize,
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph,
    },
    Frame,
};
use rten::Model;
use scrap::{Capturer, Display};
use std::error::Error;
use std::fs;

fn load_engine() -> Result<OcrEngine, Box<dyn Error>> {
    let detection_model_data = fs::read("text-detection.rten")?;
    let rec_model_data = fs::read("text-recognition.rten")?;
    let detection_model = Model::load(&detection_model_data)?;
    let recognition_model = Model::load(&rec_model_data)?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;

    Ok(engine)
}

#[derive()]
pub struct App {
    exit: bool,
    encounter_state: EncounterState,
    engine: OcrEngine,
}

impl App {
    fn new() -> Self {
        let mut t = Self {
            exit: false,
            encounter_state: EncounterState::default(),
            engine: load_engine().unwrap(),
        };
        if let Ok(loaded) = load_state() {
            t.encounter_state = loaded;
            t.encounter_state.mode = Mode::Init;
        };
        t
    }

    fn run(&mut self, terminal: &mut tui::Tui) -> Result<(), Box<dyn Error>> {
        let display = Display::primary().expect("Couldn't find primary display.");
        let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

        loop {
            terminal.draw(|frame| self.render_frame(frame))?;
            encounter_process(&self.engine, &mut capturer, &mut self.encounter_state)?;

            if self.exit {
                break;
            }

            if event::poll(std::time::Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        self.handle_key_event(key_event)
                    }
                    _ => {}
                };
            }
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render_frame(&self, frame: &mut Frame) {
        let title = Title::from("Rencounter Counter".bold());

        let instructions = Title::from(Line::from(vec![
            " Start ".into(),
            " <S> ".blue().bold(),
            " Pause ".into(),
            " <P> ".blue().bold(),
            " Reset ".into(),
            " <R> ".blue().bold(),
            " Quit ".into(),
            " <Q> ".blue().bold(),
        ]));

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        // top 5 encountered pokemon

        let mut top_five = self
            .encounter_state
            .mon_stats
            .iter()
            .collect::<Vec<(&String, &u32)>>();

        top_five.sort_by(|a, b| Ord::cmp(&b.1, &a.1));

        let mut texts = vec![
            Line::from("Encounter number").centered(),
            Line::from(format!("{}", self.encounter_state.encounters)).centered(),
            Line::from("").centered(),
            Line::from("Last encounter").centered(),
            Line::from(format!("{:?}", self.encounter_state.last_encounter)).centered(),
            Line::from("").centered(),
            Line::from("Lur").centered(),
            Line::from(format!("{}", self.encounter_state.lure_on))
                .yellow()
                .centered(),
            Line::from("").centered(),
            Line::from("Mode").centered(),
            Line::from(format!("{}", self.encounter_state.mode)).centered(),
            Line::from("").centered(),
            Line::from("Top 5 encounters").centered(),
        ];

        for i in 0..5 {
            if let Some(mon) = top_five.get(i) {
                texts.push(Line::from(format!("{}: {}", mon.0, mon.1)).centered());
            }
        }

        // .map(|(name, count)| format!("{}: {}", name, count))
        // .collect::<Vec<String>>();

        // let top_five = top_five
        //     .iter()
        //     .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
        //     .take(5)
        //     .collect::<Vec<&(String, u32)>>();

        frame.render_widget(Paragraph::new(texts).block(block), frame.size());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('s') => self.encounter_state.mode = Mode::Walk,
            KeyCode::Char('p') => self.encounter_state.mode = Mode::Pause,
            KeyCode::Char('r') => {
                self.encounter_state = EncounterState::default();
                save_state(&self.encounter_state).unwrap_or_default();
            }
            _ => {}
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = tui::init()?;
    terminal.clear()?;

    let mut app = App::default();
    app.run(&mut terminal)?;

    tui::restore()?;
    terminal.clear()?;

    Ok(())
}
