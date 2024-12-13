--- @type glyph
local glyph = require("glyph")

--- tokyodark theme adaptation for glyph
--- @class glyph.defaults.themes.tokyodark
local M = {}

local palette = {
  black = "#06080A",
  bg0 = "#11121D",
  bg1 = "#1A1B2A",
  bg2 = "#212234",
  bg3 = "#353945",
  bg4 = "#4A5057",
  bg5 = "#282C34",
  bg_red = "#FE6D85",
  bg_green = "#98C379",
  bg_blue = "#9FBBF3",
  diff_red = "#773440",
  diff_green = "#587738",
  diff_blue = "#2A3A5A",
  diff_add = "#1E2326",
  diff_change = "#262B3D",
  diff_delete = "#281B27",
  diff_text = "#1C4474",
  fg = "#A0A8CD",
  red = "#EE6D85",
  orange = "#F6955B",
  yellow = "#D7A65F",
  green = "#95C561",
  blue = "#7199EE",
  cyan = "#38A89D",
  purple = "#A485DD",
  grey = "#4A5057",
}

--- load tokyodark theme
function M.load()
  -- user interface
  glyph.api.set_hl_group("line_number", { fg = palette.grey })
  glyph.api.set_hl_group("current_line", { fg = palette.fg })
  glyph.api.set_hl_group("background", { bg = palette.bg0 })
  glyph.api.set_hl_group("foreground", { fg = palette.fg })

  -- syntax groups
  glyph.api.set_hl_group("string", { fg = palette.yellow })
  glyph.api.set_hl_group("type", { fg = palette.blue })
  glyph.api.set_hl_group("type.builtin", { fg = palette.blue })
  glyph.api.set_hl_group("property", { fg = palette.orange })
  glyph.api.set_hl_group("constant", { fg = palette.orange })
  glyph.api.set_hl_group("constructor", { fg = palette.orange })
  glyph.api.set_hl_group("function", { fg = palette.green })
  glyph.api.set_hl_group("function.method", { fg = palette.green })
  glyph.api.set_hl_group("function.macro", { fg = palette.purple })
  glyph.api.set_hl_group("comment", { fg = palette.bg4 })
  glyph.api.set_hl_group("comment.documentation", { fg = palette.bg4 })
  glyph.api.set_hl_group("punctuation.bracket", { fg = palette.fg })
  glyph.api.set_hl_group("punctuation.delimiter", { fg = palette.fg })
  glyph.api.set_hl_group("variable.parameter", { fg = palette.orange })
  glyph.api.set_hl_group("label", { fg = palette.purple })
  glyph.api.set_hl_group("keyword", { fg = palette.red })
  glyph.api.set_hl_group("variable.builtin", { fg = palette.purple })
  glyph.api.set_hl_group("string", { fg = palette.yellow })
  glyph.api.set_hl_group("constant.builtin", { fg = palette.purple })
  glyph.api.set_hl_group("escape", { fg = palette.purple })
  glyph.api.set_hl_group("attribute", { fg = palette.red })
  glyph.api.set_hl_group("operator", { fg = palette.red })
end

return M
