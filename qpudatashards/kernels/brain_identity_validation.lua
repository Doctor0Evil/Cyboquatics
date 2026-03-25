-- brain_identity_validation.lua
-- ALN Kernel Extension for Brain-Identity Shard Validation
-- Provides dynamic corridor checking, risk normalization, and KER computation
-- for augmented citizen safety enforcement within the Cyboquatic ecosystem.

local bi_kernel = {}
bi_kernel._VERSION = "1.0.0"
bi_kernel._SPEC = "BrainIdentityCybo2026"

local function clamp(v, min, max)
    if v < min then return min end
    if v > max then return max end
    return v
end

local function pow2(v)
    return v * v
end

bi_kernel.CORRIDORS = {
    ecoimpact_max = 1.0,
    neurorights_hard_max = 2,
    karma_floor_min = 0.0,
    data_sensitivity_min = 1,
    data_sensitivity_max = 5,
    rsoul_max = 1.0,
    social_exposure_max = 1.0,
    R_max = 0.13,
    K_min = 0.90,
    E_min = 0.90,
    vt_epsilon = 0.001,
    karma_nonslash_enforced = true
}

bi_kernel.WEIGHTS = {
    w_energy = 0.15,
    w_hydraulic = 0.10,
    w_biology = 0.10,
    w_carbon = 0.15,
    w_materials = 0.10,
    w_neurorights = 0.15,
    w_soul = 0.10,
    w_social = 0.08,
    w_ecoimpact = 0.07
}

local NEURORIGHTS_MAP = {
    [0] = 0.0,
    [1] = 0.5,
    [2] = 1.0
}

local EVIDENCE_MAP = {
    [0] = "REDACTED",
    [1] = "HASHONLY",
    [2] = "FULLTRACE"
}

function bi_kernel.neurorights_to_risk(status_code)
    local risk = NEURORIGHTS_MAP[status_code]
    if risk == nil then return 1.0 end
    return risk
end

function bi_kernel.validate_neurorights_status(status_code)
    return status_code >= 0 and status_code <= 2
end

function bi_kernel.validate_evidence_mode(mode_code)
    return mode_code >= 0 and mode_code <= 2
end

function bi_kernel.validate_data_sensitivity(level)
    return level >= bi_kernel.CORRIDORS.data_sensitivity_min and 
           level <= bi_kernel.CORRIDORS.data_sensitivity_max
end

function bi_kernel.normalize_risk_coordinate(value, max_value)
    if max_value <= 0 then return 0.0 end
    return clamp(value / max_value, 0.0, 1.0)
end

function bi_kernel.compute_r_neurorights(status_code)
    if not bi_kernel.validate_neurorights_status(status_code) then
        return 1.0
    end
    return bi_kernel.neurorights_to_risk(status_code)
end

function bi_kernel.compute_r_soul(rsoul_residual)
    return bi_kernel.normalize_risk_coordinate(rsoul_residual, bi_kernel.CORRIDORS.rsoul_max)
end

function bi_kernel.compute_r_social(social_exposure_coord)
    return bi_kernel.normalize_risk_coordinate(social_exposure_coord, bi_kernel.CORRIDORS.social_exposure_max)
end

function bi_kernel.compute_r_ecoimpact(ecoimpactscore)
    return bi_kernel.normalize_risk_coordinate(ecoimpactscore, bi_kernel.CORRIDORS.ecoimpact_max)
end

function bi_kernel.compute_R_residual(r_energy, r_hydraulic, r_biology, r_carbon, r_materials,
                                        r_neurorights, r_soul, r_social, r_ecoimpact)
    local R = bi_kernel.WEIGHTS.w_energy * r_energy +
              bi_kernel.WEIGHTS.w_hydraulic * r_hydraulic +
              bi_kernel.WEIGHTS.w_biology * r_biology +
              bi_kernel.WEIGHTS.w_carbon * r_carbon +
              bi_kernel.WEIGHTS.w_materials * r_materials +
              bi_kernel.WEIGHTS.w_neurorights * r_neurorights +
              bi_kernel.WEIGHTS.w_soul * r_soul +
              bi_kernel.WEIGHTS.w_social * r_social +
              bi_kernel.WEIGHTS.w_ecoimpact * r_ecoimpact
    return R
