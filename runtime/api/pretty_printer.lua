--- @class glyph.api.pretty_printer
--- @field pretty_print fun(tbl: any, opts: PrettyPrinterOpts): nil
local M = {}

-- pretty printer for lua values, heavily inspired by neovim's `vim.inspect()`

--- @class PrettyPrinter
--- @field buf string[]
--- @field cycles table
--- @field ids any
--- @field level number
--- @field depth number
--- @field indent string
--- @field newline string

--- @class PrettyPrinterOpts
--- @field depth? number
--- @field newline? string
--- @field indent? string

--- @param opts PrettyPrinterOpts
--- @return PrettyPrinter
local function create_inspector(opts)
  return {
    buf = {},
    ids = {},
    cycles = {},
    level = 0,
    depth = opts.depth or math.huge,
    indent = opts.indent or "  ",
    newline = opts.newline or "\n",
  }
end

--- count cyclic references
---
--- @generic T1
--- @param value T1
--- @param seen? table
--- @param cycles? table
--- @return table
local function count_cycles(value, seen, cycles)
  seen = seen or {}
  cycles = cycles or {}

  if type(value) == "table" then
    if seen[value] then
      cycles[value] = (cycles[value] or 0) + 1
      return cycles
    end

    seen[value] = true
    for k, v in pairs(value) do
      count_cycles(k, seen, cycles)
      count_cycles(v, seen, cycles)
    end
    local mt = getmetatable(value)
    if mt then
      count_cycles(mt, seen, cycles)
    end
  end

  return cycles
end

--- @generic T1
--- @param printer PrettyPrinter
--- @param value T1
local function put_value(printer, value)
  local buf = printer.buf
  local tv = type(value)

  if tv == "string" then
    table.insert(buf, string.format("%q", value))
  elseif tv == "number" or tv == "boolean" or tv == "nil" then
    table.insert(buf, tostring(value))
  elseif tv == "table" then
    if printer.ids[value] then
      table.insert(buf, string.format("<cycle %d>", printer.ids[value]))
    elseif printer.level >= printer.depth then
      table.insert(buf, "{...}")
    else
      local id = #printer.ids + 1
      printer.ids[value] = id

      table.insert(buf, "{" .. printer.newline)
      printer.level = printer.level + 1

      local keys = {}
      for k in pairs(value) do
        table.insert(keys, k)
      end

      table.sort(keys, function(a, b)
        return tostring(a) < tostring(b)
      end)

      for i, k in ipairs(keys) do
        if i > 1 then
          table.insert(buf, ", " .. printer.newline)
        end
        table.insert(buf, printer.indent:rep(printer.level))
        put_value(printer, k)
        table.insert(buf, " = ")
        put_value(printer, value[k])
      end

      local mt = getmetatable(value)
      if mt then
        local indent = printer.indent:rep(printer.level)
        table.insert(buf, ", " .. printer.newline .. indent .. "<metatable> = ")
        put_value(printer, mt)
      end

      printer.level = printer.level - 1
      table.insert(buf, printer.newline .. printer.indent:rep(printer.level) .. "}")
    end
  else
    table.insert(buf, string.format("<%s>", tv))
  end
end

--- @param tbl any
--- @param opts? PrettyPrinterOpts
function M.pretty_print(tbl, opts)
  opts = opts or {}
  local printer = create_inspector(opts)
  printer.cycles = count_cycles(tbl)

  put_value(printer, tbl)
  return table.concat(printer.buf)
end

return M
