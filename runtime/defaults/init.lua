--- @type glyph
local glyph = require("glyph")

local keymaps = require("defaults.keymaps")
local themes = require("defaults.themes")

local options = require("defaults.options")
--- @cast options glyph.options
glyph.options = options

themes.tokyodark.load()
keymaps.neovim.load()
