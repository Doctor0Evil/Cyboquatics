import json
import subprocess
import sys

def check_proof_coverage():
    """Ensure all critical safety predicates have verification harnesses"""
    
    critical_predicates = [
        "gate_treatment_sat_ok",
        "gate_hydraulic_structural_ok", 
        "gate_fouling_om_ok",
        "gate_social_governance_ok",
        "compute_violation_residual",
        "validate_shard_invariants"
    ]
    
    # Run Kani to list proven functions
    result = subprocess.run(
        ["cargo", "kani", "--list", "--output-format=json"],
        capture_output=True,
        text=True
    )
    
    proven_functions = json.loads(result.stdout)
    
    missing = []
    for predicate in critical_predicates:
        if not any(predicate in func for func in proven_functions):
            missing.append(predicate)
    
    if missing:
        print(f"❌ Missing verification for: {missing}")
        print("Blocking CI until proofs are added.")
        sys.exit(1)
    else:
        print("✅ All critical predicates have formal verification")

if __name__ == "__main__":
    check_proof_coverage()
