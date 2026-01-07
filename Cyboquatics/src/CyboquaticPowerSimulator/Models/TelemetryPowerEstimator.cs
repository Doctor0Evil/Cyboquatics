using System;

namespace CyboquaticPowerSimulator.Models
{
    public static class TelemetryPowerEstimator
    {
        public static double EstimateEcoImpactScore(
            CoastalNode node,
            double powerKw,
            double pfbsRemovalKgPerH)
        {
            if (powerKw <= 0.0 || pfbsRemovalKgPerH < 0.0)
                return 0.0;

            double powerScore = Math.Tanh(powerKw / 80.0);
            double pfbsScore = Math.Tanh(pfbsRemovalKgPerH / 2.0);

            double intakePenalty =
                node.MeanFlowMs > node.MaxIntakeFlowMs
                    ? 0.3
                    : 0.0;

            double score = 0.6 * powerScore + 0.4 * pfbsScore - intakePenalty;
            return Math.Clamp(score, 0.0, 1.0);
        }
    }
}
