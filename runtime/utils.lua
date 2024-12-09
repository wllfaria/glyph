local M = {}

--- Validate opts against requirements.
---
--- @generic T1
--- @param tbl T1 The table to validate.
--- @param requirements table<string, type[]> A table containing validation requirements for `tbl`.
function M.table_validate(tbl, requirements)
  if type(tbl) ~= "table" then
    error("opts must be a table")
  end

  if type(requirements) ~= "table" then
    error("requirements must be a table")
  end

  for key, valueTypes in pairs(requirements) do
    local value = tbl[key]

    if not value and not M.table_contains(valueTypes, "nil") then
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
      error(
        "invalid type for key '" .. key .. "'. Expected one of: " .. table.concat(valueTypes, ", ")
      )
    end
  end
end

--- @generic T1: table
--- @generic T2: table
--- @param behavior "keep" | "force" | "error"
--- @param ... T2 two or more tables
--- @return T1|T2 (table) Merged table
function M.table_deep_extend(behavior, ...)
  if behavior ~= "error" and behavior ~= "keep" and behavior ~= "force" then
    error('invalid "behavior": ' .. tostring(behavior))
  end

  if select("#", ...) < 2 then
    error(
      "wrong number of arguments (given "
        .. tostring(1 + select("#", ...))
        .. ", expected at least 3)"
    )
  end

  local result = {}

  for i = 1, select("#", ...) do
    local tbl = select(i, ...)
    --- @cast tbl table<any,any>
    if tbl then
      for k, v in pairs(tbl) do
        if type(v) == "table" and type(result[k]) == "table" then
          result[k] = M.table_deep_extend(behavior, result[k], v)
        elseif behavior ~= "force" and result[k] ~= nil then
          if behavior == "error" then
            error("key found in more than one map: " .. k)
          end -- else behavior is "keep".
        else
          result[k] = v
        end
      end
    end
  end

  return result
end

--- @generic T
--- @param tbl T[] | table<any, T>
--- @param needle T
--- @return boolean
function M.table_contains(tbl, needle)
  if type(tbl) ~= "table" then
    return false
  end

  if tbl[1] ~= nil then
    for _, item in ipairs(tbl) do
      if item == needle then
        return true
      end
    end
  end

  for _, val in pairs(tbl) do
    if val == needle then
      return true
    end
  end

  return false
end

return M
