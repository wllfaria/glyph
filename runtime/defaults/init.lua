local glyph = require("glyph")
local keymaps = require("defaults.keymaps")
local themes = require("defaults.themes")
glyph.config = require("defaults.options")

themes.tokyodark.load()

if not glyph.config.keymap_set or glyph.config.keymap_set == "neovim" then
  keymaps.neovim.load()
end
