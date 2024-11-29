use std::path::Path;

use mlua::{Lua, Table, Value};

use crate::dirs::DIRS;
use crate::error::{Error, Result};

pub fn setup_lua_runtime() -> Result<Lua> {
    let lua = Lua::new();
    let globals = lua.globals();

    let glyph_mod = get_or_create_module(&lua, "glyph")?;
    setup_module_tables(&lua, &glyph_mod)?;

    let package = globals.get::<Table>("package")?;
    let package_path = package.get::<String>("path")?;
    let mut path_list = package_path.split(";").map(|p| p.to_owned()).collect::<Vec<_>>();

    let prefix_path = |paths: &mut Vec<String>, path: &Path| {
        paths.insert(0, format!("{}/?.lua", path.display()));
        paths.insert(1, format!("{}/?/init.lua", path.display()));
    };
    prefix_path(&mut path_list, DIRS.get().unwrap().config());

    package.set("path", path_list.join(";"))?;

    Ok(lua)
}

pub fn get_or_create_module(lua: &Lua, name: &str) -> Result<Table> {
    let globals = lua.globals();
    let package: Table = globals.get("package")?;
    let loaded: Table = package.get("loaded")?;

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

pub fn setup_module_tables(lua: &Lua, module: &Table) -> Result<()> {
    module.set("config", lua.create_table()?)?;

    let config = module.get::<Table>("config")?;
    config.set("cursor", lua.create_table()?)?;
    config.set("gutter", lua.create_table()?)?;

    Ok(())
}
