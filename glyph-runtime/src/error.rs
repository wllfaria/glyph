use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot register module {0} as package.loaded.{0} is already set to a value of type {1}")]
    ModuleRegister(String, &'static str),
    #[error("{0}")]
    Lua(#[from] mlua::Error),
}
