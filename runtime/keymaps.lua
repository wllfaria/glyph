--- @type glyph
local glyph = require("glyph")

--- @class glyph.keymaps
--- @field set_keymap fun(mode: "n" | "i" | "c" | "v", keys: string, command: string | function, opts?: KeymapOpts): nil
local M = {}

local static_commands = {
  "move_left",
  "move_down",
  "move_up",
  "move_right",
}

--- @class KeymapOpts
--- @field description? string

--- @param mode "n" | "i" | "c" | "v"
--- @param keys string
--- @param command string | function
--- @param opts? KeymapOpts
function M.set_keymap(mode, keys, command, opts)
  if not mode or not glyph.u.table_contains({ "n", "i", "c", "v" }, mode) then
    error("invalid keymap mode " .. mode)
  end

  if not command then
    error("command is required")
  end

  if not keys then
    error("field keys is required")
  end

  opts = opts or {}
  glyph.u.table_validate(opts, {
    description = { "string", "nil" },
  })

  if type(command) == "string" then
    if not glyph.u.table_contains(static_commands, command) then
      error(command(" is not an editor command"))
    end
    glyph._core.set_keymap_command(mode, keys, command, opts)
  else
    glyph._core.set_keymap_function(mode, keys, command, opts)
  end
end

return M
