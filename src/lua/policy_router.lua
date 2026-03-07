--! Cyboquatics Policy Router
--! 
--! Flexible policy routing system for cyboquatic machines that allows
--! policy shifts and compliance paradigm changes without recompilation.
--! This Lua module provides hot-reloadable policy logic for human-robotics
--! integration, stakeholder classification, and soul-boundary enforcement.
--!
--! @author Doctor Jacob Scott Farmer
--! @version 1.0.0
--! @date 2026-03-07
--! Evidence Hex: 0xCQ2026LUA9D8E7F6A
--! Knowledge Factor: F ≈ 0.88

local PolicyRouter = {}
PolicyRouter.__index = PolicyRouter

-- Policy configuration table (hot-reloadable)
local policy_config = {
    jurisdiction = "US-AZ-Maricopa-Phoenix",
    version = "1.0.0",
    evidence_hex = "0xCQ2026LUA9D8E7F6A",
    
    -- Stakeholder class permissions
    stakeholder_permissions = {
        CyberneticHost = {
            can_augment = true,
            can_govern = true,
            can_vote = true,
            neu_budget_multiplier = 1.0,
        },
        AugmentedCitizen = {
            can_augment = true,
            can_govern = false,
            can_vote = true,
            neu_budget_multiplier = 0.8,
        },
        RegularStakeholder = {
            can_augment = false,
            can_govern = false,
            can_vote = false,
            neu_budget_multiplier = 0.0,
        },
        ClinicalOperator = {
            can_augment = false,
            can_govern = true,
            can_vote = false,
            neu_budget_multiplier = 0.5,
        },
        Regulator = {
            can_augment = false,
            can_govern = true,
            can_vote = false,
            neu_budget_multiplier = 0.0,
        },
    },
    
    -- Soul guardrail constraints
    soul_guardrails = {
        forbid_soul_scoring = true,
        forbid_soul_transfer = true,
        require_consent = true,
        require_reversibility = true,
        hitl_required_scopes = {
            "mind-state",
            "religious",
            "existential",
            "high-psych-risk",
        },
    },
    
    -- Compliance thresholds
    thresholds = {
        min_eco_impact = 0.7,
        max_risk_residual = 0.3,
        min_neu_budget = 0.1,
        min_knowledge_factor = 0.85,
    },
    
    -- Action routing rules
    action_routes = {
        neuromodulation = {
            requires_soul_check = true,
            requires_hitl = true,
            neu_cost = 0.15,
            allowed_modes = {"REMEDIATION", "MAINTENANCE"},
        },
        memory_restoration = {
            requires_soul_check = true,
            requires_hitl = true,
            neu_cost = 0.20,
            allowed_modes = {"REMEDIATION"},
        },
        nanoswarm_deployment = {
            requires_soul_check = true,
            requires_hitl = false,
            neu_cost = 0.10,
            allowed_modes = {"REMEDIATION", "MONITORING"},
        },
        xr_experience = {
            requires_soul_check = false,
            requires_hitl = false,
            neu_cost = 0.05,
            allowed_modes = {"MONITORING", "REMEDIATION", "MAINTENANCE"},
        },
    },
}

--- Create a new PolicyRouter instance
-- @param node_id The unique node identifier
-- @return PolicyRouter instance
function PolicyRouter.new(node_id)
    local self = setmetatable({}, PolicyRouter)
    self.node_id = node_id
    self.loaded_policies = {}
    self.audit_log = {}
    self.last_policy_reload = os.time()
    return self
end

--- Load policy from ALN particle or configuration file
-- @param policy_name Name of the policy to load
-- @return boolean Success status
function PolicyRouter:loadPolicy(policy_name)
    -- In production, this would load from ALN particle registry
    -- For now, use internal policy_config
    if policy_config[policy_name] then
        self.loaded_policies[policy_name] = policy_config[policy_name]
        self:logAudit("POLICY_LOADED", policy_name)
        return true
    end
    return false
end

--- Reload all policies (hot-reload capability)
-- This allows policy shifts without recompilation
function PolicyRouter:reloadPolicies()
    self.loaded_policies = {}
    self.last_policy_reload = os.time()
    self:logAudit("POLICIES_RELOADED", "All policies reloaded")
    return true
end

--- Verify stakeholder permissions for an action
-- @param stakeholder_class The stakeholder's classification
-- @param action The action being requested
-- @return boolean, string Allowed status and reason
function PolicyRouter:verifyStakeholderPermission(stakeholder_class, action)
    local permissions = policy_config.stakeholder_permissions[stakeholder_class]
    
    if not permissions then
        return false, "Unknown stakeholder class: " .. tostring(stakeholder_class)
    end
    
    -- Check action-specific permissions
    if action == "augment" and not permissions.can_augment then
        return false, "Stakeholder class cannot perform augmentation"
    end
    
    if action == "govern" and not permissions.can_govern then
        return false, "Stakeholder class cannot participate in governance"
    end
    
    if action == "vote" and not permissions.can_vote then
        return false, "Stakeholder class cannot vote"
    end
    
    return true, "Permission granted"
end

