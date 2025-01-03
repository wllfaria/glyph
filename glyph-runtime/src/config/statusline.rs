use glyph_core::highlights::HighlightGroup;
use mlua::{FromLua, LuaSerdeExt};

use crate::api::colors::LuaHighlightGroup;

#[derive(Debug, Default)]
pub struct LuaStatuslineConfig {
    pub left: Vec<LuaStatuslineSection>,
    pub right: Vec<LuaStatuslineSection>,
}

#[derive(Debug)]
pub struct LuaStatuslineSection {
    pub content: LuaStatuslineContent,
    pub style: LuaStatuslineStyle,
}

#[derive(Debug)]
pub enum LuaStatuslineStyle {
    HighlightGroup(HighlightGroup),
    Named(String),
}

#[derive(Debug)]
pub enum LuaStatuslineContent {
    Immediate(String),
    Dynamic(mlua::Function),
}

impl FromLua for LuaStatuslineConfig {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let mlua::Value::Table(statusline) = value else {
            return Err(mlua::Error::runtime("statusline must be a table"));
        };

        let left_val = statusline.get::<mlua::Table>("left")?;
        let right_val = statusline.get::<mlua::Table>("right")?;

        let mut left = vec![];
        let mut right = vec![];

        for pair in left_val.pairs::<mlua::Number, mlua::Value>() {
            let (_, value) = pair?;
            let section = LuaStatuslineSection::from_lua(value, lua)?;
            left.push(section);
        }

        for pair in right_val.pairs::<mlua::Number, mlua::Value>() {
            let (_, value) = pair?;
            let section = LuaStatuslineSection::from_lua(value, lua)?;
            right.push(section);
        }

        Ok(LuaStatuslineConfig { left, right })
    }
}

impl FromLua for LuaStatuslineSection {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let mlua::Value::Table(table) = value else {
            return Err(mlua::Error::runtime("statusline section must be a table"));
        };

        let content = LuaStatuslineContent::from_lua(table.get::<mlua::Value>("content")?, lua)?;
        let style = LuaStatuslineStyle::from_lua(table.get::<mlua::Value>("style")?, lua)?;

        Ok(LuaStatuslineSection { content, style })
    }
}

impl FromLua for LuaStatuslineContent {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::String(inner) => Ok(LuaStatuslineContent::Immediate(inner.to_string_lossy())),
            mlua::Value::Function(inner) => Ok(LuaStatuslineContent::Dynamic(inner)),
            _ => Err(mlua::Error::runtime(
                "statusline content can only be a function or string",
            )),
        }
    }
}

impl FromLua for LuaStatuslineStyle {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::Table(_) => {
                let style = lua.from_value::<LuaHighlightGroup>(value)?;
                let style = HighlightGroup::try_from(style).expect("invalid highlight group");
                Ok(LuaStatuslineStyle::HighlightGroup(style))
            }
            mlua::Value::String(inner) => Ok(LuaStatuslineStyle::Named(inner.to_string_lossy())),
            _ => Err(mlua::Error::runtime(
                "statusline style can only be a string or highlight group",
            )),
        }
    }
}
