namespace CyboquaticPowerSimulator.Models
{
    public static class PFBSRemovalSimulator
    {
        public static double EstimatePfbsRemovalKgPerH(CoastalNode node)
        {
            if (node.PfbsRemovalKgPerH > 0.0)
                return node.PfbsRemovalKgPerH;

            // Fallback: scale removal with flow and power in a simple way.[web:13]
            double baseline = 0.2;
            double flowFactor = node.MeanFlowMs;
            double powerFactor = node.RatedPowerKw / 50.0;
            double estimate = baseline * (1.0 + flowFactor * 0.3 + powerFactor * 0.4);
            return estimate;
        }
    }
}
