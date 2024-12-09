mod colors;
pub mod error;
pub mod keymap;

use std::path::{Path, PathBuf};

use colors::setup_colors_api;
use error::{Error, Result};
use glyph_core::highlights::HighlightGroup;
use keymap::{setup_keymap_api, LuaKeymap};
use mlua::{Lua, Table, Value};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub enum RuntimeMessage {
    UpdateHighlightGroup(String, HighlightGroup),
    SetKeymap(LuaKeymap),
    Error(String),
}

pub fn setup_lua_runtime(config_dir: &Path, runtime_sender: UnboundedSender<RuntimeMessage>) -> Result<Lua> {
    let lua = Lua::new();
    let globals = lua.globals();
    let glyph = get_or_create_module(&lua, "glyph")?;

    let core = lua.create_table()?;
    setup_colors_api(&lua, &core, runtime_sender.clone())?;
    setup_keymap_api(&lua, &core, runtime_sender.clone())?;
    glyph.set("_core", core)?;

    let package = globals.get::<Table>("package")?;
    let package_path = package.get::<String>("path")?;
    let mut path_list = package_path.split(";").map(|p| p.to_owned()).collect::<Vec<_>>();

    let prefix_path = |paths: &mut Vec<String>, path: &Path| {
        paths.insert(0, format!("{}/?.lua", path.display()));
        paths.insert(1, format!("{}/?/init.lua", path.display()));
    };

    #[cfg(debug_assertions)]
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
