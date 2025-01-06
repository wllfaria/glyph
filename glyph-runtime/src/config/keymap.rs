use glyph_core::command::MappableCommand;
use glyph_core::editor::Mode;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct LuaKeymapOptions {
    #[serde(default)]
    pub description: String,
}

#[derive(Debug)]
pub enum LuaMappableCommand<'key> {
    Borrowed(&'key MappableCommand),
    Owned(MappableCommand),
}

#[derive(Debug)]
pub struct LuaKeymapConfig<'key> {
    pub mode: Mode,
    pub keys: String,
    pub command: LuaMappableCommand<'key>,
    pub options: LuaKeymapOptions,
}
