// EcoSafetyRepository.kt
// Simple HTTP JSON client for ecosafety riskvector snapshots. [file:5]

package org.cyboquatic.eco

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import okhttp3.OkHttpClient
import okhttp3.Request

@Serializable
data class RiskVectorSnapshotDto(
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

class EcoSafetyRepository(
    private val baseUrl: String,
    private val client: OkHttpClient = OkHttpClient()
) {
    private val json = Json { ignoreUnknownKeys = true }

    suspend fun loadSnapshots(nodeId: String): List<RiskVectorSnapshot> =
        withContext(Dispatchers.IO) {
            val url = "$baseUrl/eco/riskvector?nodeid=$nodeId"
            val request = Request.Builder().url(url).build()
            client.newCall(request).execute().use { resp ->
                if (!resp.isSuccessful) return@use emptyList<RiskVectorSnapshot>()
                val body = resp.body?.string() ?: return@use emptyList<RiskVectorSnapshot>()
                val dtos = json.decodeFromString(ListSerializer(RiskVectorSnapshotDto.serializer()), body)
                dtos.map {
                    RiskVectorSnapshot(
                        nodeid        = it.nodeid,
                        segmentid     = it.segmentid,
                        region        = it.region,
                        lat           = it.lat,
                        lon           = it.lon,
                        windowStartUtc= it.windowStartUtc,
                        windowEndUtc  = it.windowEndUtc,
                        shardid       = it.shardid,
                        evidencehex   = it.evidencehex,
                        renergy       = it.renergy,
                        rhydraulic    = it.rhydraulic,
                        rbio          = it.rbio,
                        rcarbon       = it.rcarbon,
                        rmaterials    = it.rmaterials,
                        rcalib        = it.rcalib,
                        vt            = it.vt,
                        kmetric       = it.kmetric,
                        emetric       = it.emetric,
                        rmetric       = it.rmetric,
                        biosurfaceok  = it.biosurfaceok,
                        hydraulicok   = it.hydraulicok,
                        lyapunovok    = it.lyapunovok,
                        tailwindvalid = it.tailwindvalid,
                        lane          = it.lane
                    )
                }
            }
        }
}
