--- @type glyph
local glyph = require("glyph")

--- @class glyph.api.document
--- @field document_is_valid fun(document: integer): boolean
--- @field document_get_active fun(): integer
--- @field document_get_line_count fun(document: integer): integer
--- @field document_get_filepath fun(document: integer): string
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
--- @return integer
function M.document_get_line_count(document)
  return glyph._core.document_get_line_count(document)
end

--- @param document integer
--- @return string
function M.document_get_filepath(document)
  return glyph._core.document_get_filepath(document)
end

return M