end

function bi_kernel.compute_Vt(r_energy, r_hydraulic, r_biology, r_carbon, r_materials,
                               r_neurorights, r_soul, r_social, r_ecoimpact)
    local Vt = bi_kernel.WEIGHTS.w_energy * pow2(r_energy) +
               bi_kernel.WEIGHTS.w_hydraulic * pow2(r_hydraulic) +
               bi_kernel.WEIGHTS.w_biology * pow2(r_biology) +
               bi_kernel.WEIGHTS.w_carbon * pow2(r_carbon) +
               bi_kernel.WEIGHTS.w_materials * pow2(r_materials) +
               bi_kernel.WEIGHTS.w_neurorights * pow2(r_neurorights) +
               bi_kernel.WEIGHTS.w_soul * pow2(r_soul) +
               bi_kernel.WEIGHTS.w_social * pow2(r_social) +
               bi_kernel.WEIGHTS.w_ecoimpact * pow2(r_ecoimpact)
    return Vt
end

function bi_kernel.check_vt_stability(vt_current, vt_previous)
    local delta = vt_current - vt_previous
    return delta <= bi_kernel.CORRIDORS.vt_epsilon
end

function bi_kernel.check_karma_nonslash(karma_before, karma_after)
    if not bi_kernel.CORRIDORS.karma_nonslash_enforced then
        return true
    end
    return karma_after >= karma_before
end

function bi_kernel.check_KER(K_score, E_score, R_residual)
    return K_score >= bi_kernel.CORRIDORS.K_min and 
           E_score >= bi_kernel.CORRIDORS.E_min and 
           R_residual <= bi_kernel.CORRIDORS.R_max
end

