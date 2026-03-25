package org.econet.cyboquatic.bi

import android.content.Context
import android.content.SharedPreferences
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.security.MessageDigest
import java.util.UUID

data class BrainIdentityId(val bytes: ByteArray) {
    companion object {
        fun fromHex(hex: String): BrainIdentityId {
            require(hex.length == 64) { "BrainIdentityId must be 64 hex characters" }
            return BrainIdentityId(hex.chunked(2).map { it.toInt(16).toByte() }.toByteArray())
        }
    }
    
    fun toHex(): String = bytes.joinToString("") { "%02x".format(it) }
    
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is BrainIdentityId) return false
        return bytes.contentEquals(other.bytes)
    }
    
    override fun hashCode(): Int = bytes.contentHashCode()
}

data class HexStamp(val bytes: ByteArray) {
    fun toHex(): String = bytes.joinToString("") { "%02x".format(it) }
    
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is HexStamp) return false
        return bytes.contentEquals(other.bytes)
    }
    
    override fun hashCode(): Int = bytes.contentHashCode()
}

enum class NeurorightsStatus(val code: Int) {
    Active(0),
    Restricted(1),
    Suspended(2);
    
    companion object {
        fun fromCode(code: Int): NeurorightsStatus = 
            values().find { it.code == code } ?: Suspended
    }
    
    fun toRiskCoord(): Float = when (this) {
        Active -> 0.0f
        Restricted -> 0.5f
        Suspended -> 1.0f
    }
}

enum class EvidenceMode(val code: Int) {
    Redacted(0),
    HashOnly(1),
    FullTrace(2);
    
    companion object {
        fun fromCode(code: Int): EvidenceMode = 
            values().find { it.code == code } ?: Redacted
    }
}

data class BrainIdentityShard(
    val brainidentityid: BrainIdentityId,
    val hexstamp: HexStamp,
    var ecoimpactscore: Float,
    var neurorightsStatus: NeurorightsStatus,
    var karmaFloor: Float,
    var dataSensitivityLevel: Int,
    var evidenceMode: EvidenceMode,
    var rsoulResidual: Float,
    var socialExposureCoord: Float,
    val timestampUnix: Long
) {
    fun rNeurorights(): Float = neurorightsStatus.toRiskCoord()
    fun rSoul(): Float = rsoulResidual.coerceIn(0.0f, 1.0f)
    fun rSocial(): Float = socialExposureCoord.coerceIn(0.0f, 1.0f)
    fun rEcoImpact(): Float = ecoimpactscore.coerceIn(0.0f, 1.0f)
    
    fun anyHardViolation(): Boolean = 
        rNeurorights() >= 1.0f || rSoul() >= 1.0f || 
        rSocial() >= 1.0f || rEcoImpact() >= 1.0f
    
    fun trySetKarmaFloor(newFloor: Float): Boolean {
        if (newFloor >= karmaFloor) {
            karmaFloor = newFloor
            return true
        }
        return false
    }
    
    fun updateRsoul(residual: Float) {
        rsoulResidual = residual.coerceIn(0.0f, 1.0f)
    }
    
    fun updateSocialExposure(coord: Float) {
        socialExposureCoord = coord.coerceIn(0.0f, 1.0f)
    }
    
    fun updateEcoImpact(score: Float) {
        ecoimpactscore = score.coerceIn(0.0f, 1.0f)
    }
    
    fun updateNeurorights(status: NeurorightsStatus) {
        neurorightsStatus = status
    }
    
    fun computeBiResidual(weights: BiLyapunovWeights): BiResidual {
        val vt = 
            weights.wEnergy * rNeurorights().pow2() +
            weights.wSoul * rSoul().pow2() +
            weights.wSocial * rSocial().pow2() +
            weights.wEcoImpact * rEcoImpact().pow2()
        return BiResidual(vt)
    }
    
    fun toJson(): JSONObject = JSONObject().apply {
        put("brainidentityid", brainidentityid.toHex())
        put("hexstamp", hexstamp.toHex())
        put("ecoimpactscore", ecoimpactscore)
        put("neurorights_status", neurorightsStatus.code)
        put("karma_floor", karmaFloor)
        put("data_sensitivity_level", dataSensitivityLevel)
        put("evidence_mode", evidenceMode.code)
        put("rsoul_residual", rsoulResidual)
        put("social_exposure_coord", socialExposureCoord)
        put("timestamp_unix", timestampUnix)
    }
    
    companion object {
        fun fromJson(json: JSONObject): BrainIdentityShard {
            return BrainIdentityShard(
                brainidentityid = BrainIdentityId.fromHex(json.getString("brainidentityid")),
                hexstamp = HexStamp(json.getString("hexstamp").chunked(2).map { it.toInt(16).toByte() }.toByteArray()),
                ecoimpactscore = json.getDouble("ecoimpactscore").toFloat(),
                neurorightsStatus = NeurorightsStatus.fromCode(json.getInt("neurorights_status")),
                karmaFloor = json.getDouble("karma_floor").toFloat(),
                dataSensitivityLevel = json.getInt("data_sensitivity_level"),
                evidenceMode = EvidenceMode.fromCode(json.getInt("evidence_mode")),
                rsoulResidual = json.getDouble("rsoul_residual").toFloat(),
                socialExposureCoord = json.getDouble("social_exposure_coord").toFloat(),
                timestampUnix = json.getLong("timestamp_unix")
            )
        }
        
        fun generateNewId(): BrainIdentityId {
            val uuid = UUID.randomUUID()
            val msb = uuid.mostSignificantBits
            val lsb = uuid.leastSignificantBits
            val bytes = ByteArray(32)
            for (i in 0..7) bytes[i] = (msb shr (56 - i * 8) and 0xFF).toByte()
            for (i in 8..15) bytes[i] = (lsb shr (56 - (i - 8) * 8) and 0xFF).toByte()
            for (i in 16..31) bytes[i] = (uuid.mostSignificantBits shr (56 - (i - 16) * 8) and 0xFF).toByte()
            return BrainIdentityId(bytes)
        }
        
        fun computeHexstamp(shard: BrainIdentityShard): HexStamp {
            val json = shard.toJson().toString()
            val digest = MessageDigest.getInstance("SHA-256")
            val hash = digest.digest(json.toByteArray())
            return HexStamp(hash)
        }
    }
}

