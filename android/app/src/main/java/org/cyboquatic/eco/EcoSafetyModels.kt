// EcoSafetyModels.kt
// Data models mirroring ecosafety.riskvector.2026.v1. [file:7]

package org.cyboquatic.eco

data class RiskVectorSnapshot(
    val nodeid: String,
    val segmentid: String,
    val region: String,
    val lat: Double,
    val lon: Double,
    val windowStartUtc: String,
    val windowEndUtc: String,
    val shardid: String,
    val evidencehex: String,
    val renergy: Double,
    val rhydraulic: Double,
    val rbio: Double,
    val rcarbon: Double,
    val rmaterials: Double,
    val rcalib: Double,
    val vt: Double,
    val kmetric: Double,
    val emetric: Double,
    val rmetric: Double,
    val biosurfaceok: Boolean,
    val hydraulicok: Boolean,
    val lyapunovok: Boolean,
    val tailwindvalid: Boolean,
    val lane: String
)

enum class HealthBand {
    EXCELLENT,
    WITHIN_BAND,
    AT_RISK,
    UNSAFE,
    UNKNOWN
}

fun classifyHealth(snapshot: RiskVectorSnapshot): HealthBand {
    if (!(snapshot.biosurfaceok &&
          snapshot.hydraulicok &&
          snapshot.lyapunovok &&
          snapshot.tailwindvalid)) {
        return HealthBand.UNSAFE
    }
    return when {
        snapshot.kmetric >= 0.95 &&
        snapshot.emetric >= 0.93 &&
        snapshot.rmetric <= 0.11 -> HealthBand.EXCELLENT

        snapshot.kmetric >= 0.90 &&
        snapshot.emetric >= 0.90 &&
        snapshot.rmetric <= 0.13 -> HealthBand.WITHIN_BAND

        snapshot.kmetric > 0.0   -> HealthBand.AT_RISK
        else                     -> HealthBand.UNKNOWN
    }
}
