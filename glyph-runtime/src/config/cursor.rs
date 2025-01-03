use mlua::FromLua;
use serde::Deserialize;

#[derive(Debug, Default, Clone)]
pub struct LuaCursorConfig {
    pub style: LuaCursorModeStyle,
}

impl FromLua for LuaCursorConfig {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        let mlua::Value::Table(cursor) = value else {
            return Err(mlua::Error::runtime("cursor must be a table"));
        };

        let style = cursor.get::<mlua::Value>("style")?;
        Ok(match style {
            mlua::Value::String(style) => LuaCursorConfig {
                style: LuaCursorModeStyle::from(style.to_string_lossy()),
            },
            mlua::Value::Table(table) => LuaCursorConfig {
                style: LuaCursorModeStyle {
                    normal: table.get::<String>("normal").unwrap_or("block".into()).into(),
                    insert: table.get::<String>("insert").unwrap_or("block".into()).into(),
                    command: table.get::<String>("command").unwrap_or("block".into()).into(),
                    visual: table.get::<String>("visual").unwrap_or("block".into()).into(),
                },
            },
            _ => return Err(mlua::Error::runtime("cursor style must be a string or table")),
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct LuaCursorModeStyle {
    pub normal: LuaCursorStyle,
    pub insert: LuaCursorStyle,
    pub command: LuaCursorStyle,
    pub visual: LuaCursorStyle,
}

impl LuaCursorModeStyle {
    pub fn block() -> LuaCursorModeStyle {
        LuaCursorModeStyle {
            normal: LuaCursorStyle::Block,
            insert: LuaCursorStyle::Block,
            command: LuaCursorStyle::Block,
            visual: LuaCursorStyle::Block,
        }
    }

    pub fn bar() -> LuaCursorModeStyle {
        LuaCursorModeStyle {
            normal: LuaCursorStyle::SteadyBar,
            insert: LuaCursorStyle::SteadyBar,
            command: LuaCursorStyle::SteadyBar,
            visual: LuaCursorStyle::SteadyBar,
        }
    }
}

impl<S: AsRef<str>> From<S> for LuaCursorModeStyle {
    fn from(value: S) -> Self {
        match value.as_ref() {
            "block" => LuaCursorModeStyle::block(),
            "steady_bar" => LuaCursorModeStyle::bar(),
            _ => LuaCursorModeStyle::block(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LuaCursorStyle {
    #[default]
    Block,
    SteadyBar,
}

impl<S: AsRef<str>> From<S> for LuaCursorStyle {
    fn from(value: S) -> LuaCursorStyle {
        match value.as_ref() {
            "block" => LuaCursorStyle::Block,
            "steady_bar" => LuaCursorStyle::SteadyBar,
            _ => LuaCursorStyle::Block,
        }
    }
}
