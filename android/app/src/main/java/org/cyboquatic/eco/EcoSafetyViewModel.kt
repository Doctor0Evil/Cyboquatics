// EcoSafetyViewModel.kt
// Prepares K, E, R trajectories and health bands for plotting. [file:5][file:7]

package org.cyboquatic.eco

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

data class KerPoint(
    val tLabel: String,
    val k: Double,
    val e: Double,
    val r: Double
)

data class NodeKerState(
    val nodeid: String,
    val kerSeries: List<KerPoint>,
    val latestHealth: HealthBand
)

class EcoSafetyViewModel(
    private val repo: EcoSafetyRepository
) : ViewModel() {

    private val _nodeKerState = MutableStateFlow<NodeKerState?>(null)
    val nodeKerState: StateFlow<NodeKerState?> = _nodeKerState

    fun loadNode(nodeId: String) {
        viewModelScope.launch {
            val snapshots = repo.loadSnapshots(nodeId)
            if (snapshots.isEmpty()) {
                _nodeKerState.value = null
                return@launch
            }
            val series = snapshots.sortedBy { it.windowEndUtc }.map { snap ->
                KerPoint(
                    tLabel = snap.windowEndUtc,
                    k = snap.kmetric,
                    e = snap.emetric,
                    r = snap.rmetric
                )
            }
            val latest = snapshots.maxByOrNull { it.windowEndUtc }!!
            val health = classifyHealth(latest)
            _nodeKerState.value = NodeKerState(
                nodeid = nodeId,
                kerSeries = series,
                latestHealth = health
            )
        }
    }
}
