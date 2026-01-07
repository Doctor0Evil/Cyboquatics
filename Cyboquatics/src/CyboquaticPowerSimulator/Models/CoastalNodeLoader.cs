using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;

namespace CyboquaticPowerSimulator.Models
{
    public static class CoastalNodeLoader
    {
        public static IReadOnlyList<CoastalNode> Load(string path)
        {
            var list = new List<CoastalNode>();
            using var reader = new StreamReader(path);
            string? header = reader.ReadLine();
            if (header == null)
                return list;

            while (!reader.EndOfStream)
            {
                var line = reader.ReadLine();
                if (string.IsNullOrWhiteSpace(line))
                    continue;

                var parts = line.Split(',');
                if (parts.Length < 9)
                    continue;

                list.Add(new CoastalNode
                {
                    NodeId = parts[0].Trim(),
                    LatitudeDeg = Parse(parts[1]),
                    LongitudeDeg = Parse(parts[2]),
                    DepthM = Parse(parts[3]),
                    MeanFlowMs = Parse(parts[4]),
                    FlowVarianceMs2 = Parse(parts[5]),
                    RatedPowerKw = Parse(parts[6]),
                    PfbsRemovalKgPerH = Parse(parts[7]),
                    MaxIntakeFlowMs = Parse(parts[8])
                });
            }

            return list;
        }

        private static double Parse(string s) =>
            double.Parse(s.Trim(), CultureInfo.InvariantCulture);
    }
}
