use serde::Deserialize;

fn yes() -> bool {
    true
}

#[derive(Debug, Default, Deserialize)]
pub struct LuaGutterConfig {
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub anchor: LuaGutterAnchor,
    #[serde(default)]
    pub line_numbers: LuaLineNumbersConfig,
    #[serde(default)]
    pub sign_column: LuaSignColumnConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LuaGutterAnchor {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LuaLineNumbersConfig {
    #[default]
    Absolute,
    Relative,
    #[serde(rename = "relative_numbered")]
    RelativeNumbered,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LuaSignColumnConfig {
    #[default]
    All,
    None,
}
