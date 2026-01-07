namespace CyboquaticPowerSimulator.Models
{
    public sealed class CoastalNode
    {
        public string NodeId { get; init; } = "";
        public double LatitudeDeg { get; init; }
        public double LongitudeDeg { get; init; }
        public double DepthM { get; init; }
        public double MeanFlowMs { get; init; }
        public double FlowVarianceMs2 { get; init; }
        public double RatedPowerKw { get; init; }
        public double PfbsRemovalKgPerH { get; init; }
        public double MaxIntakeFlowMs { get; init; }
    }
}
