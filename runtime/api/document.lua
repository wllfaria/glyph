--- @type glyph
local glyph = require("glyph")

--- @class glyph.api.document
--- @field document_is_valid fun(document: integer): boolean
--- @field document_get_active fun(): integer
--- @field document_get_line_count fun(document: integer): boolean, integer
local M = {}

--- @param document integer
--- @return boolean
function M.document_is_valid(document)
  return glyph._core.document_is_valid(document)
end

--- @return integer
function M.document_get_active()
  return glyph._core.document_get_active()
end

--- @param document integer
--- @return boolean, integer
function M.document_get_line_count(document)
  if glyph._core.document_is_valid(document) then
    return pcall(glyph._core.document_get_line_count, document)
  end
  return false, 0
end

return M
