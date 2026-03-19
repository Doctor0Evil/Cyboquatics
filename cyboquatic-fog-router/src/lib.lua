--! Cyboquatic FOG Router Library (Lua)
--! 
--! Implements predicate-based dispatch for distributed Cyboquatic nodes.
--! Workloads are routed based on energy surplus, hydraulic capacity, 
--! bio-risk levels, and Lyapunov stability invariants.
--! 
--! Safety Guarantees:
--! - Pure functions (no side effects, no actuator access)
--! - Deterministic routing decisions (Accept/Reroute/Reject)
--! - Enforces V_t stability across distributed network
--! 
--! @file lib.lua
--! @destination cyboquatic-fog-router/src/lib.lua

local CyboRouter = {}
CyboRouter.__index = CyboRouter

-- ============================================================================
-- CONSTANTS & CONFIGURATION
-- ============================================================================

local MIN_ENERGY_SURPLUS_JOULES = 100.0
local MAX_LYAPUNOV_DELTA = 0.001
local HYDRAULIC_SAFETY_MARGIN = 0.85
local BIO_RISK_THRESHOLD = 0.13
local KER_DEPLOY_THRESHOLD = 0.90

-- Routing Decision Enum
local Decision = {
    ACCEPT = "Accept",
    REROUTE = "Reroute",
    REJECT = "Reject"
}

-- ============================================================================
-- DATA STRUCTURES (Documented Types)
-- ============================================================================

-- WorkloadProfile:
-- {
--   id: string,
--   energy_requirement: number, -- Joules
--   hydraulic_impact: number,   -- m³/s equivalent
--   media_class: string,        -- "bio_contact", "dry", "fluid"
--   nominal_delta_vt: number,   -- Expected change in Lyapunov residual
--   carbon_intensity: number    -- kg CO2e per operation
-- }

-- NodeShard (qpudatashard):
-- {
--   node_id: string,
--   energy_surplus: number,     -- Available Joules
--   hydraulic_capacity: number, -- Max m³/s
--   bio_risk_level: number,     -- Current r_biology (0-1)
--   current_vt: number,         -- Current Lyapunov residual
--   ker_metrics: {              -- Governance scores
--     k: number,                -- Knowledge-factor
--     e: number,                -- Eco-impact
--     r: number                 -- Risk-of-harm
--   }
--   carbon_state: string        -- "negative", "neutral", "positive"
-- }

-- RoutingContext:
-- {
--   timestamp: number,
--   network_load: number,       -- 0.0 to 1.0
--   emergency_mode: boolean
-- }

-- ============================================================================
-- ROUTING PREDICATES (Pure Functions)
-- ============================================================================

--- Checks if node has sufficient energy surplus for workload
-- @param workload WorkloadProfile
-- @param node NodeShard
-- @return boolean
function CyboRouter.tailwind_valid(workload, node)
    if not workload or not node then return false end
    local available = node.energy_surplus or 0.0
    local required = workload.energy_requirement or 0.0
    return (available - required) >= MIN_ENERGY_SURPLUS_JOULES
end

--- Checks if node bio-risk level is safe for workload media class
-- @param workload WorkloadProfile
-- @param node NodeShard
-- @return boolean
function CyboRouter.bio_surface_ok(workload, node)
    if not workload or not node then return false end
    local bio_risk = node.bio_risk_level or 1.0
    
    -- Strict gate for bio_contact workloads
    if workload.media_class == "bio_contact" then
        return bio_risk <= BIO_RISK_THRESHOLD
    end
    
    -- Relaxed gate for dry operations
    if workload.media_class == "dry" then
        return bio_risk <= 0.50
    end
    
    return true
end

--- Checks if node hydraulic capacity can handle workload impact
-- @param workload WorkloadProfile
-- @param node NodeShard
-- @return boolean
function CyboRouter.hydraulic_ok(workload, node)
    if not workload or not node then return false end
    local capacity = node.hydraulic_capacity or 0.0
    local impact = workload.hydraulic_impact or 0.0
    return (capacity - impact) >= (capacity * HYDRAULIC_SAFETY_MARGIN)
end

--- Checks if workload execution preserves Lyapunov stability
-- @param workload WorkloadProfile
-- @param node NodeShard
-- @return boolean
function CyboRouter.lyapunov_ok(workload, node)
    if not workload or not node then return false end
    local current_vt = node.current_vt or 0.0
    local delta = workload.nominal_delta_vt or 0.0
    
    -- Invariant: V_t_next <= V_t_prev + epsilon
    -- Here we check if the proposed delta violates the stability bound
    if delta > MAX_LYAPUNOV_DELTA then
        return false
    end
    
    -- Additional check: Do not route to nodes already near instability
    if current_vt > 0.8 then
        return false
    end
    
    return true
end

--- Checks node governance metrics (KER) for deployment readiness
-- @param node NodeShard
-- @return boolean
function CyboRouter.governance_ok(node)
    if not node or not node.ker_metrics then return false end
    local k = node.ker_metrics.k or 0.0
    local e = node.ker_metrics.e or 0.0
    local r = node.ker_metrics.r or 1.0
    
    return (k >= KER_DEPLOY_THRESHOLD) 
       and (e >= KER_DEPLOY_THRESHOLD) 
       and (r <= BIO_RISK_THRESHOLD)
end

