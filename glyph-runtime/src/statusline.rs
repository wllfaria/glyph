use glyph_core::highlights::HighlightGroup;
use mlua::{FromLua, Function, Lua, LuaSerdeExt, Number, Table, Value};

use crate::colors::LuaHighlightGroup;

#[derive(Debug, Default)]
pub struct StatuslineConfig {
    pub left: Vec<StatuslineSection>,
    pub right: Vec<StatuslineSection>,
}

#[derive(Debug)]
pub enum StatuslineStyle {
    HighlightGroup(HighlightGroup),
    Named(String),
}

#[derive(Debug)]
pub struct StatuslineSection {
    pub content: StatuslineContent,
    pub style: StatuslineStyle,
}

#[derive(Debug)]
pub enum StatuslineContent {
    Immediate(String),
    Dynamic(Function),
}

impl FromLua for StatuslineConfig {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let Value::Table(statusline) = value else {
            return Err(mlua::Error::runtime("statusline must be a table"));
        };

        let left_val = statusline.get::<Table>("left")?;
        let right_val = statusline.get::<Table>("right")?;

        let mut left = vec![];
        let mut right = vec![];

        for pair in left_val.pairs::<Number, Value>() {
            let (_, value) = pair?;
            let section = StatuslineSection::from_lua(value, lua)?;
            left.push(section);
        }

        for pair in right_val.pairs::<Number, Value>() {
            let (_, value) = pair?;
            let section = StatuslineSection::from_lua(value, lua)?;
            right.push(section);
        }

        Ok(StatuslineConfig { left, right })
    }
}

impl FromLua for StatuslineSection {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let Value::Table(table) = value else {
            return Err(mlua::Error::runtime("statusline section must be a table"));
        };

        let content = StatuslineContent::from_lua(table.get::<Value>("content")?, lua)?;
        let style = StatuslineStyle::from_lua(table.get::<Value>("style")?, lua)?;

        Ok(StatuslineSection { content, style })
    }
}

impl FromLua for StatuslineContent {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::String(inner) => Ok(StatuslineContent::Immediate(inner.to_string_lossy())),
            Value::Function(inner) => Ok(StatuslineContent::Dynamic(inner)),
            _ => Err(mlua::Error::runtime(
                "statusline content can only be a function or string",
            )),
        }
    }
}

impl FromLua for StatuslineStyle {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Table(_) => {
                let style = lua.from_value::<LuaHighlightGroup>(value)?;
                let style = HighlightGroup::try_from(style).expect("invalid highlight group");
                Ok(StatuslineStyle::HighlightGroup(style))
            }
            Value::String(inner) => Ok(StatuslineStyle::Named(inner.to_string_lossy())),
            _ => Err(mlua::Error::runtime(
                "statusline style can only be a string or highlight group",
            )),
        }
    }
}
