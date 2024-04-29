mod config;
mod default_config;

pub use config::{get_config_dir, load_config, setup_logger, APP_NAME, THEMES_DIR};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

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

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Command,
    Search,
}

impl Default for &Mode {
    fn default() -> Self {
        &Mode::Normal
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Search => f.write_str("SEARCH"),
            Self::Insert => f.write_str("INSERT"),
            Self::Normal => f.write_str("NORMAL"),
            Self::Command => f.write_str("COMMAND"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    SaveAllBuffers,
    DeleteUntilEOL,
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

    ShowMessage(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum KeyAction {
    Simple(Action),
    Multiple(Vec<Action>),
    Complex(HashMap<String, KeyAction>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde()]
    pub keys: Keys,
    pub theme: String,
    pub gutter_width: usize,
    pub line_numbers: LineNumbers,
    pub background: EditorBackground,
    pub empty_line_char: char,
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Self {
        Self {
            keys: Keys::default(),
            theme: String::from("default"),
            gutter_width: 6,
            line_numbers: LineNumbers::Relative,
            background: EditorBackground::Light,
            empty_line_char: ' ',
        }
    }
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