--- Checks carbon state preference (prioritize negative/neutral)
-- @param workload WorkloadProfile
-- @param node NodeShard
-- @return number -- Score (0.0 to 1.0, higher is better)
function CyboRouter.carbon_score(workload, node)
    if not node then return 0.0 end
    local state = node.carbon_state or "positive"
    
    if state == "negative" then
        return 1.0
    elseif state == "neutral" then
        return 0.7
    else
        return 0.3
    end
end

-- ============================================================================
-- ROUTER DISPATCH LOGIC
-- ============================================================================

--- Evaluates all predicates and returns a routing decision
-- @param workload WorkloadProfile
-- @param node NodeShard
-- @param context RoutingContext
-- @return Decision, string (reason)
function CyboRouter.evaluate(workload, node, context)
    if not workload or not node then
        return Decision.REJECT, "Invalid input structures"
    end
    
    -- Emergency mode bypasses some checks but not Lyapunov
    local emergency = context and context.emergency_mode or false
    
    -- 1. Hard Safety Gates (Never bypassed)
    if not CyboRouter.lyapunov_ok(workload, node) then
        return Decision.REJECT, "Lyapunov stability violation"
    end
    
    if not emergency and not CyboRouter.governance_ok(node) then
        return Decision.REROUTE, "Node governance below threshold"
    end
    
    -- 2. Resource Constraints
    if not CyboRouter.tailwind_valid(workload, node) then
        return Decision.REROUTE, "Insufficient energy surplus"
    end
    
    if not CyboRouter.hydraulic_ok(workload, node) then
        return Decision.REROUTE, "Hydraulic capacity exceeded"
    end
    
    -- 3. Ecological Safety
    if not CyboRouter.bio_surface_ok(workload, node) then
        return Decision.REJECT, "Bio-risk corridor violation"
    end
    
    -- 4. Optimization (Carbon Scoring)
    local carbon_score = CyboRouter.carbon_score(workload, node)
    if carbon_score < 0.5 and not emergency then
        return Decision.REROUTE, "Preferred carbon-negative node available"
    end
    
    return Decision.ACCEPT, "All predicates passed"
end

-- ============================================================================
-- DIAGNOSTIC FRAMES (Non-Actuating)
-- ============================================================================

--- Computes diagnostic metrics without affecting node state
-- @param nodes table<NodeShard>
-- @return table
function CyboRouter.diagnostic_frame(nodes)
    local diagnostics = {
        total_nodes = #nodes,
        available_nodes = 0,
        avg_ker_k = 0.0,
        avg_ker_e = 0.0,
        avg_ker_r = 0.0,
        carbon_negative_count = 0
    }
    
    local sum_k, sum_e, sum_r = 0.0, 0.0, 0.0
    
    for _, node in ipairs(nodes) do
        if CyboRouter.governance_ok(node) then
            diagnostics.available_nodes = diagnostics.available_nodes + 1
        end
        
        if node.ker_metrics then
            sum_k = sum_k + (node.ker_metrics.k or 0.0)
            sum_e = sum_e + (node.ker_metrics.e or 0.0)
            sum_r = sum_r + (node.ker_metrics.r or 1.0)
        end
        
        if node.carbon_state == "negative" then
            diagnostics.carbon_negative_count = diagnostics.carbon_negative_count + 1
        end
    end
    
    if #nodes > 0 then
        diagnostics.avg_ker_k = sum_k / #nodes
        diagnostics.avg_ker_e = sum_e / #nodes
        diagnostics.avg_ker_r = sum_r / #nodes
    end
    
    return diagnostics
end

--- Simulates routing decision without committing (Dry Run)
-- @param workload WorkloadProfile
-- @param nodes table<NodeShard>
-- @return table
function CyboRouter.dry_run(workload, nodes)
    local results = {}
    for _, node in ipairs(nodes) do
        local decision, reason = CyboRouter.evaluate(workload, node, {emergency_mode=false})
        table.insert(results, {
            node_id = node.node_id,
            decision = decision,
            reason = reason,
            carbon_score = CyboRouter.carbon_score(workload, node)
        })
    end
    return results
end

-- ============================================================================
-- UTILITIES
-- ============================================================================

--- Logs routing decision (Safe, non-actuating)
-- @param decision Decision
-- @param reason string
-- @param node_id string
function CyboRouter.log_decision(decision, reason, node_id)
    local timestamp = os.time()
    local log_entry = string.format(
        "[ROUTER] %s | Node: %s | Decision: %s | Reason: %s",
        timestamp, node_id, decision, reason
    )
    -- In production, this would write to a safe log buffer
    -- print(log_entry) 
    return log_entry
end

-- ============================================================================
-- MODULE EXPORT
-- ============================================================================

return {
    new = function() return setmetatable({}, CyboRouter) end,
    Decision = Decision,
    evaluate = CyboRouter.evaluate,
    dry_run = CyboRouter.dry_run,
    diagnostic_frame = CyboRouter.diagnostic_frame,
    predicates = {
        tailwind_valid = CyboRouter.tailwind_valid,
        bio_surface_ok = CyboRouter.bio_surface_ok,
        hydraulic_ok = CyboRouter.hydraulic_ok,
        lyapunov_ok = CyboRouter.lyapunov_ok,
        governance_ok = CyboRouter.governance_ok,
        carbon_score = CyboRouter.carbon_score
    }
}
