local glyph = require("glyph")

--- Validate opts against requirements.
---
--- @generic T1
--- @param tbl T1 The table to validate.
--- @param requirements table<string, type[]> A table containing validation requirements for `tbl`.
function glyph.table_validate(tbl, requirements)
  if type(tbl) ~= "table" then
    error("opts must be a table")
  end

  if type(requirements) ~= "table" then
    error("requirements must be a table")
  end

  for key, valueTypes in pairs(requirements) do
    local value = tbl[key]

    if not value and not glyph.table_contains(valueTypes, "nil") then
      error("missing required key '" .. key .. "'")
    end

    local isValidType = false
    for _, allowedType in ipairs(valueTypes) do
      if (allowedType == "nil" and value == nil) or type(value) == allowedType then
        isValidType = true
        break
      end
    end

    if not isValidType then
      error("invalid type for key '" .. key .. "'. Expected one of: " .. table.concat(valueTypes, ", "))
    end
  end
end

--- Checks if a element exists in a table
---
--- @generic T1
--- @param tbl table<string, T1>
--- @param element T1
--- @return boolean
function glyph.table_contains(tbl, element)
  for _, value in ipairs(tbl) do
    if value == element then
      return true
    end
  end
  return false
end
