-- edge/ker_status_panel.lua
-- Read ecosafety.riskvector.2026v1 CSV and print per-node KER + lane health.
-- Non-actuating; for terminals / consoles only. [file:7]

local csv_path = arg[1] or "qpudatashards/phoenixEcosafety2026v1.csv"

local function split(line, sep)
  local t = {}
  for part in string.gmatch(line, "([^" .. sep .. "]+)") do
    table.insert(t, part)
  end
  return t
end

local function parse_bool(s)
  return s == "true" or s == "TRUE" or s == "1"
end

local function read_rows(path)
  local rows = {}
  local fh = assert(io.open(path, "r"))
  local header = fh:read("*l")
  if not header then return rows end
  local cols = split(header, ",")
  local idx = {}
  for i, name in ipairs(cols) do
    idx[name] = i
  end

  for line in fh:lines() do
    if line ~= "" then
      local parts = split(line, ",")
      local function col(name) return parts[idx[name]] or "" end
      local row = {
        nodeid        = col("nodeid"),
        k             = tonumber(col("kmetric")),
        e             = tonumber(col("emetric")),
        r             = tonumber(col("rmetric")),
        vt            = tonumber(col("vt")),
        lane          = col("lane"),
        biosurfaceok  = parse_bool(col("biosurfaceok")),
        hydraulicok   = parse_bool(col("hydraulicok")),
        lyapunovok    = parse_bool(col("lyapunovok")),
        tailwindvalid = parse_bool(col("tailwindvalid")),
      }
      table.insert(rows, row)
    end
  end
  fh:close()
  return rows
end

local function classify_health(row)
  if not (row.biosurfaceok and row.hydraulicok and row.lyapunovok and row.tailwindvalid) then
    return "UNSAFE"
  end
  if row.k and row.e and row.r then
    if row.k >= 0.95 and row.e >= 0.93 and row.r <= 0.11 then
      return "EXCELLENT"
    elseif row.k >= 0.90 and row.e >= 0.90 and row.r <= 0.13 then
      return "WITHIN_BAND"
    else
      return "AT_RISK"
    end
  end
  return "UNKNOWN"
end

local rows = read_rows(csv_path)

print(string.format("%-18s  %-10s  %-5s  %-5s  %-5s  %-10s  %s",
  "nodeid", "lane", "K", "E", "R", "health", "flags"))

for _, row in ipairs(rows) do
  local health = classify_health(row)
  local flags = {}
  if not row.biosurfaceok  then table.insert(flags, "bio") end
  if not row.hydraulicok   then table.insert(flags, "hyd") end
  if not row.lyapunovok    then table.insert(flags, "vt")  end
  if not row.tailwindvalid then table.insert(flags, "tw")  end
  local flag_str = (#flags > 0) and table.concat(flags, "|") or "-"
  print(string.format("%-18s  %-10s  %5.2f  %5.2f  %5.2f  %-10s  %s",
    row.nodeid, row.lane, row.k or 0.0, row.e or 0.0, row.r or 0.0, health, flag_str))
end
