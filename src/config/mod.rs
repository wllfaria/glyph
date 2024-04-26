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
    #[serde()]
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

#[cfg(test)]
impl Default for Config {
    fn default() -> Self {
        Self {
            keys: Keys::default(),
            theme: String::from("default"),
            log_file: None,
            mouse_scroll_lines: None,
            gutter_width: 6,
            line_numbers: LineNumbers::Relative,
            background: EditorBackground::Light,
            empty_line_char: ' ',
            show_diagnostics: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Action {
    EnterMode(Mode),
    Quit,
    Undo,
    InsertLine,
    InsertLineBelow,
    InsertLineAbove,
    PasteBelow,
    FindNext,
    FindPrevious,
    CenterLine,
    InsertTab,
    InsertChar(char),
    InsertCommand(char),
    ExecuteCommand,
    SaveBuffer,
    DeleteUntilEOL,
    /// Action sent when the terminal is resized,
    /// `Resize(width, height)`
    Resize(u16, u16),
    LoadFile(PathBuf),

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
    Hover,
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
#[serde(untagged)]
pub enum KeyAction {
    Simple(Action),
    Multiple(Vec<Action>),
    Complex(HashMap<String, KeyAction>),
}

impl Config {
    pub fn get_path() -> PathBuf {
        let home = dirs::home_dir().unwrap();
        home.join(".config/glyph")
    }

    pub fn themes_path() -> PathBuf {
        let config_path = Config::get_path();
        config_path.join("themes")
    }
}
