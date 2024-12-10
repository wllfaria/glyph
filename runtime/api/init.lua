--- @type glyph
local glyph = require("glyph")

--- @class glyph.api: glyph.api.colors
--- @class glyph.api: glyph.api.keymaps
--- @class glyph.api: glyph.api.pretty_printer
--- @class glyph.api: glyph.api.themes
--- @class glyph.api: glyph.api.editor
--- @class glyph.api: glyph.api.window
--- @class glyph.api: glyph.api.document

--- @type glyph.api.colors
local colors = require("api.colors")

--- @type glyph.api.keymaps
local keymaps = require("api.keymaps")

--- @type glyph.api.pretty_printer
local pretty_printer = require("api.pretty_printer")

--- @type glyph.api.themes
local themes = require("api.themes")

--- @type glyph.api.editor
local editor = require("api.editor")

--- @type glyph.api.window
local window = require("api.window")

--- @type glyph.api.document
local document = require("api.document")

--- @type glyph.api
local M = glyph.u.table_deep_extend(
  "error",
  colors,
  keymaps,
  pretty_printer,
  themes,
  editor,
  window,
  document
)

return M
