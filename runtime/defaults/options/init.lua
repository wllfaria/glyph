--- @type glyph
local glyph = require("glyph")

--- @class glyph.defaults.options
--- @field scroll_offset number
--- @field cursor glyph.options.cursor
--- @field gutter glyph.options.gutter
--- @field statusline glyph.options.statusline
local M = {}

M.scroll_offset = 8

M.cursor = {
  style = "block",
}

M.gutter = {
  enabled = true,
  anchor = "left",
  line_numbers = "absolute",
  sign_column = "all",
}

--- @return string
local function format_mode()
  local mode = glyph.api.editor_get_mode()
  return " " .. mode:upper() .. " "
end

--- @return string
local function format_cursor()
  local window = glyph.api.window_get_active()
  local ok, cursor = glyph.api.window_get_cursor(window)
  if not ok then
    return " 0:0 "
  end
  return " " .. cursor.y .. ":" .. cursor.x .. " "
end

local function cursor_percentage()
  local labels = {
    top = " TOP ",
    bottom = " BOT ",
  }

  local window = glyph.api.window_get_active()
  local ok, cursor = glyph.api.window_get_cursor(window)
  if not ok then
    return labels.top
  end
  local document = glyph.api.document_get_active()
  local ok, lines = glyph.api.document_get_line_count(document)

  if not ok then
    return labels.top
  end

  local percentage = math.floor((cursor.y / lines * 100) + 0.5)

  if percentage == 0 then
    return labels.top
  elseif percentage == 100 then
    return labels.bottom
  else
    return " " .. percentage .. "% "
  end
end

M.statusline = {
  left = {
    { content = format_mode, style = { fg = "#11121D", bg = "#95c561" } },
    { content = " TODO: filename_here.rs ", style = { fg = "#98C379", bg = "#1A1B2A" } },
  },
  right = {
    { content = cursor_percentage, style = { fg = "#11121D", bg = "#95c561" } },
    { content = format_cursor, style = { fg = "#11121D", bg = "#95c561" } },
  },
}

return M
