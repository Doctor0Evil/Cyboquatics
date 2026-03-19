-- ============================================================================
-- FILE: cyboquatics_orchestrator/src/orchestration.lua
-- DESTINATION: /cyboquatics/cyboquatics_orchestrator/src/orchestration.lua
-- LICENSE: MIT Public Good License (Non-Commercial, Open Ecosafety)
-- VERSION: 1.0.0-alpha
-- ============================================================================
-- Cyboquatics Lua Orchestration Layer - Pilot Configuration & Safety Binding
-- Bridges high-level ALN contracts with low-level Rust kernel enforcement
-- ============================================================================

local bit = require("bit")
local ffi = require("ffi")
local json = require("json")
local hashlib = require("hashlib")

-- ============================================================================
-- CONSTANTS & SAFETY THRESHOLDS
-- ============================================================================

local KER_K_THRESHOLD = 0.90
local KER_E_THRESHOLD = 0.90
local KER_R_THRESHOLD = 0.13
local KER_R_PRODUCTION = 0.10
local LYAPUNOV_THRESHOLD = 1.0
local MAX_RISK_COORDINATE = 0.13

local SAFETY_MODES = {
    RESEARCH = "research_lane",
    PRODUCTION = "production_lane",
    EMERGENCY = "emergency_mode"
}

-- ============================================================================
-- TYPE VALIDATION HELPERS (Lua Dynamic Typing Safety)
-- ============================================================================

local function validate_number(val, min, max, name)
    if type(val) ~= "number" then
        error(string.format("Invalid %s: expected number, got %s", name, type(val)))
    end
    if val < min or val > max then
        error(string.format("Invalid %s: %f out of range [%f, %f]", name, val, min, max))
    end
    return val
end

local function validate_string(val, pattern, name)
    if type(val) ~= "string" then
        error(string.format("Invalid %s: expected string, got %s", name, type(val)))
    end
    if pattern and not val:match(pattern) then
        error(string.format("Invalid %s: %s does not match pattern %s", name, val, pattern))
    end
    return val
end

local function validate_table(val, name)
    if type(val) ~= "table" then
        error(string.format("Invalid %s: expected table, got %s", name, type(val)))
    end
    return val
end

-- ============================================================================
-- DATA STRUCTURES - Ecosafety State Objects
-- ============================================================================

local KERScore = {}
KERScore.__index = KERScore

function KERScore:new(k, e, r)
    local instance = setmetatable({}, self)
    instance.knowledge_factor = validate_number(k, 0.0, 1.0, "K")
    instance.eco_impact = validate_number(e, 0.0, 1.0, "E")
    instance.risk_of_harm = validate_number(r, 0.0, 1.0, "R")
    return instance
end

function KERScore:is_deployable()
    return self.knowledge_factor >= KER_K_THRESHOLD
        and self.eco_impact >= KER_E_THRESHOLD
        and self.risk_of_harm <= KER_R_THRESHOLD
end

function KERScore:is_production_ready()
    return self:is_deployable() and self.risk_of_harm <= KER_R_PRODUCTION
end

function KERScore:to_string()
    return string.format("K=%.2f E=%.2f R=%.2f", self.knowledge_factor, self.eco_impact, self.risk_of_harm)
end

local LyapunovResidual = {}
LyapunovResidual.__index = LyapunovResidual

function LyapunovResidual:new(timestamp_ns, value, derivative)
    local instance = setmetatable({}, self)
    instance.timestamp_ns = validate_number(timestamp_ns, 0, math.huge, "timestamp_ns")
    instance.value = validate_number(value, 0.0, LYAPUNOV_THRESHOLD, "value")
    instance.derivative = validate_number(derivative, -math.huge, 0.0, "derivative")
    instance.is_stable = instance.derivative <= 0.0
    return instance
end

local EcosafetyCorridor = {}
EcosafetyCorridor.__index = EcosafetyCorridor

function EcosafetyCorridor:new(id, dimensions, ker_score)
    local instance = setmetatable({}, self)
    instance.corridor_id = validate_string(id, "^%l[%l%d_]+$", "corridor_id")
    instance.dimensions = validate_table(dimensions, "dimensions")
    instance.ker_score = ker_score
    instance.is_active = true
    instance.created_ns = os.time() * 1e9
    instance.last_validated_ns = instance.created_ns
    return instance
