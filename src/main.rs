extern crate image;
extern crate regex;
extern crate rten_imageio;
extern crate rten_tensor;
extern crate scrap;

mod encounter;
mod tui;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use encounter::{encounter_process, load_state, EncounterState, Mode};
use ocrs::{OcrEngine, OcrEngineParams};
use ratatui::{
    layout::Alignment,
    prelude::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph,
    },
    Frame,
};
use rten::Model;
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
        loop {
            terminal.draw(|frame| self.render_frame(frame))?;
            encounter_process(&self.engine, &mut self.encounter_state)?;

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

        let text = Text::from(vec![
            Line::from("Encounter number").centered(),
            Line::from(format!("{}", self.encounter_state.encounters)).centered(),
            Line::from("Last encounter").centered(),
            Line::from(format!("{:?}", self.encounter_state.last_encounter)).centered(),
            Line::from("Mode").centered(),
            Line::from(format!("{}", self.encounter_state.mode)).centered(),
        ]);

        frame.render_widget(Paragraph::new(text).block(block), frame.size());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('s') => self.encounter_state.mode = Mode::Walk,
            KeyCode::Char('p') => self.encounter_state.mode = Mode::Pause,
            KeyCode::Char('r') => self.encounter_state = EncounterState::default(),
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
