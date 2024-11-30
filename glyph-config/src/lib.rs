pub mod dirs;
mod error;
pub mod lua;

pub type GlyphConfig<'a> = &'a Config;

use std::fmt::Debug;

use dirs::DIRS;
use error::Result;
use mlua::{Lua, LuaSerdeExt, Value};
use serde::Deserialize;

fn yes() -> bool {
    true
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    #[default]
    Block,
}

#[derive(Debug, Default, Deserialize)]
pub struct CursorConfig {
    #[serde(default)]
    style: CursorStyle,
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

#[derive(Debug, Deserialize)]
pub struct Config {
    cursor: CursorConfig,
    gutter: GutterConfig,
    #[serde(skip)]
    lua: Lua,
}

impl Config {
    pub fn load() -> Result<Config> {
        dirs::setup_dirs();
        let config = DIRS.get().unwrap().config();
        let init = config.join("init.lua");

        let lua = lua::setup_lua_runtime()?;

        let content = std::fs::read_to_string(&init).unwrap();
        lua.load(content).set_name(init.to_string_lossy()).eval::<Value>()?;
        let glyph_mod = lua::get_or_create_module(&lua, "glyph")?;

        let config = lua.from_value(glyph_mod.get::<Value>("config")?)?;
        Ok(config)
    }

    pub fn cursor(&self) -> &CursorConfig {
        &self.cursor
    }

    pub fn gutter(&self) -> &GutterConfig {
        &self.gutter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a() {
        Config::load().unwrap();

        panic!();
    }
}
