pub mod colors;
pub mod command;
pub mod document;
pub mod editor;
pub mod error;
pub mod keymap;
pub mod statusline;
pub mod window;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use colors::setup_colors_api;
use command::setup_command_api;
use document::setup_document_api;
use editor::{setup_editor_api, QuitOptions, WriteOptions};
use error::{Error, Result};
use glyph_core::cursor::Cursor;
use glyph_core::editor::{Editor, Mode};
use glyph_core::highlights::HighlightGroup;
use glyph_core::window::WindowId;
use keymap::{setup_keymap_api, LuaKeymap};
use mlua::{Function, Lua, Table, Value};
use parking_lot::RwLock;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::Sender;
use window::setup_window_api;

#[derive(Debug)]
pub enum RuntimeQuery {
    EditorMode(Sender<Mode>),
}

#[derive(Debug)]
pub enum RuntimeMessage<'msg> {
    UpdateHighlightGroup(String, HighlightGroup),
    SetKeymap(LuaKeymap<'msg>),
    UserCommandCreate(String, Function),
    Error(String),
    Quit(QuitOptions),
    Write(WriteOptions),
    OpenFile(String),
}

pub fn setup_lua_runtime(
    config_dir: &Path,
    runtime_sender: UnboundedSender<RuntimeMessage<'static>>,
    context: Arc<RwLock<GlyphContext>>,
) -> Result<Lua> {
    let lua = Lua::new();
    let globals = lua.globals();
    let glyph = get_or_create_module(&lua, "glyph")?;

    let core = lua.create_table()?;
    setup_colors_api(&lua, &core, runtime_sender.clone())?;
    setup_keymap_api(&lua, &core, runtime_sender.clone())?;
    setup_editor_api(&lua, &core, runtime_sender.clone(), context.clone())?;
    setup_window_api(&lua, &core, runtime_sender.clone(), context.clone())?;
    setup_document_api(&lua, &core, runtime_sender.clone(), context.clone())?;
    setup_command_api(&lua, &core, runtime_sender.clone(), context.clone())?;
    glyph.set("_core", core)?;

    let package = globals.get::<Table>("package")?;
    let package_path = package.get::<String>("path")?;
    let mut path_list = package_path.split(";").map(|p| p.to_owned()).collect::<Vec<_>>();

    let prefix_path = |paths: &mut Vec<String>, path: &Path| {
        paths.insert(0, format!("{}/?.lua", path.display()));
        paths.insert(1, format!("{}/?/init.lua", path.display()));
    };

    // #[cfg(debug_assertions)]
    {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = workspace.parent().unwrap();
        let runtime = root.join("runtime");
        prefix_path(&mut path_list, &runtime);
    }

    prefix_path(&mut path_list, config_dir);

    package.set("path", path_list.join(";"))?;

    let runtime = include_str!("../../runtime/init.lua");
    lua.load(runtime).exec()?;

    Ok(lua)
}

#[derive(Debug)]
pub struct GlyphContext {
    pub editor: Arc<RwLock<Editor>>,
    pub cursors: Arc<RwLock<BTreeMap<WindowId, Cursor>>>,
}

pub fn get_or_create_module(lua: &Lua, name: &str) -> Result<Table> {
    let globals = lua.globals();
    let package = globals.get::<Table>("package")?;
    let loaded = package.get::<Table>("loaded")?;

    let module = loaded.get(name)?;
    match module {
        Value::Nil => {
            let module = lua.create_table()?;
            loaded.set(name, module.clone())?;
            Ok(module)
        }
        Value::Table(table) => Ok(table),
        other => Err(Error::ModuleRegister(name.to_string(), other.type_name())),
    }
}
