using System;

namespace EcoNet.Cyboquatics
{
    public enum FlowVacContext { Urban, Coastal }

    public struct FlowVacSiteVolume
    {
        public string SiteId { get; init; }
        public double Lat { get; init; }
        public double Lon { get; init; }
        public double AFootprintM2 { get; init; } // Max footprint area (m²)
        public double HClearM { get; init; } // Vertical clearance (m)
        public double DSoilM { get; init; } // Soil depth (m)
        public double DPipeM { get; init; } // Pipe depth (m)
        public string ZoningCode { get; init; } // e.g., "MU-IND"

        public bool IsFeasibleGeom(double deviceFootprint, double deviceHeight, double deviceDepth)
        {
            return deviceFootprint <= AFootprintM2 && deviceHeight <= HClearM && deviceDepth <= DSoilM && deviceDepth >= DPipeM;
        }
    }

    public struct FlowVacHydraulicEnvelope
    {
        public double QMinM3S { get; init; } // Min flow (m³/s)
        public double QMaxM3S { get; init; } // Max flow (m³/s)
        public double VMaxMS { get; init; } // Max velocity (m/s)
        public double HLossMaxM { get; init; } // Max head-loss (m)
        public bool BackflowFlag { get; init; }

        public bool IsFeasibleHyd(double designQ, double designV, double designHLoss)
        {
            return designQ >= QMinM3S && designQ <= QMaxM3S && designV <= VMaxMS && designHLoss <= HLossMaxM && !BackflowFlag;
        }
    }

    public struct FlowVacResourceBudget
    {
        public double PAvailKW { get; init; } // Available power (kW)
        public double EDailyKWh { get; init; } // Daily energy (kWh)
        public double OCrewHours { get; init; } // Crew hours/month
        public int MIntervalDays { get; init; } // Maintenance interval (days)

        public bool IsFeasibleResource(double devicePNominal, double deviceEDaily, double requiredCrew, int requiredInterval)
        {
            return devicePNominal <= PAvailKW && deviceEDaily <= EDailyKWh && requiredCrew <= OCrewHours && requiredInterval <= MIntervalDays;
        }
    }

    public struct FlowVacMaterialBOM
    {
        public double MSteelKg { get; init; }
        public double MPolyKg { get; init; }
        public double MFilterKg { get; init; }
        public double LCableM { get; init; }
        public double LPipeM { get; init; }

        public double TotalEmbodiedCarbonKgCO2() // From EPA embodied carbon data
        {
            return (MSteelKg * 1.8) + (MPolyKg * 2.5) + (MFilterKg * 3.2) + (LCableM * 0.5) + (LPipeM * 0.8); // kgCO2e
        }
    }

    public struct FlowVacBioSafety
    {
        public string HabitatType { get; init; } // e.g., "URBAN_CANAL"
        public double DExclusionM { get; init; } // Exclusion radius (m)
        public int NSpecies { get; init; } // Sensitive species count
        public double NoiseLimitDB { get; init; } // Max noise (dB)
        public double EMLimitUT { get; init; } // Max EM field (µT)
        public double BioStressIndexMax { get; init; } // Max stress index (0-1)

        public bool IsFeasibleBio(double modeledStress, double modeledNoise, double modeledEM, int impactedSpecies)
        {
            return modeledStress <= BioStressIndexMax && modeledNoise <= NoiseLimitDB && modeledEM <= EMLimitUT && impactedSpecies <= NSpecies;
        }
    }

    public struct FlowVacPlacementDecision
    {
        public FlowVacSiteVolume Site { get; init; }
        public FlowVacHydraulicEnvelope Hyd { get; init; }
        public FlowVacResourceBudget Res { get; init; }
        public FlowVacMaterialBOM Bom { get; init; }
        public FlowVacBioSafety Bio { get; init; }
        public FlowVacContext Context { get; init; }
        public double DeltaKn { get; init; } // CEIM node impact change
        public double DeltaE { get; init; } // Energy change (kWh)
        public double DeltaEcoImpact { get; init; } // Ecoimpact change

        public bool IsAcceptable()
        {
            // Quantum_Reflection check: mass/energy balanced
            bool qrOk = DeltaKn > 0 && DeltaE <= 0 && DeltaEcoImpact >= 0;
            return qrOk && Site.IsFeasibleGeom(10.0, 2.5, 2.0) && Hyd.IsFeasibleHyd(0.1, 1.5, 0.4) && Res.IsFeasibleResource(12.0, 50.0, 3.5, 100) && Bio.IsFeasibleBio(0.15, 60.0, 0.4, 3) && Bom.TotalEmbodiedCarbonKgCO2() < 500.0;
        }
    }

    // Example usage in CPVM guard
    public static class CPVMGuard
    {
        public static bool ValidatePlacement(FlowVacPlacementDecision decision)
        {
            // CPVM viability: all feasible and QR balanced
            return decision.IsAcceptable();
        }
    }
}
