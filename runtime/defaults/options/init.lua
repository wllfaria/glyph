--- @class glyph.defaults.options
--- @field cursor glyph.options.cursor
--- @field gutter glyph.options.gutter
local M = {}

M.cursor = {
  style = "block",
}

M.gutter = {
  enabled = true,
  anchor = "left",
  line_numbers = "relative_numbered",
  sign_column = "all",
}

return M
