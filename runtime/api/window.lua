--- @type glyph
local glyph = require("glyph")

--- @class glyph.t.window_create_opts
--- @field enter boolean

--- @class glyph.api.window
--- @field window_get_active fun(): integer
--- @field window_is_valid fun(window: integer): boolean
--- @field window_get_cursor fun(window: integer): glyph.t.point
--- @field window_create fun(document: integer, opts: glyph.t.window_create_opts): integer
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

--- @param window integer
--- @return glyph.t.point
function M.window_get_cursor(window)
  return glyph._core.window_get_cursor(window)
end

--- @param document integer
--- @param opts? glyph.t.window_create_opts
--- @return integer
function M.window_create(document, opts)
  opts = opts or {}

  if not glyph.api.document_is_valid(document) then
    error("invalid document provided")
  end

  glyph.u.table_validate(opts, {
    enter = { "boolean", "nil" },
  })

  return glyph._core.window_create(document, opts)
end

return M
