--- @type glyph
local glyph = require("glyph")

--- Neovim's keymap set for glyph
--- @class glyph.defaults.keymap.neovim
--- @field setup fun():nil
local M = {}

--- load neovim keymap set
function M.setup()
  glyph.api.keymap_set("n", "h", "move_left", { description = "Move the cursor left" })
  glyph.api.keymap_set("n", "j", "move_down", { description = "Move the cursor down" })
  glyph.api.keymap_set("n", "k", "move_up", { description = "Move the cursor up" })
  glyph.api.keymap_set("n", "l", "move_right", { description = "Move the cursor right" })
  glyph.api.keymap_set("n", "dd", "delete_line", { descripton = "Deletes the line under cursor" })
  glyph.api.keymap_set("n", "G", "move_to_eof", { descripton = "Moves cursor to the end of file" })
  glyph.api.keymap_set("n", "gg", "move_to_sof", { descripton = "Moves cursor to start of file" })
  glyph.api.keymap_set("n", "0", "move_to_sol", { descripton = "Moves cursor to start of line" })
  glyph.api.keymap_set("n", "$", "move_to_eol", { descripton = "Moves cursor to end of line" })
  glyph.api.keymap_set("n", "<c-d>", "page_down", { descripton = "Moves cursor half screen down" })
  glyph.api.keymap_set("n", "<c-u>", "page_up", { descripton = "Moves cursor half screen up" })
  glyph.api.keymap_set("n", "i", "insert_mode", { description = "Change into insert mode" })
  glyph.api.keymap_set("n", ":", "command_mode", { description = "Change into command mode" })

  glyph.api.keymap_set("c", "<c-c>", "normal_mode", { description = "Changes into normal mode" })
  glyph.api.keymap_set("c", "<esc>", "normal_mode", { description = "Changes into normal mode" })

  glyph.api.keymap_set("i", "jk", "normal_mode", { description = "Changes into normal mode" })
  glyph.api.keymap_set("i", "<esc>", "normal_mode", { description = "Changes into normal mode" })
  glyph.api.keymap_set("i", "<c-c>", "normal_mode", { description = "Changes into normal mode" })
end

return M
