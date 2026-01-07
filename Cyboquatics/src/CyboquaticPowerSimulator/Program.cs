using System;
using System.Globalization;
using CyboquaticPowerSimulator.Models;

namespace CyboquaticPowerSimulator
{
    internal static class Program
    {
        static void Main(string[] args)
        {
            Console.OutputEncoding = System.Text.Encoding.UTF8;

            string csvPath = args.Length > 0
                ? args[0]
                : "Data/CoastalNodes2026.csv";

            var nodes = CoastalNodeLoader.Load(csvPath);
            Console.WriteLine("# Cyboquatic Power and PFBS Simulation");
            Console.WriteLine("# Source: " + csvPath);

            foreach (var node in nodes)
            {
                var powerKw = HydrokineticCalculator.EstimatePowerKw(node);
                var pfbsKgPerH = PFBSRemovalSimulator.EstimatePfbsRemovalKgPerH(node);
                var ecoScore = TelemetryPowerEstimator.EstimateEcoImpactScore(node, powerKw, pfbsKgPerH);

                Console.WriteLine(string.Join(",",
                    node.NodeId,
                    powerKw.ToString("F2", CultureInfo.InvariantCulture),
                    pfbsKgPerH.ToString("F3", CultureInfo.InvariantCulture),
                    ecoScore.ToString("F3", CultureInfo.InvariantCulture)
                ));
            }
        }
    }
}
