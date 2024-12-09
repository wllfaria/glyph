local glyph = require("glyph")

--- Neovim's keymap set for glyph
--- @class glyph.defaults.keymap.neovim
--- @field load fun():nil
local M = {}

--- load neovim keymap set
function M.load()
  glyph.api.set_keymap("n", "h", "move_left", { description = "Move the cursor left" })
  glyph.api.set_keymap("n", "j", "move_down", { description = "Move the cursor down" })
  glyph.api.set_keymap("n", "k", "move_up", { description = "Move the cursor up" })
  glyph.api.set_keymap("n", "l", "move_right", { description = "Move the cursor right" })
end

return M