data class BiLyapunovWeights(
    val wEnergy: Float = 0.15f,
    val wHydraulic: Float = 0.10f,
    val wBiology: Float = 0.10f,
    val wCarbon: Float = 0.15f,
    val wMaterials: Float = 0.10f,
    val wNeurorights: Float = 0.15f,
    val wSoul: Float = 0.10f,
    val wSocial: Float = 0.08f,
    val wEcoImpact: Float = 0.07f
) {
    fun toJson(): JSONObject = JSONObject().apply {
        put("w_energy", wEnergy)
        put("w_hydraulic", wHydraulic)
        put("w_biology", wBiology)
        put("w_carbon", wCarbon)
        put("w_materials", wMaterials)
        put("w_neurorights", wNeurorights)
        put("w_soul", wSoul)
        put("w_social", wSocial)
        put("w_ecoimpact", wEcoImpact)
    }
}

data class BiResidual(val vt: Float)

data class BiRiskVector(
    val rEnergy: Float,
    val rHydraulic: Float,
    val rBiology: Float,
    val rCarbon: Float,
    val rMaterials: Float,
    val rNeurorights: Float,
    val rSoul: Float,
    val rSocial: Float,
    val rEcoImpact: Float
) {
    fun anyHardViolation(): Boolean = 
        rEnergy >= 1.0f || rHydraulic >= 1.0f || rBiology >= 1.0f ||
        rCarbon >= 1.0f || rMaterials >= 1.0f || rNeurorights >= 1.0f ||
        rSoul >= 1.0f || rSocial >= 1.0f || rEcoImpact >= 1.0f
    
    fun computeResidual(weights: BiLyapunovWeights): BiResidual {
        val vt = 
            weights.wEnergy * rEnergy.pow2() +
            weights.wHydraulic * rHydraulic.pow2() +
            weights.wBiology * rBiology.pow2() +
            weights.wCarbon * rCarbon.pow2() +
            weights.wMaterials * rMaterials.pow2() +
            weights.wNeurorights * rNeurorights.pow2() +
            weights.wSoul * rSoul.pow2() +
            weights.wSocial * rSocial.pow2() +
            weights.wEcoImpact * rEcoImpact.pow2()
        return BiResidual(vt)
    }
}

