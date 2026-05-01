// File: android/app/src/main/java/org/cyboquatic/ShardViewModel.kt

package org.cyboquatic

import androidx.lifecycle.ViewModel

data class KerSnapshot(
    val k: Double,
    val e: Double,
    val r: Double,
    val lane: String,
    val nodeId: String,
    val region: String,
)

class ShardViewModel : ViewModel() {

    fun kerBandColor(snapshot: KerSnapshot): Int {
        return when {
            snapshot.k >= 0.90 && snapshot.e >= 0.90 && snapshot.r <= 0.13 ->
                0xFF00C853.toInt() // green
            snapshot.k >= 0.85 && snapshot.e >= 0.85 && snapshot.r <= 0.16 ->
                0xFFFFAB00.toInt() // amber
            else ->
                0xFFD50000.toInt() // red
        }
    }

    fun ecoScore(snapshot: KerSnapshot): Double {
        // Simple eco-score for dashboards, higher is better.
        return (snapshot.k + snapshot.e - snapshot.r).coerceIn(0.0, 2.0)
    }
}
