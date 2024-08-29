extern crate image;
extern crate regex;
extern crate rten_imageio;
extern crate rten_tensor;

mod encounter;
mod tui;

use core::panic;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use encounter::{
    encounter_process, get_current_working_dir, load_state, save_state, EncounterState, Mode,
    APP_NAME, JAVA,
};
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
use std::fs;
use std::{env, error::Error};
use xcap::Window;

fn load_engine() -> Result<ocrs::OcrEngine, Box<dyn Error>> {
    let path = get_current_working_dir();
    let detection_model_data = fs::read(format!("{}/text-detection.rten", path))?;
    let rec_model_data = fs::read(format!("{}/text-recognition.rten", path))?;
    let detection_model = Model::load(&detection_model_data)?;
    let recognition_model = Model::load(&rec_model_data)?;

    let engine = ocrs::OcrEngine::new(ocrs::OcrEngineParams {
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
    engine: ocrs::OcrEngine,
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
            " GameMode ".into(),
            " <T> ".blue().bold(),
            " Quit ".into(),
            " <Q> ".blue().bold(),
            " Debug ".into(),
            " <D> ".blue().bold(),
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

        let mut encounter_text = self.encounter_state.encounters.to_string();

        if self.encounter_state.debug {
            encounter_text += "(debug mode)";
        }

        let mut texts = vec![
            Line::from("Encounter number").centered(),
            Line::from(encounter_text.to_string()).centered(),
            Line::from("").centered(),
            Line::from("Last encounter").centered(),
            Line::from(format!("{:?}", self.encounter_state.last_encounter)).centered(),
            Line::from("").centered(),
            Line::from("Lur").centered(),
            Line::from(format!("{}", self.encounter_state.lure_on))
                .yellow()
                .centered(),
            Line::from("").centered(),
            Line::from("Encounter Mode").centered(),
            Line::from(format!("{}", self.encounter_state.mode)).centered(),
            Line::from("").centered(),
            Line::from("Game Mode").centered(),
            Line::from(format!("{}", self.encounter_state.toggle)).centered(),
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
            KeyCode::Char('d') => self.encounter_state.debug = !self.encounter_state.debug,
            KeyCode::Char('p') => self.encounter_state.mode = Mode::Pause,
            KeyCode::Char('t') => {
                self.encounter_state.toggle = match self.encounter_state.toggle {
                    encounter::Toggle::Exp => encounter::Toggle::Runaway,
                    encounter::Toggle::Runaway => encounter::Toggle::Safari,
                    encounter::Toggle::Safari => encounter::Toggle::Exp,
                };
            }
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
    let is_debug = env::args().find(|arg| arg == "debug");

    if is_debug.is_some() {
        println!("The current directory is {}", get_current_working_dir());
        for window in Window::all().unwrap().iter() {
            println!("Window: {:?}", (window.app_name(), window.title()));

            if window.title().to_lowercase() == APP_NAME || window.app_name() == APP_NAME {
                let img = window.capture_image().unwrap();
                let _ = img.save("debug.png");
            }
        }
        return Ok(());
    }

    if let Some(_) = Window::all().unwrap().iter().find(|w| {
        let name = w.app_name().to_lowercase();
        let title = w.title().to_lowercase();
        return name == APP_NAME || title == APP_NAME || name == JAVA || title == JAVA;
    }) {
        let mut terminal = tui::init()?;
        terminal.clear()?;

        let mut app = App::default();
        app.run(&mut terminal)?;

        tui::restore()?;
        terminal.clear()?;
        Ok(())
    } else {
        panic!("{} game not found", APP_NAME);
    }
}