enum class BiSafeStepDecision {
    Accept,
    Derate,
    Stop,
    StopKarmaViolation
}

data class BiSafeStepConfig(
    val epsilon: Float = 0.001f,
    val enforceKarmaNonslash: Boolean = true
)

data class BiKerWindow(
    var steps: Int = 0,
    var safeSteps: Int = 0,
    var maxR: Float = 0.0f,
    var karmaPreserved: Boolean = true
) {
    fun update(rv: BiRiskVector, decision: BiSafeStepDecision, karmaOk: Boolean) {
        steps += 1
        if (decision == BiSafeStepDecision.Accept) safeSteps += 1
        karmaPreserved = karmaPreserved && karmaOk
        
        maxR = maxOf(
            maxR, rv.rEnergy, rv.rHydraulic, rv.rBiology,
            rv.rCarbon, rv.rMaterials, rv.rNeurorights,
            rv.rSoul, rv.rSocial, rv.rEcoImpact
        )
    }
    
    fun k(): Float = if (steps == 0) 1.0f else safeSteps.toFloat() / steps.toFloat()
    fun r(): Float = maxR
    fun e(): Float = 1.0f - maxR
    fun biKerDeployable(): Boolean = k() >= 0.90f && e() >= 0.90f && r() <= 0.13f && karmaPreserved
}

class BrainIdentityShardManager(private val context: Context) {
    private val prefs: SharedPreferences = 
        context.getSharedPreferences("cyboquatic_bi_shards", Context.MODE_PRIVATE)
    
    private val activeShards: MutableMap<BrainIdentityId, BrainIdentityShard> = mutableMapOf()
    private val kerWindows: MutableMap<BrainIdentityId, BiKerWindow> = mutableMapOf()
    
    suspend fun loadShard(brainidentityid: BrainIdentityId): BrainIdentityShard? = 
        withContext(Dispatchers.IO) {
            val jsonStr = prefs.getString("shard_${brainidentityid.toHex()}", null)
            jsonStr?.let { BrainIdentityShard.fromJson(JSONObject(it)) }
        }
    
    suspend fun saveShard(shard: BrainIdentityShard): Boolean = withContext(Dispatchers.IO) {
        try {
            val editor = prefs.edit()
            editor.putString("shard_${shard.brainidentityid.toHex()}", shard.toJson().toString())
            editor.putLong("last_update_${shard.brainidentityid.toHex()}", System.currentTimeMillis() / 1000)
            editor.apply()
            activeShards[shard.brainidentityid] = shard
            if (!kerWindows.containsKey(shard.brainidentityid)) {
                kerWindows[shard.brainidentityid] = BiKerWindow()
            }
            true
        } catch (e: Exception) {
            false
        }
    }
    
    suspend fun createNewShard(
        initialKarma: Float = 100.0f,
        dataSensitivityLevel: Int = 2
    ): BrainIdentityShard = withContext(Dispatchers.IO) {
        val newId = BrainIdentityShard.generateNewId()
        val timestamp = System.currentTimeMillis() / 1000
        
        val shard = BrainIdentityShard(
            brainidentityid = newId,
            hexstamp = HexStamp(ByteArray(32)),
            ecoimpactscore = 0.0f,
            neurorightsStatus = NeurorightsStatus.Active,
            karmaFloor = initialKarma,
            dataSensitivityLevel = dataSensitivityLevel,
            evidenceMode = EvidenceMode.Redacted,
            rsoulResidual = 0.0f,
            socialExposureCoord = 0.0f,
            timestampUnix = timestamp
        )
        
        val hexstamp = BrainIdentityShard.computeHexstamp(shard)
        val finalShard = shard.copy(hexstamp = hexstamp)
        
        saveShard(finalShard)
        finalShard
    }
    
