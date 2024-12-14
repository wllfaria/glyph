--- @type glyph
local glyph = require("glyph")

local commands = require("defaults.commands")
local keymaps = require("defaults.keymaps")
local themes = require("defaults.themes")

local options = require("defaults.options")
--- @cast options glyph.options
glyph.options = options

themes.tokyodark.setup()
keymaps.neovim.setup()
commands.setup()
