use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::editor::Mode;

const fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LineNumbers {
    Absolute,
    Relative,
    RelativeNumbered,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EditorBackground {
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keys: Keys,
    pub theme: String,
    pub log_file: Option<String>,
    pub mouse_scroll_lines: Option<usize>,
    pub gutter_width: usize,
    pub line_numbers: LineNumbers,
    pub background: EditorBackground,
    pub empty_line_char: char,
    #[serde(default = "default_true")]
    pub show_diagnostics: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Action {
    EnterMode(Mode),
    Quit,
    Undo,
    InsertLine,
    InsertLineBelow,
    PasteBelow,
    FindNext,
    FindPrevious,
    CenterLine,
    InsertTab,
    InsertChar(char),

    NextWord,
    PreviousWord,
    MoveLeft,
    MoveDown,
    MoveUp,
    MoveRight,
    MoveToBottom,
    MoveToTop,
    MoveToLineEnd,
    MoveToLineStart,
    PageDown,
    PageUp,

    DeleteCurrentChar,
    DeleteBack,
    DeleteWord,
    DeleteLine,
    DeletePreviousChar,

    GoToDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Keys {
    #[serde(default)]
    pub normal: HashMap<String, KeyAction>,
    #[serde(default)]
    pub insert: HashMap<String, KeyAction>,
    #[serde(default)]
    pub command: HashMap<String, KeyAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum KeyAction {
    Single(Action),
    Multiple(Vec<Action>),
    Nested(HashMap<String, KeyAction>),
}

fn default_normal() -> HashMap<String, KeyAction> {
    HashMap::from([
        ("n".to_string(), KeyAction::Single(Action::FindNext)),
        ("N".to_string(), KeyAction::Single(Action::FindPrevious)),
        ("w".to_string(), KeyAction::Single(Action::NextWord)),
        ("b".to_string(), KeyAction::Single(Action::PreviousWord)),
        ("p".to_string(), KeyAction::Single(Action::PasteBelow)),
        (
            "a".to_string(),
            KeyAction::Multiple(vec![Action::EnterMode(Mode::Insert), Action::MoveRight]),
        ),
        (
            "A".to_string(),
            KeyAction::Multiple(vec![Action::EnterMode(Mode::Insert), Action::MoveToLineEnd]),
        ),
        (
            "O".to_string(),
            KeyAction::Multiple(vec![Action::InsertLine, Action::EnterMode(Mode::Insert)]),
        ),
        (
            "o".to_string(),
            KeyAction::Multiple(vec![
                Action::InsertLineBelow,
                Action::EnterMode(Mode::Insert),
            ]),
        ),
        ("q".to_string(), KeyAction::Single(Action::Quit)),
        ("u".to_string(), KeyAction::Single(Action::Undo)),
        ("k".to_string(), KeyAction::Single(Action::MoveUp)),
        ("Up".to_string(), KeyAction::Single(Action::MoveUp)),
        ("j".to_string(), KeyAction::Single(Action::MoveDown)),
        ("h".to_string(), KeyAction::Single(Action::MoveLeft)),
        ("l".to_string(), KeyAction::Single(Action::MoveRight)),
        ("G".to_string(), KeyAction::Single(Action::MoveToBottom)),
        ("$".to_string(), KeyAction::Single(Action::MoveToLineEnd)),
        ("0".to_string(), KeyAction::Single(Action::MoveToLineStart)),
        (
            "x".to_string(),
            KeyAction::Single(Action::DeleteCurrentChar),
        ),
        (
            "/".to_string(),
            KeyAction::Single(Action::EnterMode(Mode::Search)),
        ),
        (
            "i".to_string(),
            KeyAction::Single(Action::EnterMode(Mode::Insert)),
        ),
        (
            "I".to_string(),
            KeyAction::Multiple(vec![
                Action::MoveToLineStart,
                Action::EnterMode(Mode::Insert),
            ]),
        ),
        (
            ":".to_string(),
            KeyAction::Single(Action::EnterMode(Mode::Command)),
        ),
        ("Down".to_string(), KeyAction::Single(Action::MoveDown)),
        ("Left".to_string(), KeyAction::Single(Action::MoveLeft)),
        ("Right".to_string(), KeyAction::Single(Action::MoveRight)),
        ("C-d".to_string(), KeyAction::Single(Action::PageDown)),
        ("C-u".to_string(), KeyAction::Single(Action::PageUp)),
        ("End".to_string(), KeyAction::Single(Action::MoveToLineEnd)),
        (
            "Home".to_string(),
            KeyAction::Single(Action::MoveToLineStart),
        ),
        (
            "g".to_string(),
            KeyAction::Nested(HashMap::from([
                ("g".to_string(), KeyAction::Single(Action::MoveToTop)),
                ("d".to_string(), KeyAction::Single(Action::GoToDefinition)),
            ])),
        ),
        (
            "d".to_string(),
            KeyAction::Nested(HashMap::from([
                ("b".to_string(), KeyAction::Single(Action::DeleteBack)),
                ("w".to_string(), KeyAction::Single(Action::DeleteWord)),
                ("d".to_string(), KeyAction::Single(Action::DeleteLine)),
            ])),
        ),
        (
            "z".to_string(),
            KeyAction::Nested(HashMap::from([(
                "z".to_string(),
                KeyAction::Single(Action::CenterLine),
            )])),
        ),
    ])
}

fn default_insert() -> HashMap<String, KeyAction> {
    HashMap::from([
        ("Enter".to_string(), KeyAction::Single(Action::InsertLine)),
        (
            "Backspace".to_string(),
            KeyAction::Single(Action::DeletePreviousChar),
        ),
        ("Tab".to_string(), KeyAction::Single(Action::InsertTab)),
        (
            "Esc".to_string(),
            KeyAction::Single(Action::EnterMode(Mode::Normal)),
        ),
        (
            "Caps".to_string(),
            KeyAction::Single(Action::EnterMode(Mode::Normal)),
        ),
        (
            "j".to_string(),
            KeyAction::Nested(HashMap::from([(
                "k".to_string(),
                KeyAction::Single(Action::EnterMode(Mode::Normal)),
            )])),
        ),
    ])
}

fn default_command() -> HashMap<String, KeyAction> {
    HashMap::from([
        (
            "Esc".to_string(),
            KeyAction::Single(Action::EnterMode(Mode::Normal)),
        ),
        (
            "C-c".to_string(),
            KeyAction::Single(Action::EnterMode(Mode::Normal)),
        ),
    ])
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "".to_string(),
            log_file: None,
            mouse_scroll_lines: Some(3),
            show_diagnostics: true,
            gutter_width: 6,
            background: EditorBackground::Dark,
            line_numbers: LineNumbers::Absolute,
            empty_line_char: '~',
            keys: Keys {
                normal: default_normal(),
                insert: default_insert(),
                command: default_command(),
            },
        }
    }
}

impl Keys {
    pub fn extend(&mut self, src: Keys) {
        self.normal.extend(src.normal);
        self.insert.extend(src.insert);
        self.command.extend(src.command);
    }
}

impl Config {
    pub fn extend(&mut self, src: Config) {
        self.keys.extend(src.keys);

        if src.background != self.background {
            self.background = src.background;
        }
        if src.line_numbers != self.line_numbers {
            self.line_numbers = src.line_numbers;
        }
        if src.gutter_width != self.gutter_width {
            self.gutter_width = src.gutter_width;
        }
        if src.empty_line_char != self.empty_line_char {
            self.empty_line_char = src.empty_line_char;
        }

        if !src.theme.is_empty() {
            self.theme = src.theme;
        }

        if src.show_diagnostics != self.show_diagnostics {
            self.show_diagnostics = src.show_diagnostics
        }

        if let Some(log_file) = src.log_file {
            self.log_file = Some(log_file);
        }

        if let Some(scrolloff) = src.mouse_scroll_lines {
            self.mouse_scroll_lines = Some(scrolloff);
        }
    }

    pub fn get_path() -> PathBuf {
        let home = dirs::home_dir().unwrap();
        home.join(".config/glyph")
    }

    pub fn themes_path() -> PathBuf {
        let config_path = Config::get_path();
        config_path.join("themes")
    }
}