end

function EcosafetyCorridor:validate_dimensions()
    for _, dim in ipairs(self.dimensions) do
        for key, risk_val in pairs(dim) do
            if risk_val > MAX_RISK_COORDINATE then
                return false, string.format("Dimension %s exceeds risk limit: %f", key, risk_val)
            end
        end
    end
    return true, nil
end

-- ============================================================================
-- FFI BINDINGS - Rust Kernel Interface
-- ============================================================================

-- Simulated FFI definitions for linking with cyboquatics_core
ffi.cdef[[
    typedef struct EcosafetyKernel EcosafetyKernel;
    typedef struct QpuDatashard QpuDatashard;
    
    EcosafetyKernel* kernel_new();
    void kernel_free(EcosafetyKernel* k);
    int kernel_register_corridor(EcosafetyKernel* k, const char* corridor_json);
    int kernel_execute_safe_action(EcosafetyKernel* k, const char* action_data, const char* corridor_id);
    const char* kernel_get_last_shard(EcosafetyKernel* k);
    int kernel_emergency_stop(EcosafetyKernel* k);
]]

local rust_kernel = ffi.load("cyboquatics_core")

-- ============================================================================
-- QPUDATASHARD LOGGER - Audit Trail Management
-- ============================================================================

local ShardLogger = {}
ShardLogger.__index = ShardLogger

function ShardLogger:new(kernel_ptr)
    local instance = setmetatable({}, self)
    instance.kernel_ptr = kernel_ptr
    instance.shard_count = 0
    instance.last_hash = nil
    return instance
end

function ShardLogger:generate_hex_stamp(shard_id, timestamp_ns)
    return string.format("%s_%08x", shard_id, bit.band(timestamp_ns, 0xFFFFFFFF))
end

function ShardLogger:compute_hash(data)
    local hasher = hashlib.sha3_256()
    hasher:update(data)
    return "0x" .. hasher:finalise():hex()
end

function ShardLogger:submit_shard(corridor_id, ker_score, lyapunov, action_data)
    local timestamp_ns = os.time() * 1e9
    local shard_id = string.format("shard_%016x", timestamp_ns)
    local hex_stamp = self:generate_hex_stamp(shard_id, timestamp_ns)
    local did_signature = string.format("did:bostrom:cyboquatics:%s:%s", shard_id, hex_stamp)
    
    local action_hash = self:compute_hash(action_data)
    local prev_hash = self.last_hash
    
    local shard_data = {
        shard_id = shard_id,
        hex_stamp = hex_stamp,
        did_signature = did_signature,
        timestamp_ns = timestamp_ns,
        corridor_id = corridor_id,
        ker_snapshot = ker_score,
        lyapunov_snapshot = lyapunov,
        action_hash = action_hash,
        previous_shard_hash = prev_hash
    }
    
    local json_shard = json.encode(shard_data)
    
    -- Submit to Rust kernel for immutable storage
    local result = rust_kernel.kernel_execute_safe_action(self.kernel_ptr, json_shard, corridor_id)
    if result ~= 0 then
        error(string.format("Shard submission failed with error code: %d", result))
    end
    
    self.last_hash = self:compute_hash(shard_id)
    self.shard_count = self.shard_count + 1
    
    return shard_data
end

-- ============================================================================
-- ORCHESTRATOR - Pilot Lifecycle Management
-- ============================================================================

local Orchestrator = {}
Orchestrator.__index = Orchestrator

function Orchestrator:new(safety_mode)
    local kernel_ptr = rust_kernel.kernel_new()
    if kernel_ptr == nil then
        error("Failed to initialize Rust Ecosafety Kernel")
    end
    
    local instance = setmetatable({}, self)
    instance.kernel_ptr = kernel_ptr
    instance.logger = ShardLogger:new(kernel_ptr)
    instance.safety_mode = safety_mode or SAFETY_MODES.RESEARCH
    instance.corridors = {}
    instance.active_pilots = {}
    
    -- Load safety mode configuration
    instance:load_safety_config(safety_mode)
    
    return instance
