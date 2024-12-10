--- @type glyph
local glyph = require("glyph")

--- @class glyph.defaults.options
--- @field cursor glyph.options.cursor
--- @field gutter glyph.options.gutter
--- @field statusline glyph.options.statusline
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

--- @return string
local function format_mode()
  local mode = glyph.api.get_editor_mode()
  return "[ " .. mode .. " ]"
end

--- @return string
local function format_cursor()
  local window = glyph.api.get_active_window()
  local x, y = glyph.api.get_window_cursor(window)
  return y .. ":" .. x
end

M.statusline = {
  left = {
    {
      content = format_mode,
      style = {
        fg = "#ff0000",
        bg = "#0000ff",
        bold = true,
      },
    },
  },
  right = {
    {
      content = format_cursor,
      style = {
        fg = "#ff0000",
        bg = "#0000ff",
        bold = true,
      },
    },
  },
}

return M
