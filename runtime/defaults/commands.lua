--- @type glyph
local glyph = require("glyph")

--- @class glyph.defaults.commands
--- @field setup fun(): nil
local M = {}

function M.setup() end

glyph.api.user_command_create("q", function()
  glyph._core.editor_quit()
end)

glyph.api.user_command_create("q!", function()
  glyph._core.editor_quit({ force = true })
end)

glyph.api.user_command_create("qa!", function()
  glyph._core.editor_quit({ force = true, all = true })
end)

glyph.api.user_command_create("qa", function()
  glyph._core.editor_quit({ all = true })
end)

glyph.api.user_command_create("w", function()
  glyph._core.editor_write()
end)

glyph.api.user_command_create("wq", function()
  glyph._core.editor_write()
  glyph._core.editor_quit()
end)

glyph.api.user_command_create("wq!", function()
  glyph._core.editor_write({ force = true })
  glyph._core.editor_quit({ force = true })
end)

glyph.api.user_command_create("wa", function()
  glyph._core.editor_write({ all = true })
end)

glyph.api.user_command_create("waq", function()
  glyph._core.editor_write({ all = true })
  glyph._core.editor_quit()
end)

glyph.api.user_command_create("waq!", function()
  glyph._core.editor_write({ all = true, force = true })
  glyph._core.editor_quit({ all = true })
end)

return M
