using System;

namespace CyboquaticPowerSimulator.Models
{
    public class HydrokineticCalculator
    {
        private const double SeawaterDensity = 1025.0; // kg/m³
        private const double TypicalCp = 0.38;         // Practical power coefficient

        public double CalculatePower(double rotorDiameterM, double velocityMs)
        {
            if (rotorDiameterM <= 0 || velocityMs <= 0)
                throw new ArgumentException("Positive non-zero values required");

            double area = Math.PI * Math.Pow(rotorDiameterM / 2.0, 2);
            double mechanicalPower = 0.5 * SeawaterDensity * area * Math.Pow(velocityMs, 3) * TypicalCp;
            double electricalPower = mechanicalPower * 0.80; // 80% generator + electronics efficiency
            return electricalPower; // Watts
        }

        public double EstimateDailyTelemetrymWh(bool useLoRaWAN = true)
        {
            // Empirical marine values 2025-2026 deployments
            return useLoRaWAN ? 21.5 : 45.0; // mWh/day at 15-min intervals
        }
    }
}
