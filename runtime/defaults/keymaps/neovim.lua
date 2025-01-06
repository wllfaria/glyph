--- @type glyph
local glyph = require("glyph")

--- Neovim's keymap set for glyph
--- @class glyph.defaults.keymap.neovim
--- @field setup fun()
local M = {}

--- load neovim keymap set
function M.setup()
  -- stylua: ignore start
  glyph.api.keymap_set({ "n", "v" }, "h", "move_left", { description = "Move the cursor left" })
  glyph.api.keymap_set({ "n", "v" }, "j", "move_down", { description = "Move the cursor down" })
  glyph.api.keymap_set({ "n", "v" }, "k", "move_up", { description = "Move the cursor up" })
  glyph.api.keymap_set({ "n", "v" }, "l", "move_right", { description = "Move the cursor right" })
  glyph.api.keymap_set({ "n", "v" }, "dd", "delete_line", { descripton = "Deletes the line under cursor" })
  glyph.api.keymap_set({ "n", "v" }, "G", "move_to_eof", { descripton = "Moves cursor to the end of file" })
  glyph.api.keymap_set({ "n", "v" }, "gg", "move_to_sof", { descripton = "Moves cursor to start of file" })
  glyph.api.keymap_set({ "n", "v" }, "0", "move_to_sol", { descripton = "Moves cursor to start of line" })
  glyph.api.keymap_set({ "n", "v" }, "$", "move_to_eol", { descripton = "Moves cursor to end of line" })
  glyph.api.keymap_set({ "n", "v" }, "<c-d>", "page_down", { descripton = "Moves cursor half screen down" })
  glyph.api.keymap_set({ "n", "v" }, "<c-u>", "page_up", { descripton = "Moves cursor half screen up" })
  glyph.api.keymap_set({ "n", "v" }, "i", "insert_mode", { description = "Change into insert mode" })
  glyph.api.keymap_set({ "n", "v" }, ":", "command_mode", { description = "Change into command mode" })
  glyph.api.keymap_set({ "n", "v" }, "o", "insert_line_below", { description = "Insert line below cursor" })
  glyph.api.keymap_set({ "n", "v" }, "O", "insert_line_above", { description = "Insert line above cursor" })
  glyph.api.keymap_set({ "n", "v" }, "A", "insert_at_eol", { description = "Insert mode end of line" })
  glyph.api.keymap_set({ "n", "v" }, "a", "insert_ahead", { description = "Insert ahead of cursor" })
  glyph.api.keymap_set({ "n", "v" }, "x", "remove_curr_char", { description = "Remove char under cursor" })
  glyph.api.keymap_set({ "n", "v" }, "X", "remove_prev_char", { description = "Remove char before cursor" })
  glyph.api.keymap_set({ "n", "v" }, "dw", "delete_word", { description = "Delete word forward" })
  glyph.api.keymap_set({ "n", "v" }, "db", "delete_word_prev", { description = "Delete word backwards" })
  glyph.api.keymap_set({ "n", "v" }, "w", "next_word", { description = "Move to next word forward" })
  glyph.api.keymap_set({ "n", "v" }, "W", "next_word_big", { description = "Move spaced word forward" })
  glyph.api.keymap_set({ "n", "v" }, "b", "prev_word", { description = "Move to previous word forward" })
  glyph.api.keymap_set({ "n", "v" }, "B", "prev_word_big", { description = "Move previous word forward" })
  glyph.api.keymap_set("n", "J", "join_line_below", { description = "Joins the line below" })
  glyph.api.keymap_set("n", "v", "visual_mode", { description = "Enters visual mode" })
  glyph.api.keymap_set("n", "V", "visual_line_mode", { description = "Enters visual line mode" })
  glyph.api.keymap_set("n", "<c-v>", "visual_block_mode", { description = "Enters visual block mode" })

  glyph.api.keymap_set("c", "<c-c>", "normal_mode", { description = "Changes into normal mode" })
  glyph.api.keymap_set("c", "<esc>", "normal_mode", { description = "Changes into normal mode" })

  glyph.api.keymap_set("i", "jk", "normal_mode", { description = "Changes into normal mode" })
  glyph.api.keymap_set("i", "<esc>", "normal_mode", { description = "Changes into normal mode" })
  glyph.api.keymap_set("i", "<c-c>", "normal_mode", { description = "Changes into normal mode" })
  glyph.api.keymap_set("i", "<c-w>", "delete_word_prev", { description = "Delete word backwards" })
  -- stylua: ignore end
end

return M
