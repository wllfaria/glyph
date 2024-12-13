--- @type glyph
local glyph = require("glyph")

--- @class glyph.api.editor
--- @field editor_get_mode fun(): string
local M = {}

--- gets the current active editor mode in lowercer string format
---
--- @return string
function M.editor_get_mode()
  return glyph._core.editor_get_mode()
end

return M
