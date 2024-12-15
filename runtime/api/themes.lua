--- @class glyph.api.themes
--- @field load_theme fun(name: string): nil
local M = {}

function M.load_theme(name)
  local default_themes = require("defaults.themes")
  if default_themes[name] then
    default_themes[name].setup()
  end
end

return M
