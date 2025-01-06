--- @type glyph
local glyph = require("glyph")

--- tokyodark theme adaptation for glyph
--- @class glyph.defaults.themes.tokyodark
--- @field setup fun(): nil
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
function M.setup()
  -- user interface
  glyph.api.set_hl_group("line_number", { fg = palette.grey })
  glyph.api.set_hl_group("current_line", { fg = palette.fg })
  glyph.api.set_hl_group("background", { bg = palette.bg0 })
  glyph.api.set_hl_group("foreground", { fg = palette.fg })

  glyph.api.set_hl_group("visual", { fg = palette.bg0, bg = palette.fg })

  -- syntax groups
  glyph.api.set_hl_group("string", { fg = palette.yellow })

  glyph.api.set_hl_group("type", { fg = palette.blue })
  glyph.api.set_hl_group("type.builtin", { fg = palette.blue })
  glyph.api.set_hl_group("type.parameter", { fg = palette.purple })
  glyph.api.set_hl_group("type.builtin", { fg = palette.purple })
  glyph.api.set_hl_group("type.enum.variant", { fg = palette.purple })
  glyph.api.set_hl_group("type.enum.variant.builtin", { fg = palette.purple })

  glyph.api.set_hl_group("property", { fg = palette.orange })

  glyph.api.set_hl_group("constant", { fg = palette.orange })
  glyph.api.set_hl_group("constant.builtin", { fg = palette.purple })
  glyph.api.set_hl_group("constant.builtin.boolean", { fg = palette.purple })
  glyph.api.set_hl_group("constant.numeric.integer", { fg = palette.purple })
  glyph.api.set_hl_group("constant.numeric.float", { fg = palette.purple })
  glyph.api.set_hl_group("constant.character", { fg = palette.purple })
  glyph.api.set_hl_group("constant.character.escape", { fg = palette.purple })

  glyph.api.set_hl_group("constructor", { fg = palette.orange })

  glyph.api.set_hl_group("function", { fg = palette.green })
  glyph.api.set_hl_group("function.method", { fg = palette.green })
  glyph.api.set_hl_group("function.macro", { fg = palette.purple })

  glyph.api.set_hl_group("comment", { fg = palette.bg4 })
  glyph.api.set_hl_group("comment.documentation", { fg = palette.bg4 })

  glyph.api.set_hl_group("punctuation.bracket", { fg = palette.fg })
  glyph.api.set_hl_group("punctuation.delimiter", { fg = palette.fg })

  glyph.api.set_hl_group("variable.parameter", { fg = palette.orange })
  glyph.api.set_hl_group("variable.builtin", { fg = palette.purple })
  glyph.api.set_hl_group("variable.other.member", { fg = palette.orange })
  glyph.api.set_hl_group("variable", { fg = palette.fg })

  glyph.api.set_hl_group("label", { fg = palette.purple })

  glyph.api.set_hl_group("keyword", { fg = palette.red })
  glyph.api.set_hl_group("keyword.control.import", { fg = palette.red })
  glyph.api.set_hl_group("keyword.control.repeat", { fg = palette.purple })
  glyph.api.set_hl_group("keyword.control", { fg = palette.purple })
  glyph.api.set_hl_group("keyword.control.conditional", { fg = palette.purple })
  glyph.api.set_hl_group("keyword.control.return", { fg = palette.purple })
  glyph.api.set_hl_group("keyword.operator", { fg = palette.purple })
  glyph.api.set_hl_group("keyword", { fg = palette.purple })
  glyph.api.set_hl_group("keyword.storage.type", { fg = palette.purple })
  glyph.api.set_hl_group("keyword.storage", { fg = palette.red })
  glyph.api.set_hl_group("keyword.function", { fg = palette.purple })
  glyph.api.set_hl_group("keyword.special", { fg = palette.purple })
  glyph.api.set_hl_group("keyword.storage.modifier.mut", { fg = palette.purple })
  glyph.api.set_hl_group("keyword.storage.modifier.ref", { fg = palette.purple })
  glyph.api.set_hl_group("keyword.storage.modifier", { fg = palette.purple })

  glyph.api.set_hl_group("escape", { fg = palette.purple })
  glyph.api.set_hl_group("attribute", { fg = palette.red })
  glyph.api.set_hl_group("operator", { fg = palette.red })

  glyph.api.set_hl_group("special", { fg = palette.purple })
  glyph.api.set_hl_group("namespace", { fg = palette.red })
end

return M
