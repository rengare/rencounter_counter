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
    APP_NAME,
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

use std::fs;
use std::{env, error::Error};
use xcap::Window;

fn init_engine() -> Result<ocrs::OcrEngine, Box<dyn Error>> {
    let (detection_path, recognition_path) = get_path_to_models();
    let (detection_model, recognition_model) = load_rten_model(detection_path, recognition_path)?;

    create_engine(detection_model, recognition_model)
}

fn create_engine(
    detection_model: rten::Model,
    recognition_model: rten::Model,
) -> Result<ocrs::OcrEngine, Box<dyn Error>> {
    let engine = ocrs::OcrEngine::new(ocrs::OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;
    Ok(engine)
}

fn load_rten_model(
    detection_path: String,
    recognition_path: String,
) -> Result<(rten::Model, rten::Model), Box<dyn Error>> {
    let detection_model_data = fs::read(detection_path)?;
    let rec_model_data = fs::read(recognition_path)?;
    let detection_model = rten::Model::load(detection_model_data)?;
    let recognition_model = rten::Model::load(rec_model_data)?;
    Ok((detection_model, recognition_model))
}

fn get_path_to_models() -> (String, String) {
    let (exe_path, path) = get_current_working_dir();

    let detection_path = format!("{}/text-detection.rten", path);
    let detection_path_exe = format!("{}/text-detection.rten", exe_path);
    let recognition_path = format!("{}/text-recognition.rten", path);
    let recognition_path_exe = format!("{}/text-recognition.rten", exe_path);

    match fs::read(&detection_path) {
        Ok(_) => (detection_path, recognition_path),
        _ => (detection_path_exe, recognition_path_exe),
    }
}

enum RunResult {
    Exit,
}

#[derive()]
pub struct App {
    exit: bool,
    pub encounter_state: EncounterState,
    engine: ocrs::OcrEngine,
}

impl App {
    fn new() -> Self {
        let mut t = Self {
            exit: false,
            encounter_state: EncounterState::default(),
            engine: init_engine().unwrap(),
        };
        if let Ok(loaded) = load_state() {
            t.encounter_state = loaded;
            t.encounter_state.mode = Mode::Init;
        };
        t
    }

    fn run(
        &mut self,
        terminal: &mut tui::Tui,
        window: &Window,
    ) -> Result<RunResult, Box<dyn Error>> {
        loop {
            if self.exit {
                return Ok(RunResult::Exit);
            }

            terminal.draw(|frame| self.render_frame(frame))?;

            if encounter_process(&self.engine, &mut self.encounter_state, window).is_err() {
                try_to_restart(terminal)?;
            }

            self.process_keys()?;
        }
    }

    fn process_keys(&mut self) -> Result<(), Box<dyn Error>> {
        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        };
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render_frame(&self, frame: &mut Frame) {
        let title = Title::from("Rencounter Counter".bold());
        let instructions = get_instruction_line();
        let block = get_block(title, instructions);
        let top_five = self.get_top_five();
        let encounter_text = self.get_encounter_text();

        let mut info_lines = self.get_info_lines(encounter_text);
        prepare_info_lines_to_display(top_five, &mut info_lines);

        frame.render_widget(Paragraph::new(info_lines).block(block), frame.area());
    }

    fn get_encounter_text(&self) -> String {
        let mut encounter_text = self.encounter_state.encounters.to_string();

        if self.encounter_state.debug {
            encounter_text += "(debug mode)";
        }
        encounter_text
    }

    fn get_top_five(&self) -> Vec<(&String, &u32)> {
        let mut top_five = self
            .encounter_state
            .mon_stats
            .iter()
            .collect::<Vec<(&String, &u32)>>();

        top_five.sort_by(|a, b| Ord::cmp(&b.1, &a.1));
        top_five
    }

    fn get_info_lines(&self, encounter_text: String) -> Vec<Line<'_>> {
        vec![
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
        ]
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

fn try_to_restart(
    terminal: &mut ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>,
) -> Result<RunResult, Box<dyn Error>> {
    terminal.clear()?;
    if let Some(new_window) = Window::all()?.iter().find(encounter::game_exist) {
        let mut new_app = App::new();
        new_app.encounter_state.mode = Mode::Encounter;
        new_app.run(terminal, new_window)
    } else {
        panic!("{} game not found", APP_NAME);
    }
}

fn prepare_info_lines_to_display(top_five: Vec<(&String, &u32)>, info_lines: &mut Vec<Line<'_>>) {
    for i in 0..5 {
        if let Some(mon) = top_five.get(i) {
            info_lines.push(Line::from(format!("{}: {}", mon.0, mon.1)).centered());
        }
    }
}

fn get_block<'a>(title: Title<'a>, instructions: Title<'a>) -> Block<'a> {
    Block::default()
        .title(title.alignment(Alignment::Center))
        .title(
            instructions
                .alignment(Alignment::Center)
                .position(Position::Bottom),
        )
        .borders(Borders::ALL)
        .border_set(border::THICK)
}

fn get_instruction_line() -> Title<'static> {
    Title::from(Line::from(vec![
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
    ]))
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let is_debug = env::args().find(|arg| arg == "debug");

    if is_debug.is_some() {
        if let Some(value) = debug_mode() {
            return value;
        }
    }

    if let Some(window) = Window::all().unwrap().iter().find(encounter::game_exist) {
        let mut terminal = tui::init()?;
        terminal.clear()?;

        let mut app = App::default();

        if let Ok(RunResult::Exit) = app.run(&mut terminal, window) {
            clear_terminal(terminal)?;
            return Ok(());
        }

        clear_terminal(terminal)?;
        Ok(())
    } else {
        panic!("{} game not found", APP_NAME);
    }
}

fn clear_terminal(
    mut terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    tui::restore()?;
    terminal.clear()?;
    Ok(())
}

fn debug_mode() -> Option<Result<(), Box<dyn Error>>> {
    let (exe_path, path) = get_current_working_dir();
    println!("The current directory is {path} exe path {exe_path}",);
    for window in Window::all().unwrap().iter() {
        println!("Window: {:?}", (window.app_name(), window.title()));

        if window.title().to_lowercase() == APP_NAME || window.app_name() == APP_NAME {
            let img = window.capture_image().unwrap();
            let _ = img.save("debug.png");
        }
    }
    Some(Ok(()))
}