end

function Orchestrator:load_safety_config(mode)
    if mode == SAFETY_MODES.RESEARCH then
        self.ker_k_min = 0.85
        self.ker_e_min = 0.85
        self.ker_r_max = 0.20
        self.audit_frequency = "per_operation"
    elseif mode == SAFETY_MODES.PRODUCTION then
        self.ker_k_min = KER_K_THRESHOLD
        self.ker_e_min = KER_E_THRESHOLD
        self.ker_r_max = KER_R_THRESHOLD
        self.audit_frequency = "continuous"
    elseif mode == SAFETY_MODES.EMERGENCY then
        self.ker_k_min = 0.95
        self.ker_e_min = 0.95
        self.ker_r_max = 0.05
        self.audit_frequency = "per_microoperation"
    else
        error("Unknown safety mode: " .. tostring(mode))
    end
end

function Orchestrator:register_corridor(corridor)
    -- Enforce "No Corridor, No Build" policy
    local valid, err = corridor:validate_dimensions()
    if not valid then
        error(string.format("Corridor registration failed: %s", err))
    end
    
    if not corridor.ker_score:is_deployable() then
        error(string.format("Corridor K/E/R score not deployable: %s", corridor.ker_score:to_string()))
    end
    
    local json_corridor = json.encode(corridor)
    local result = rust_kernel.kernel_register_corridor(self.kernel_ptr, json_corridor)
    if result ~= 0 then
        error(string.format("Kernel registration failed with error code: %d", result))
    end
    
    self.corridors[corridor.corridor_id] = corridor
    print(string.format("Corridor registered: %s (Mode: %s)", corridor.corridor_id, self.safety_mode))
end

function Orchestrator:execute_pilot_step(corridor_id, action_name, action_params)
    local corridor = self.corridors[corridor_id]
    if not corridor then
        error(string.format("Corridor not found: %s", corridor_id))
    end
    
    if not corridor.is_active then
        error(string.format("Corridor inactive - violating invariant.corridorcomplete: %s", corridor_id))
    end
    
    -- Re-validate dimensions before every action (Violated Corridor → Derate/Stop)
    local valid, err = corridor:validate_dimensions()
    if not valid then
        print(string.format("CRITICAL: Corridor violation detected: %s", err))
        self:trigger_emergency_stop(corridor_id)
        return nil
    end
    
    -- Update Lyapunov Residual (Simulated for pilot step)
    local current_time = os.time() * 1e9
    local lyapunov = LyapunovResidual:new(current_time, 0.5, -0.01)
    
    if not lyapunov.is_stable then
        error("Lyapunov instability detected - aborting action")
    end
    
    -- Prepare action data
    local action_data = json.encode({
        action = action_name,
        params = action_params,
        timestamp_ns = current_time
    })
    
    -- Submit shard
    local shard = self.logger:submit_shard(corridor_id, corridor.ker_score, lyapunov, action_data)
    
    print(string.format("Action executed: %s | Shard: %s", action_name, shard.shard_id))
    return shard
end

function Orchestrator:trigger_emergency_stop(corridor_id)
    print(string.format("EMERGENCY STOP TRIGGERED for corridor: %s", corridor_id))
    if self.corridors[corridor_id] then
        self.corridors[corridor_id].is_active = false
    end
    rust_kernel.kernel_emergency_stop(self.kernel_ptr)
    error("Emergency stop activated - system halted")
end

function Orchestrator:update_ker_score(corridor_id, new_k, new_e, new_r)
    local corridor = self.corridors[corridor_id]
    if not corridor then
        error(string.format("Corridor not found for update: %s", corridor_id))
    end
    
    local new_score = KERScore:new(new_k, new_e, new_r)
    
    -- Check against safety mode thresholds
    if new_score.knowledge_factor < self.ker_k_min or
       new_score.eco_impact < self.ker_e_min or
       new_score.risk_of_harm > self.ker_r_max then
        print(string.format("K/E/R update rejected - below safety mode thresholds: %s", new_score:to_string()))
        return false
    end
    
    corridor.ker_score = new_score
    print(string.format("K/E/R updated for %s: %s", corridor_id, new_score:to_string()))
    return true
