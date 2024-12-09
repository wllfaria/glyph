local glyph = require("glyph")

glyph.u = require("utils")

local colors = require("colors")
local keymaps = require("keymaps")
local pretty_printer = require("pretty_printer")
local themes = require("themes")
glyph.api = glyph.u.table_deep_extend("error", colors, pretty_printer, themes, keymaps)

require("defaults")
