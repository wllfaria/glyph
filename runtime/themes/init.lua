local M = {}

local default_themes = {
  default = "default",
}

function M.load_theme(name)
  if name == default_themes.default then
    require("themes.default")
  end
end

return M
