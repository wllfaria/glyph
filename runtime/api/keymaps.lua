--- @type glyph
local glyph = require("glyph")

--- @class glyph.api.keymaps
--- @field keymap_set fun(mode: glyph.t.mode | glyph.t.mode[], keys: string, command: string | function, opts?: KeymapOpts)
local M = {}

local static_commands = {
  "move_left",
  "move_down",
  "move_up",
  "move_right",
  "delete_line",
  "insert_mode",
  "normal_mode",
  "command_mode",
  "move_to_eof",
  "move_to_sof",
  "move_to_sol",
  "move_to_eol",
  "page_down",
  "page_up",
  "insert_line_below",
  "insert_line_above",
  "insert_at_eol",
  "insert_ahead",
  "remove_curr_char",
  "remove_prev_char",
  "delete_word",
  "delete_word_prev",
  "next_word",
  "next_word_big",
  "prev_word",
  "prev_word_big",
  "join_line_below",
  "visual_mode",
  "visual_line_mode",
  "visual_block_mode",
}

--- @class KeymapOpts
--- @field description? string

--- @param mode glyph.t.mode | glyph.t.mode[]
--- @param keys string
--- @param command string | function
--- @param opts? KeymapOpts
function M.keymap_set(mode, keys, command, opts)
  opts = opts or {}

  if not mode then
    error("keybind must have a mode")
  end

  if not command then
    error("command is required")
  end

  if not keys then
    error("field keys is required")
  end

  if type(mode) == "table" then
    for _, m in pairs(mode) do
      if not glyph.u.table_contains({ "n", "i", "c", "v" }, m) then
        error("invalid keymap mode " .. mode)
      end
    end
  else
    if not glyph.u.table_contains({ "n", "i", "c", "v" }, mode) then
      error("invalid keymap mode " .. mode)
    end
  end

  glyph.u.table_validate(opts, {
    description = { "string", "nil" },
  })

  if type(command) == "string" then
    if not glyph.u.table_contains(static_commands, command) then
      error(command(" is not an editor command"))
    end
    glyph._core.keymap_command_set(mode, keys, command, opts)
  else
    glyph._core.keymap_function_set(mode, keys, command, opts)
  end
end

return M
