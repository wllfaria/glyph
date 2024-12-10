--- @type glyph
local glyph = require("glyph")

glyph.options.statusline = {
  left = {
    { content = glyph.api.get_editor_mode, style = "some_group" },
    { content = "some_string", style = { fg = "#ffffff", bg = "#ffffff" } },
  },
  right = {},
}
