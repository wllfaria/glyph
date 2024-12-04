pub mod dirs;

pub type GlyphConfig<'a> = &'a Config;

use std::collections::HashMap;
use std::fmt::Debug;

use dirs::DIRS;
use glyph_core::highlights::HighlightGroup;
use glyph_runtime::RuntimeMessage;
use mlua::{Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

fn yes() -> bool {
    true
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    #[default]
    Block,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CursorConfig {
    #[serde(default)]
    pub style: CursorStyle,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GutterAnchor {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineNumbersConfig {
    #[default]
    Absolute,
    Relative,
    #[serde(rename = "relative_numbered")]
    RelativeNumbered,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Default, Deserialize)]
pub struct GutterConfig {
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub anchor: GutterAnchor,
    #[serde(default)]
    pub line_numbers: LineNumbersConfig,
    #[serde(default)]
    pub sign_column: SignColumnConfig,
}

#[derive(Debug)]
pub struct Config {
    cursor: CursorConfig,
    gutter: GutterConfig,
    pub highlight_groups: HashMap<String, HighlightGroup>,
}

impl Config {
    pub fn load(
        runtime: &Lua,
        _runtime_sender: UnboundedSender<RuntimeMessage>,
        runtime_receiver: &mut UnboundedReceiver<RuntimeMessage>,
    ) -> glyph_runtime::error::Result<Config> {
        let config = DIRS.get().unwrap().config();
        let init = config.join("init.lua");

        let content = std::fs::read_to_string(&init).unwrap();
        if let Err(err) = runtime.load(content).set_name(init.to_string_lossy()).eval::<Value>() {
            todo!("error in lua ----- {err:?}");
        }

        let mut setup_messages = vec![];
        while let Ok(message) = runtime_receiver.try_recv() {
            setup_messages.push(message);
        }

        let highlight_groups = handle_setup_messages(setup_messages);

        let glyph_mod = glyph_runtime::get_or_create_module(runtime, "glyph")?;
        let config = glyph_mod.get::<Table>("config")?;

        let cursor = runtime.from_value::<CursorConfig>(config.get::<Value>("cursor")?)?;
        let gutter = runtime.from_value::<GutterConfig>(config.get::<Value>("gutter")?)?;

        Ok(Config {
            cursor,
            gutter,
            highlight_groups,
        })
    }

    pub fn cursor(&self) -> &CursorConfig {
        &self.cursor
    }

    pub fn gutter(&self) -> &GutterConfig {
        &self.gutter
    }
}

fn handle_setup_messages(messages: Vec<RuntimeMessage>) -> HashMap<String, HighlightGroup> {
    let mut highlight_groups = HashMap::default();

    for message in messages {
        match message {
            RuntimeMessage::UpdateHighlightGroup(name, group) => _ = highlight_groups.insert(name, group),
            RuntimeMessage::Error(error) => println!("{error:?}"),
        };
    }

    highlight_groups
}
