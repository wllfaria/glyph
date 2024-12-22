--- @type glyph
local glyph = require("glyph")

--- @class glyph.api.editor
--- @field editor_get_mode fun(): string
--- @field editor_quit fun(opts?: glyph.t.quit_opts)
--- @field editor_write fun(opts?: glyph.t.write_opts)
local M = {}

--- gets the current active editor mode in lowercer string format
---
--- @return string
function M.editor_get_mode()
  return glyph._core.editor_get_mode()
end

--- @param opts? glyph.t.quit_opts
function M.editor_quit(opts)
  opts = opts or {}
  glyph._core.editor_quit(opts)
end

--- @param opts? glyph.t.write_opts
function M.editor_write(opts)
  opts = opts or {}
  glyph._core.editor_write(opts)
end

return M
