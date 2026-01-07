using System;

namespace CyboquaticPowerSimulator.Models
{
    public static class HydrokineticCalculator
    {
        private const double WaterDensityKgM3 = 1025.0;

        public static double EstimatePowerKw(CoastalNode node)
        {
            // Use rated power if present; otherwise compute from mean flow and a nominal area.
            if (node.RatedPowerKw > 0.0)
                return node.RatedPowerKw;

            double areaM2 = 10.0;
            double efficiency = 0.35;

            double powerW = 0.5 * WaterDensityKgM3 * areaM2 * Math.Pow(node.MeanFlowMs, 3) * efficiency;
            return powerW / 1000.0;
        }
    }
}
