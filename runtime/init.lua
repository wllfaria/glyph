--- @class glyph
--- @field u glyph.utils
--- @field api glyph.api
--- @field options glyph.options
--- @field _core glyph.core

--- @class glyph.core
--- @field set_keymap_command fun(mode: "n" | "i" | "c" | "v", keys: string, command: string, opts?: KeymapOpts)
--- @field set_keymap_function fun(mode: "n" | "i" | "c" | "v", keys: string, command: function, opts?: KeymapOpts)
--- @field set_hl_group fun(name: string, opts: glyph.colors.hl_group_opts)
--- @field get_editor_mode fun(): string

--- @class glyph.options
--- @field cursor glyph.options.cursor
--- @field gutter glyph.options.gutter

--- @class glyph.options.cursor
--- @field style "block" | "steady_bar"

--- @class glyph.options.gutter
--- @field enabled boolean
--- @field anchor "left" | "right"
--- @field line_numbers "absolute" | "relative" | "relative_numbered"
--- @field sign_column "all" | "git" | "lsp"

--- @type glyph
local glyph = require("glyph")

glyph.u = require("utils")
glyph.api = require("api")

require("defaults")
