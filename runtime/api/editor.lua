--- @type glyph
local glyph = require("glyph")

--- @class glyph.api.editor
--- @field get_editor_mode fun(): string
local M = {}

--- gets the current active editor mode in lowercer string format
---
--- @return string
function M.get_editor_mode()
  return glyph._core.get_editor_mode()
end

return M
