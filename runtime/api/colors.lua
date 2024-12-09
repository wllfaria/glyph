--- @type glyph
local glyph = require("glyph")

--- @class glyph.colors.hl_group_opts
--- @field fg? string
--- @field bg? string
--- @field bold? boolean

--- @class glyph.api.colors
--- @field set_hl_group fun(name: string, opts: glyph.colors.hl_group_opts)
local M = {}

--- @param name string
--- @param opts glyph.colors.hl_group_opts
function M.set_hl_group(name, opts)
  opts = opts or {}

  glyph.u.table_validate(opts, {
    fg = { "string", "nil" },
    bg = { "string", "nil" },
    bold = { "boolean", "nil" },
  })

  glyph._core.set_hl_group(name, opts)
end

return M
