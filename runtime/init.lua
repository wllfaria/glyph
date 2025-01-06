--- @class glyph
--- @field u glyph.utils
--- @field api glyph.api
--- @field options glyph.options
--- @field plugins glyph.plugins
--- @field _core glyph.core

--- @class glyph.t.quit_opts
--- @field force? boolean
--- @field all? boolean

--- @alias glyph.t.mode "n" | "i" | "c" | "v"

--- @class glyph.t.write_opts
--- @field force? boolean
--- @field all? boolean

--- @class glyph.t.point
--- @field x integer
--- @field y integer

--- @class glyph.core
--- @field keymap_command_set fun(mode: glyph.t.mode | glyph.t.mode[], keys: string, command: string, opts?: KeymapOpts)
--- @field keymap_function_set fun(mode: glyph.t.mode | glyph.t.mode[], keys: string, command: function, opts?: KeymapOpts)
--- @field set_hl_group fun(name: string, opts: glyph.colors.hl_group)
--- @field editor_get_mode fun(): string
--- @field editor_quit fun(opts?: glyph.t.quit_opts)
--- @field editor_write fun(opts?: glyph.t.write_opts)
--- @field editor_open_file fun(filename: string)
--- @field window_get_active fun(): integer
--- @field window_get_cursor fun(window: integer): glyph.t.point
--- @field window_is_valid fun(window: integer): boolean
--- @field window_create fun(document: integer, opts?: glyph.t.window_create_opts): integer
--- @field document_get_active fun(): integer
--- @field document_get_line_count fun(document: integer): integer
--- @field document_is_valid fun(document: integer): boolean
--- @field document_get_filepath fun(document: integer): string
--- @field document_create fun(opts?: glyph.t.document_create_opts): integer
--- @field user_command_create fun(name: string, callback: fun(document: integer))

--- @class glyph.options
--- @field scroll_offset number
--- @field cursor glyph.options.cursor
--- @field gutter glyph.options.gutter
--- @field statusline glyph.options.statusline

--- @class glyph.options.cursor
--- @field style "block" | "steady_bar"

--- @class glyph.options.gutter
--- @field enabled boolean
--- @field anchor "left" | "right"
--- @field line_numbers "absolute" | "relative" | "relative_numbered"
--- @field sign_column "all" | "git" | "lsp"

--- @class glyph.options.statusline
--- @field left glyph.options.statusline.section[]
--- @field right glyph.options.statusline.section[]

--- @class glyph.options.statusline.section
--- @field content string|fun(): string
--- @field style string|glyph.colors.hl_group

--- @type glyph
local glyph = require("glyph")

glyph.u = require("utils")
glyph.api = require("api")

require("defaults")
