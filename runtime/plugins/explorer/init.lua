--- @type glyph
local glyph = require("glyph")

--- @class glyph.plugins.explorer
--- @field open fun()
local M = {}
M.__index = M

function M.open()
  local document = glyph.api.document_create()
  local window = glyph.api.window_create(document, {
    enter = true,
  })
end

return M