end

function Orchestrator:get_audit_report()
    return {
        safety_mode = self.safety_mode,
        active_corridors = #self.corridors,
        shard_count = self.logger.shard_count,
        last_shard_hash = self.logger.last_hash,
        timestamp = os.time()
    }
end

function Orchestrator:cleanup()
    rust_kernel.kernel_free(self.kernel_ptr)
    print("Orchestrator cleanup complete - kernel freed")
end

-- ============================================================================
-- PILOT TEMPLATES - Pre-configured Ecological Restoration Scenarios
-- ============================================================================

local PilotTemplates = {
    PhoenixMAR = function()
        local ker = KERScore:new(0.93, 0.92, 0.14)
        local dimensions = {
            { ph = 0.05, turbidity = 0.08, contaminants = 0.10 },
            { toxicity = 0.07, erosion = 0.06, biodiversity = 0.09 }
        }
        return EcosafetyCorridor:new("phoenix_mar_001", dimensions, ker)
    end,
    
    AirGlobeUrban = function()
        local ker = KERScore:new(0.91, 0.89, 0.12)
        local dimensions = {
            { pm25 = 0.08, pm10 = 0.09, voc = 0.07 },
            { species_risk = 0.05, displacement = 0.04, recovery = 0.06 }
        }
        return EcosafetyCorridor:new("airglobe_urban_001", dimensions, ker)
    end,
    
    WetlandBiofilter = function()
        local ker = KERScore:new(0.94, 0.93, 0.11)
        local dimensions = {
            { ph = 0.04, turbidity = 0.06, contaminants = 0.08 },
            { species_risk = 0.03, displacement = 0.02, recovery = 0.05 }
        }
        return EcosafetyCorridor:new("wetland_biofilter_001", dimensions, ker)
    end
}

-- ============================================================================
-- MAIN EXECUTION BLOCK - Example Pilot Run
-- ============================================================================

local function run_pilot_example()
    print("Initializing Cyboquatics Orchestrator...")
    local orch = Orchestrator:new(SAFETY_MODES.RESEARCH)
    
    -- Register Phoenix MAR Corridor
    local phoenix_corridor = PilotTemplates.PhoenixMAR()
    orch:register_corridor(phoenix_corridor)
    
    -- Execute Pilot Steps
    local success, err = pcall(function()
        orch:execute_pilot_step("phoenix_mar_001", "water_sampling", { depth = 5.0, location = "basin_A" })
        orch:execute_pilot_step("phoenix_mar_001", "nanoswarm_deploy", { count = 1000, type = "filter" })
        orch:execute_pilot_step("phoenix_mar_001", "aquifer_recharge", { rate = 50.0, duration = 3600 })
        
        -- Update K/E/R based on pilot data
        orch:update_ker_score("phoenix_mar_001", 0.94, 0.93, 0.12)
    end)
    
    if not success then
        print(string.format("Pilot execution failed: %s", err))
    else
        print("Pilot execution completed successfully")
    end
    
    -- Generate Audit Report
    local report = orch:get_audit_report()
    print(json.encode(report, { pretty = true }))
    
    -- Cleanup
    orch:cleanup()
end

-- Uncomment to run standalone
-- run_pilot_example()

-- ============================================================================
-- EXPORTS
-- ============================================================================

return {
    Orchestrator = Orchestrator,
    KERScore = KERScore,
    EcosafetyCorridor = EcosafetyCorridor,
    LyapunovResidual = LyapunovResidual,
    ShardLogger = ShardLogger,
    PilotTemplates = PilotTemplates,
    SAFETY_MODES = SAFETY_MODES,
    THRESHOLDS = {
        KER_K = KER_K_THRESHOLD,
        KER_E = KER_E_THRESHOLD,
        KER_R = KER_R_THRESHOLD,
        RISK_MAX = MAX_RISK_COORDINATE
    }
}

-- ============================================================================
-- END OF FILE: cyboquatics_orchestrator/src/orchestration.lua
-- ============================================================================
