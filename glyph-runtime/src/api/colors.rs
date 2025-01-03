use glyph_core::highlights::HighlightGroup;
use mlua::{Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::error::Result;
use crate::RuntimeMessage;

fn default_color() -> String {
    "reset".to_string()
}

impl TryFrom<LuaHighlightGroup> for HighlightGroup {
    type Error = glyph_core::color::Error;

    fn try_from(group: LuaHighlightGroup) -> std::result::Result<Self, Self::Error> {
        let fg = group.fg.try_into()?;
        let bg = group.bg.try_into()?;
        let bold = group.bold;
        let italic = group.italic;

        Ok(HighlightGroup { fg, bg, bold, italic })
    }
}

#[derive(Debug, Deserialize)]
pub struct LuaHighlightGroup {
    #[serde(default = "default_color")]
    fg: String,
    #[serde(default = "default_color")]
    bg: String,
    #[serde(default)]
    bold: bool,
    #[serde(default)]
    italic: bool,
}

pub fn setup_colors_api(
    lua: &Lua,
    core: &Table,
    runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
) -> Result<()> {
    core.set(
        "set_hl_group",
        lua.create_function(move |lua: &Lua, args: (String, Table)| set_hl_group(lua, args, runtime_sender.clone()))?,
    )?;

    Ok(())
}

fn set_hl_group(
    lua: &Lua,
    (name, opts): (String, Table),
    runtime_sender: UnboundedSender<RuntimeMessage<'_>>,
) -> mlua::Result<()> {
    let style = lua.from_value::<LuaHighlightGroup>(Value::Table(opts))?;

    match HighlightGroup::try_from(style) {
        Ok(style) => runtime_sender
            .send(RuntimeMessage::UpdateHighlightGroup(name, style))
            .ok(),
        Err(err) => runtime_sender.send(RuntimeMessage::Error(err.to_string())).ok(),
    };

    Ok(())
}
