stds.glyph = {}
std = "lua51+glyph"

-- Don't report unused self arguments of methods.
self = false

-- Rerun tests only if their modification time changed.
cache = true

ignore = {
  "631", -- max_line_length
  "212/_.*", -- unused argument, for vars with "_" prefix
  "214", -- used variable with unused hint ("_" prefix)
}

-- Global objects defined by the C code
read_globals = {
  "glyph",
}