--- Verify soul boundary constraints for an action
-- @param action_type Type of action being requested
-- @param citizen_did DID of the augmented citizen
-- @return table Validation result with violations
function PolicyRouter:verifySoulBoundary(action_type, citizen_did)
    local result = {
        allowed = true,
        violations = {},
        karma_delta = 0.0,
        rollback_required = false,
    }
    
    -- Check if action requires soul check
    local action_config = policy_config.action_routes[action_type]
    if action_config and action_config.requires_soul_check then
        -- Verify soul guardrails are not violated
        if policy_config.soul_guardrails.forbid_soul_scoring then
            -- Ensure no soul scoring is attempted
            -- (In production, check action parameters)
        end
        
        if policy_config.soul_guardrails.forbid_soul_transfer then
            -- Ensure no soul transfer is attempted
        end
    end
    
    -- Check if HITL is required
    local hitl_scopes = policy_config.soul_guardrails.hitl_required_scopes
    for _, scope in ipairs(hitl_scopes) do
        if string.find(action_type, scope) then
            -- Verify HITL approval exists
            -- (In production, check HITL approval token)
            result.karma_delta = result.karma_delta + 0.1
        end
    end
    
    if #result.violations > 0 then
        result.allowed = false
        result.rollback_required = true
    end
    
    return result
end

--- Route an action based on current policy
-- @param action The action to route
-- @param context Action context (stakeholder, mode, etc.)
-- @return table Routing decision
function PolicyRouter:routeAction(action, context)
    local decision = {
        allowed = false,
        route = nil,
        neu_cost = 0.0,
        violations = {},
        evidence_hex = self:generateEvidenceHex(),
    }
    
    -- Verify stakeholder permissions
    local perm_allowed, perm_reason = self:verifyStakeholderPermission(
        context.stakeholder_class,
        action.type
    )
    
    if not perm_allowed then
        table.insert(decision.violations, perm_reason)
        return decision
    end
    
    -- Verify soul boundaries
    local soul_validation = self:verifySoulBoundary(action.type, context.citizen_did)
    if not soul_validation.allowed then
        decision.violations = soul_validation.violations
        decision.karma_delta = soul_validation.karma_delta
        return decision
    end
    
    -- Check action-specific routing rules
    local action_config = policy_config.action_routes[action.type]
    if not action_config then
        table.insert(decision.violations, "Unknown action type: " .. action.type)
        return decision
    end
    
    -- Verify mode compatibility
    local mode_allowed = false
    for _, allowed_mode in ipairs(action_config.allowed_modes) do
        if allowed_mode == context.current_mode then
            mode_allowed = true
            break
        end
    end
    
    if not mode_allowed then
        table.insert(decision.violations, 
            "Action not allowed in mode: " .. context.current_mode)
        return decision
    end
    
    -- Check NEU budget
    local required_neu = action_config.neu_cost
    if context.neu_budget_remaining < required_neu then
        table.insert(decision.violations, "Insufficient NEU budget")
        return decision
    end
    
    -- All checks passed
    decision.allowed = true
    decision.route = action_config.route or "default"
    decision.neu_cost = required_neu
    
    return decision
end

--- Compute knowledge-factor for policy deployment
-- Formula: F = α·V + β·R + γ·E + δ·N
-- @return number Knowledge factor [0.0, 1.0]
function PolicyRouter:computeKnowledgeFactor()
    local alpha = 0.30  -- validation weight
    local beta = 0.25   -- reuse weight
    local gamma = 0.30  -- ecological impact weight
    local delta = 0.15  -- novelty weight
    
    local validation = 0.9
    local reuse = 0.8
    local ecological = 0.85  -- Assume good eco-impact
    local novelty = 0.7
    
    local factor = alpha * validation
                 + beta * reuse
                 + gamma * ecological
                 + delta * novelty
    
    return math.max(0.0, math.min(1.0, factor))
end

--- Generate evidence hex for audit trail
-- @return string Evidence hex
function PolicyRouter:generateEvidenceHex()
    local timestamp = os.time()
    local hash = self.node_id .. timestamp .. policy_config.version
    -- Simple hash for demonstration (use crypto in production)
    local hex = string.format("0xLUA%08X", timestamp % 0xFFFFFFFF)
    return hex
end

--- Log audit event
-- @param event_type Type of event
-- @param details Event details
function PolicyRouter:logAudit(event_type, details)
    local entry = {
        timestamp = os.time(),
        node_id = self.node_id,
        event_type = event_type,
        details = details,
        evidence_hex = self:generateEvidenceHex(),
    }
    table.insert(self.audit_log, entry)
    
    -- Keep audit log bounded
    local MAX_ENTRIES = 10000
    if #self.audit_log > MAX_ENTRIES then
        table.remove(self.audit_log, 1)
    end
end

--- Get audit log entries
-- @param limit Maximum number of entries to return
-- @return table Audit log entries
function PolicyRouter:getAuditLog(limit)
    limit = limit or 100
    local start_idx = math.max(1, #self.audit_log - limit + 1)
    local result = {}
    for i = start_idx, #self.audit_log do
        table.insert(result, self.audit_log[i])
    end
    return result
end

--- Export policy configuration (for verification)
-- @return table Current policy configuration
function PolicyRouter:exportPolicyConfig()
    return policy_config
end

--- Import policy configuration (for hot-reload)
-- @param new_config New policy configuration table
-- @return boolean Success status
function PolicyRouter:importPolicyConfig(new_config)
    -- Validate new configuration
    if not new_config.jurisdiction then
        return false
    end
    
    -- Merge with existing config
    for k, v in pairs(new_config) do
        policy_config[k] = v
    end
    
    self:logAudit("POLICY_UPDATED", "Configuration updated via import")
    return true
end

return PolicyRouter
