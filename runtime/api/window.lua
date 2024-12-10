--- @type glyph
local glyph = require("glyph")

--- @class glyph.api.window
--- @field get_active_window fun(): number
--- @field get_window_cursor fun(window: number): boolean, glyph.t.point?
local M = {}

--- @return number
function M.get_active_window()
  return glyph._core.get_active_window()
end

--- @class glyph.t.point
--- @field x number
--- @field y number

--- @param window number
--- @return boolean, glyph.t.point?
function M.get_window_cursor(window)
  if glyph._core.window_is_valid(window) then
    return true, glyph._core.get_window_cursor(window)
  end
  return false, nil
end

return M
