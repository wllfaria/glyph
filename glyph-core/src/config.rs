use std::collections::HashMap;

use glyph_trie::Trie;

use crate::command::{Context, MappableCommand};
use crate::editor::Mode;
use crate::highlights::HighlightGroup;

pub type GlyphConfig<'a> = &'a Config<'a>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CursorStyle {
    #[default]
    Block,
    SteadyBar,
}

#[derive(Debug, Default, Clone)]
pub struct CursorConfig {
    pub style: CursorModeStyle,
}

#[derive(Debug, Default, Clone)]
pub struct CursorModeStyle {
    pub normal: CursorStyle,
    pub insert: CursorStyle,
    pub command: CursorStyle,
    pub visual: CursorStyle,
}

impl CursorModeStyle {
    pub fn block() -> CursorModeStyle {
        CursorModeStyle {
            normal: CursorStyle::Block,
            insert: CursorStyle::Block,
            command: CursorStyle::Block,
            visual: CursorStyle::Block,
        }
    }

    pub fn bar() -> CursorModeStyle {
        CursorModeStyle {
            normal: CursorStyle::SteadyBar,
            insert: CursorStyle::SteadyBar,
            command: CursorStyle::SteadyBar,
            visual: CursorStyle::SteadyBar,
        }
    }
}

#[derive(Debug, Default)]
pub enum GutterAnchor {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Default)]
pub enum LineNumbersConfig {
    #[default]
    Absolute,
    Relative,
    RelativeNumbered,
}

#[derive(Debug, Default)]
pub enum SignColumnConfig {
    #[default]
    All,
    None,
}

impl SignColumnConfig {
    pub fn size(&self) -> u16 {
        match self {
            SignColumnConfig::None => 0,
            _ => 1,
        }
    }
}

#[derive(Debug, Default)]
pub struct GutterConfig {
    pub enabled: bool,
    pub anchor: GutterAnchor,
    pub line_numbers: LineNumbersConfig,
    pub sign_column: SignColumnConfig,
}

#[derive(Debug)]
pub struct KeymapOptions {
    pub description: String,
}

#[derive(Debug)]
pub enum MappableCommandConfig<'cmd> {
    Borrowed(&'cmd MappableCommand),
    Owned(MappableCommand),
}

impl MappableCommandConfig<'_> {
    pub fn run(&self, ctx: &mut Context<'_>) {
        match self {
            MappableCommandConfig::Borrowed(MappableCommand::Static { fun, .. }) => fun(ctx),
            MappableCommandConfig::Borrowed(MappableCommand::Dynamic { callback, .. }) => callback(),
            MappableCommandConfig::Owned(MappableCommand::Static { fun, .. }) => fun(ctx),
            MappableCommandConfig::Owned(MappableCommand::Dynamic { callback, .. }) => callback(),
        }
    }
}

#[derive(Debug)]
pub struct KeymapConfig<'cfg> {
    pub mode: Mode,
    pub command: MappableCommandConfig<'cfg>,
    pub options: KeymapOptions,
}

#[derive(Debug, Default)]
pub struct StatuslineConfig {
    pub left: Vec<StatuslineSection>,
    pub right: Vec<StatuslineSection>,
}

#[derive(Debug)]
pub struct StatuslineSection {
    pub content: StatuslineContent,
    pub style: StatuslineStyle,
}

#[derive(Debug)]
pub enum StatuslineStyle {
    HighlightGroup(HighlightGroup),
    Named(String),
}

pub enum StatuslineContent {
    Immediate(String),
    Dynamic(Box<dyn Fn() -> String>),
}

impl std::fmt::Debug for StatuslineContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatuslineContent::Immediate(name) => f.write_fmt(format_args!("Immediate({name})")),
            StatuslineContent::Dynamic(_) => f.write_str("Dynamic(dyn Fn())"),
        }
    }
}

pub trait UserCommand {
    fn call(&self, args: Vec<String>) -> Result<(), String>;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub enum ModeMaps {
    #[default]
    Normal,
    Insert,
    Command,
    Visual,
}

impl From<Mode> for ModeMaps {
    fn from(mode: Mode) -> ModeMaps {
        match mode {
            Mode::Normal => ModeMaps::Normal,
            Mode::Insert => ModeMaps::Insert,
            Mode::Command => ModeMaps::Command,
            Mode::Visual | Mode::VisualLine | Mode::VisualBlock => ModeMaps::Visual,
        }
    }
}

#[derive(Default)]
pub struct Config<'cfg> {
    pub cursor: CursorConfig,
    pub gutter: GutterConfig,
    pub user_commands: HashMap<String, Box<dyn UserCommand>>,
    pub scroll_offset: usize,
    pub statusline: StatuslineConfig,
    pub highlight_groups: HashMap<String, HighlightGroup>,
    pub keymaps: HashMap<ModeMaps, Trie<KeymapConfig<'cfg>>>,
}

impl std::fmt::Debug for Config<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("cursor", &self.cursor)
            .field("gutter", &self.gutter)
            .field("scroll_offset", &self.scroll_offset)
            .field("statusline", &self.statusline)
            .field("highlight_groups", &self.highlight_groups)
            .field("keymaps", &self.keymaps)
            .field("user_commands", &self.user_commands.keys().collect::<Vec<_>>())
            .finish()
    }
}
