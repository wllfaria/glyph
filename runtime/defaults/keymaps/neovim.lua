local glyph = require("glyph")

--- Neovim's keymap set for glyph
--- @class glyph.keymap.neovim
local M = {}

--- load neovim keymap set
function M.load()
  glyph.api.set_keymap("n", "j", "move_down", { description = "Move the cursor down" })
  glyph.api.set_keymap("n", "k", "move_up", { description = "Move the cursor up" })
end

return M
