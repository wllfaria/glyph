local glyph = require("glyph")

local M = {}

function M.set_hl_group(name, opts)
  glyph.table_validate(opts, {
    fg = { "string", "nil" },
    bg = { "string", "nil" },
    bold = { "boolean", "nil" },
  })

  glyph.api.set_hl_group(name, opts)
end

return M
