-- File: lua/cyboquatic_shard_logger.lua

local logger = {}

local function esc(s)
  return tostring(s):gsub('"', '""')
end

-- Append a CSV shard row for a node.
function logger.append_node_shard(path, shard)
  local f = assert(io.open(path, "a"))
  local row = string.format(
    '"%s","%s","%s","%s","%s","%s",%.6f,%.6f,%.6f,%.6f,%.6f,%.6f,%.6f,%.6f,%.6f,%.6f,%.6f,%.6f,"%"s","%s","%s"\n',
    esc(shard.shard_id),
    esc(shard.node_id),
    esc(shard.region),
    esc(shard.nodetype),
    esc(shard.t_start),
    esc(shard.t_end),
    shard.esurplus_j,
    shard.pmargin_kw,
    shard.dEdt_w,
    shard.q_m3s,
    shard.hlr_m_per_h,
    shard.rsurcharge,
    shard.r_pathogen,
    shard.r_fouling,
    shard.r_cec,
    shard.r_carbon,
    shard.r_materials,
    shard.r_biodiversity,
    shard.r_sigma,
    esc(shard.evidence_hex),
    esc(shard.bostrom_did),
    esc(shard.lane)
  )
  f:write(row)
  f:close()
end

return logger