function bi_kernel.validate_shard(shard)
    local violations = {}
    local violation_count = 0
    
    if not bi_kernel.validate_neurorights_status(shard.neurorights_status) then
        violation_count = violation_count + 1
        violations[#violations + 1] = "neurorights_status_invalid"
    end
    
    if not bi_kernel.validate_evidence_mode(shard.evidence_mode) then
        violation_count = violation_count + 1
        violations[#violations + 1] = "evidence_mode_invalid"
    end
    
    if not bi_kernel.validate_data_sensitivity(shard.data_sensitivity_level) then
        violation_count = violation_count + 1
        violations[#violations + 1] = "data_sensitivity_out_of_range"
    end
    
    if shard.karma_floor < bi_kernel.CORRIDORS.karma_floor_min then
        violation_count = violation_count + 1
        violations[#violations + 1] = "karma_floor_below_minimum"
    end
    
    if shard.rsoul_residual < 0 or shard.rsoul_residual > bi_kernel.CORRIDORS.rsoul_max then
        violation_count = violation_count + 1
        violations[#violations + 1] = "rsoul_residual_out_of_bounds"
    end
    
    if shard.social_exposure_coord < 0 or shard.social_exposure_coord > bi_kernel.CORRIDORS.social_exposure_max then
        violation_count = violation_count + 1
        violations[#violations + 1] = "social_exposure_out_of_bounds"
    end
    
    if shard.ecoimpactscore < 0 or shard.ecoimpactscore > bi_kernel.CORRIDORS.ecoimpact_max then
        violation_count = violation_count + 1
        violations[#violations + 1] = "ecoimpactscore_out_of_bounds"
    end
    
    return violation_count == 0, violations, violation_count
end

function bi_kernel.evaluate_step(shard, physical_rv, vt_previous, proposed_karma)
    local r_neurorights = bi_kernel.compute_r_neurorights(shard.neurorights_status)
    local r_soul = bi_kernel.compute_r_soul(shard.rsoul_residual)
    local r_social = bi_kernel.compute_r_social(shard.social_exposure_coord)
    local r_ecoimpact = bi_kernel.compute_r_ecoimpact(shard.ecoimpactscore)
    
    local R_residual = bi_kernel.compute_R_residual(
        physical_rv.r_energy, physical_rv.r_hydraulic, physical_rv.r_biology,
        physical_rv.r_carbon, physical_rv.r_materials,
        r_neurorights, r_soul, r_social, r_ecoimpact
    )
    
    local Vt_current = bi_kernel.compute_Vt(
        physical_rv.r_energy, physical_rv.r_hydraulic, physical_rv.r_biology,
        physical_rv.r_carbon, physical_rv.r_materials,
        r_neurorights, r_soul, r_social, r_ecoimpact
    )
    
    local vt_stable = bi_kernel.check_vt_stability(Vt_current, vt_previous)
    local karma_ok = bi_kernel.check_karma_nonslash(shard.karma_floor, proposed_karma)
    
    local any_hard_violation = 
        r_neurorights >= 1.0 or r_soul >= 1.0 or r_social >= 1.0 or r_ecoimpact >= 1.0 or
        physical_rv.r_energy >= 1.0 or physical_rv.r_hydraulic >= 1.0 or
        physical_rv.r_biology >= 1.0 or physical_rv.r_carbon >= 1.0 or
        physical_rv.r_materials >= 1.0
    
    local decision = "Accept"
    if not karma_ok then
        decision = "StopKarmaViolation"
    elseif any_hard_violation then
        decision = "Stop"
    elseif not vt_stable then
        decision = "Derate"
    end
    
    local K_score = 1.0
    local E_score = 1.0 - R_residual
    local ker_ok = bi_kernel.check_KER(K_score, E_score, R_residual)
    
    local ker_deployable = ker_ok and vt_stable and karma_ok and not any_hard_violation
    
    return {
        decision = decision,
        r_neurorights = r_neurorights,
        r_soul = r_soul,
        r_social = r_social,
        r_ecoimpact = r_ecoimpact,
        R_residual = R_residual,
        Vt_current = Vt_current,
        vt_delta = Vt_current - vt_previous,
        vt_stable = vt_stable,
        karma_ok = karma_ok,
        ker_deployable = ker_deployable,
        K_score = K_score,
        E_score = E_score,
        any_hard_violation = any_hard_violation
    }
end

function bi_kernel.generate_audit_entry(shard, step_result, vt_previous, timestamp_unix)
    return {
        timestamp_unix = timestamp_unix,
        brainidentityid = shard.brainidentityid,
        hexstamp = shard.hexstamp,
        vt_previous = vt_previous,
        vt_current = step_result.Vt_current,
        vt_delta = step_result.vt_delta,
        decision = step_result.decision,
        karma_floor_before = shard.karma_floor,
        karma_floor_after = step_result.karma_ok and math.max(shard.karma_floor, shard.karma_floor) or shard.karma_floor,
        ker_deployable = step_result.ker_deployable,
        karma_violated = not step_result.karma_ok,
        vt_violated = not step_result.vt_stable
    }
end

function bi_kernel.update_ker_window(ker_window, step_result)
    ker_window.steps = ker_window.steps + 1
    
    if step_result.decision == "Accept" then
        ker_window.safe_steps = ker_window.safe_steps + 1
    end
    
    ker_window.karma_preserved = ker_window.karma_preserved and step_result.karma_ok
    
    local max_r = ker_window.max_r
    max_r = math.max(max_r, step_result.r_neurorights)
    max_r = math.max(max_r, step_result.r_soul)
    max_r = math.max(max_r, step_result.r_social)
    max_r = math.max(max_r, step_result.r_ecoimpact)
    ker_window.max_r = max_r
    
    return ker_window
end

function bi_kernel.compute_ker_summary(ker_window)
    local K = ker_window.steps > 0 and ker_window.safe_steps / ker_window.steps or 1.0
    local E = 1.0 - ker_window.max_r
    local R = ker_window.max_r
    
    return {
        K = K,
        E = E,
        R = R,
        karma_preserved = ker_window.karma_preserved,
        deployable = K >= bi_kernel.CORRIDORS.K_min and 
                     E >= bi_kernel.CORRIDORS.E_min and 
                     R <= bi_kernel.CORRIDORS.R_max and 
                     ker_window.karma_preserved
    }
end

function bi_kernel.get_corridor_config()
    local config = {}
    for k, v in pairs(bi_kernel.CORRIDORS) do
        config[k] = v
    end
    return config
end

function bi_kernel.get_weights_config()
    local weights = {}
    for k, v in pairs(bi_kernel.WEIGHTS) do
        weights[k] = v
    end
    return weights
end

function bi_kernel.set_corridor(key, value)
    if bi_kernel.CORRIDORS[key] ~= nil then
        bi_kernel.CORRIDORS[key] = value
        return true
    end
    return false
end

function bi_kernel.set_weight(key, value)
    if bi_kernel.WEIGHTS[key] ~= nil then
        bi_kernel.WEIGHTS[key] = value
        return true
    end
    return false
end

function bi_kernel.export_to_csv_row(shard, step_result, ker_summary, timestamp_unix)
    local row = string.format(
        "%s,%s,%s,%.4f,%d,%.2f,%d,%d,%.4f,%.4f,%.4f,%.4f,%.4f,%.6f,%.6f,%.6f,%s,%s,%d,%s",
        shard.node_id or "NODE_UNKNOWN",
        shard.brainidentityid,
        shard.hexstamp,
        shard.ecoimpactscore,
        shard.neurorights_status,
        shard.karma_floor,
        shard.data_sensitivity_level,
        shard.evidence_mode,
        shard.rsoul_residual,
        shard.social_exposure_coord,
        step_result.R_residual,
        ker_summary.K,
        ker_summary.E,
        step_result.Vt_current,
        step_result.Vt_current - step_result.vt_delta,
        step_result.vt_delta,
        tostring(ker_summary.deployable),
        tostring(step_result.karma_ok),
        timestamp_unix,
        shard.notes or ""
    )
    return row
end

function bi_kernel.validate_csv_row(csv_row)
    local fields = {}
    for field in csv_row:gmatch("([^,]+)") do
        fields[#fields + 1] = field
    end
    
    if #fields < 20 then
        return false, "insufficient_fields"
    end
    
    local ecoimpact = tonumber(fields[4])
    local neurorights = tonumber(fields[5])
    local karma_floor = tonumber(fields[6])
    local data_sens = tonumber(fields[7])
    local evidence = tonumber(fields[8])
    local rsoul = tonumber(fields[9])
    local social = tonumber(fields[10])
    
    if ecoimpact == nil or ecoimpact < 0 or ecoimpact > 1 then
        return false, "ecoimpactscore_invalid"
    end
    
    if neurorights == nil or neurorights < 0 or neurorights > 2 then
        return false, "neurorights_status_invalid"
    end
    
    if karma_floor == nil or karma_floor < 0 then
        return false, "karma_floor_invalid"
    end
    
    if data_sens == nil or data_sens < 1 or data_sens > 5 then
        return false, "data_sensitivity_invalid"
    end
    
    if evidence == nil or evidence < 0 or evidence > 2 then
        return false, "evidence_mode_invalid"
    end
    
    if rsoul == nil or rsoul < 0 or rsoul > 1 then
        return false, "rsoul_residual_invalid"
    end
    
    if social == nil or social < 0 or social > 1 then
        return false, "social_exposure_invalid"
    end
    
    return true, nil
end

return bi_kernel
