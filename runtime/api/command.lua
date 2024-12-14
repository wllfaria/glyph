--- @type glyph
local glyph = require("glyph")

--- @class glyph.api.command
--- @field user_command_create fun(name: string, callback: fun(document: integer))
local M = {}

--- @param name string
--- @param callback fun(document: integer)
function M.user_command_create(name, callback)
  if type(name) ~= "string" then
    error("command name must be a string")
  end

  if type(callback) ~= "function" then
    error("command callback must be a function")
  end

  glyph._core.user_command_create(name, callback)
end

return M
