--- @type glyph
local glyph = require("glyph")

--- @class glyph.api.window
--- @field window_get_active fun(): integer
--- @field window_is_valid fun(window: integer): boolean
--- @field window_get_cursor fun(window: integer): boolean, glyph.t.point
local M = {}

--- @return integer
function M.window_get_active()
  return glyph._core.window_get_active()
end

--- @param window integer
--- @return boolean
function M.window_is_valid(window)
  return glyph._core.window_is_valid(window)
end

--- @class glyph.t.point
--- @field x integer
--- @field y integer

--- @param window integer
--- @return boolean, glyph.t.point
function M.window_get_cursor(window)
  if glyph._core.window_is_valid(window) then
    return pcall(glyph._core.window_get_cursor, window)
  end
  return false, {}
end

return M
