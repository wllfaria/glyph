local glyph = require("glyph")

local M = {}

function M.set_hl_group(name, opts)
  glyph.api.table_validate(opts, {
    fg = { "string", "nil" },
    bg = { "string", "nil" },
    bold = { "boolean", "nil" },
  })

  glyph._core.set_hl_group(name, opts)
end

return M
