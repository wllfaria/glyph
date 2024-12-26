--- @type glyph
local glyph = require("glyph")

--- @class glyph.defaults.commands
--- @field setup fun(): nil
local M = {}

function M.setup() end

glyph.api.user_command_create("q", function()
  glyph.api.editor_quit()
end)

glyph.api.user_command_create("q!", function()
  glyph.api.editor_quit({ force = true })
end)

glyph.api.user_command_create("qa!", function()
  glyph.api.editor_quit({ force = true, all = true })
end)

glyph.api.user_command_create("qa", function()
  glyph.api.editor_quit({ all = true })
end)

glyph.api.user_command_create("w", function()
  glyph.api.editor_write()
end)

glyph.api.user_command_create("wq", function()
  glyph.api.editor_write()
  glyph.api.editor_quit()
end)

glyph.api.user_command_create("wq!", function()
  glyph.api.editor_write({ force = true })
  glyph.api.editor_quit({ force = true })
end)

glyph.api.user_command_create("wa", function()
  glyph.api.editor_write({ all = true })
end)

glyph.api.user_command_create("waq", function()
  glyph.api.editor_write({ all = true })
  glyph.api.editor_quit()
end)

glyph.api.user_command_create("waq!", function()
  glyph.api.editor_write({ all = true, force = true })
  glyph.api.editor_quit({ all = true })
end)

glyph.api.user_command_create("e", function(args)
  --- @type string
  local filename = args[1]
  glyph.api.editor_open_file(filename)
end)

return M