    suspend fun evaluateStep(
        brainidentityid: BrainIdentityId,
        physicalRv: BiRiskVector,
        prevResidual: BiResidual,
        proposedKarma: Float,
        weights: BiLyapunovWeights,
        config: BiSafeStepConfig
    ): BiStepResult = withContext(Dispatchers.Default) {
        val shard = loadShard(brainidentityid)
            ?: return@withContext BiStepResult(BiSafeStepDecision.Stop, null, false)
        
        val biRv = BiRiskVector(
            rEnergy = physicalRv.rEnergy,
            rHydraulic = physicalRv.rHydraulic,
            rBiology = physicalRv.rBiology,
            rCarbon = physicalRv.rCarbon,
            rMaterials = physicalRv.rMaterials,
            rNeurorights = shard.rNeurorights(),
            rSoul = shard.rSoul(),
            rSocial = shard.rSocial(),
            rEcoImpact = shard.rEcoImpact()
        )
        
        val nextResidual = biRv.computeResidual(weights)
        
        val decision = biSafeStep(
            prevResidual,
            nextResidual,
            biRv,
            shard.karmaFloor,
            proposedKarma,
            config
        )
        
        val karmaPreserved = when (decision) {
            BiSafeStepDecision.StopKarmaViolation -> false
            else -> true
        }
        
        if (decision == BiSafeStepDecision.Accept || decision == BiSafeStepDecision.Derate) {
            val updatedShard = shard.copy(karmaFloor = maxOf(shard.karmaFloor, proposedKarma))
            saveShard(updatedShard)
        }
        
        val kerWindow = kerWindows.getOrPut(brainidentityid) { BiKerWindow() }
        kerWindow.update(biRv, decision, karmaPreserved)
        
        BiStepResult(decision, nextResidual, karmaPreserved)
    }
    
    private fun biSafeStep(
        prevResidual: BiResidual,
        nextResidual: BiResidual,
        rvNext: BiRiskVector,
        prevKarma: Float,
        proposedKarma: Float,
        config: BiSafeStepConfig
    ): BiSafeStepDecision {
        if (config.enforceKarmaNonslash && proposedKarma < prevKarma) {
            return BiSafeStepDecision.StopKarmaViolation
        }
        
        if (rvNext.anyHardViolation()) {
            return BiSafeStepDecision.Stop
        }
        
        return if (nextResidual.vt <= prevResidual.vt + config.epsilon) {
            BiSafeStepDecision.Accept
        } else {
            BiSafeStepDecision.Derate
        }
    }
    
    suspend fun getKerSummary(brainidentityid: BrainIdentityId): BiKerSummary? = 
        withContext(Dispatchers.Default) {
            val kerWindow = kerWindows[brainidentityid] ?: return@withContext null
            BiKerSummary(
                k = kerWindow.k(),
                e = kerWindow.e(),
                r = kerWindow.r(),
                karmaPreserved = kerWindow.karmaPreserved,
                deployable = kerWindow.biKerDeployable()
            )
        }
    
    suspend fun getAllActiveShards(): List<BrainIdentityShard> = 
        withContext(Dispatchers.Default) {
            activeShards.values.toList()
        }
    
    suspend fun getShardCount(): Int = withContext(Dispatchers.Default) {
        activeShards.size
    }
    
    companion object {
        @Volatile private var instance: BrainIdentityShardManager? = null
        
        fun getInstance(context: Context): BrainIdentityShardManager {
            return instance ?: synchronized(this) {
                instance ?: BrainIdentityShardManager(context.applicationContext).also {
                    instance = it
                }
            }
        }
    }
}

data class BiStepResult(
    val decision: BiSafeStepDecision,
    val residual: BiResidual?,
    val karmaPreserved: Boolean
)

data class BiKerSummary(
    val k: Float,
    val e: Float,
    val r: Float,
    val karmaPreserved: Boolean,
    val deployable: Boolean
)

private fun Float.pow2(): Float = this * this
