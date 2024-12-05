local glyph = require("glyph")

glyph.config = require("cfg")
glyph.api = require("utils")

local colors = require("colors")
local pretty_printer = require("pretty_printer")
local themes = require("themes")

glyph.api = glyph.api.table_deep_extend("error", glyph.api, colors, pretty_printer, themes)

glyph.api.load_theme("default")
