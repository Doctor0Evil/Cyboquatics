SuperStrict

' ---------------------------------------------------------
' File: tools/hydraulics/hydraulic_decay_explorer.bmx
' Role: Non-actuating research tool to explore simple
'       hydraulic decay / surcharge scenarios and emit
'       RFC4180 CSV for Rust/ALN ingestion.
' ---------------------------------------------------------

Framework brl.standardio
Import brl.filesystem
Import brl.math

Type THydraulicParams
    Field NodeId:String
    Field LengthM:Double
    Field DiameterM:Double
    Field Roughness:Double
    Field Slope:Double
    Field QinM3s:Double
    Field MaxHLR_MpH:Double      ' corridor reference
    Field MaxSurchargeRisk:Double ' nominal 0..1 reference
End Type

Type TSimConfig
    Field DtSeconds:Double
    Field Steps:Int
    Field OutputCsv:String
End Type

Type TSimState
    Field TimeS:Double
    Field Q_M3s:Double
    Field HLR_MpH:Double
    Field SurchargeRisk:Double
End Type

Function Main:Int()
    Local params:THydraulicParams = LoadHydraulicParams()
    Local cfg:TSimConfig = LoadSimConfig()

    If cfg.OutputCsv = "" Then
        cfg.OutputCsv = "output/hydraulics/" + params.NodeId + "_hydraulic_sim.csv"
    End If

    EnsureParentDir(cfg.OutputCsv)

    Local f:TStream = WriteFile(cfg.OutputCsv)
    If Not f Then
        Print "ERROR: Unable to open output CSV: " + cfg.OutputCsv
        Return 1
    End If

    ' RFC4180 header
    f.WriteLine("node_id,time_s,q_m3s,hlr_m_per_h,risk_surcharge_raw")

    Local state:TSimState = New TSimState
    state.TimeS = 0.0
    state.Q_M3s = params.QinM3s
    state.HLR_MpH = ComputeHLR(params, state.Q_M3s)
    state.SurchargeRisk = ComputeSurchargeRisk(params, state)

    Local i:Int
    For i = 0 Until cfg.Steps
        ' Write row
        f.WriteLine( _
            params.NodeId + "," + _
            FormatDouble(state.TimeS) + "," + _
            FormatDouble(state.Q_M3s) + "," + _
            FormatDouble(state.HLR_MpH) + "," + _
            FormatDouble(state.SurchargeRisk) )

        ' Advance
        state = StepHydraulics(params, cfg, state)
    Next

    f.Close()
    Print "Wrote hydraulic simulation CSV to: " + cfg.OutputCsv
    Return 0
End Function

Function LoadHydraulicParams:THydraulicParams()
    Local p:THydraulicParams = New THydraulicParams
    ' In practice you can parameterize via a small INI/JSON,
    ' but hard-coded defaults are fine for an initial research tool.
    p.NodeId = "HYDRO_RCH_TEST_01"
    p.LengthM = 250.0
    p.DiameterM = 0.9
    p.Roughness = 0.0003
    p.Slope = 0.002
    p.QinM3s = 0.25
    p.MaxHLR_MpH = 10.0
    p.MaxSurchargeRisk = 1.0
    Return p
End Function

Function LoadSimConfig:TSimConfig()
    Local c:TSimConfig = New TSimConfig
    c.DtSeconds = 60.0
    c.Steps = 24 * 60 / 1  ' one day at 1-min dt
    c.OutputCsv = ""       ' let Main choose default
    Return c
End Function

Function EnsureParentDir(path:String)
    Local dir:String = ExtractDir(path)
    If dir <> "" Then
        If Not FileType(dir) = FILETYPE_DIR Then
            CreateDir(dir, True)
        End If
    End If
End Function

Function FormatDouble:String(v:Double)
    ' Simple locale-independent formatting
    Return Replace(String(v), ",", "")
End Function

Function ComputeHLR:Double(p:THydraulicParams, q:Double)
    ' Hydraulic loading rate [m/h] = Q / area, converted to hours
    Local area:Double = 0.25 * Pi * p.DiameterM * p.DiameterM
    If area <= 0.0 Then Return 0.0
    Local velocity:Double = q / area           ' m/s
    Local hlr:Double = velocity * 3600.0       ' m/h
    Return hlr
End Function

Function ComputeSurchargeRisk:Double(p:THydraulicParams, st:TSimState)
    ' Very simple normalized risk: HLR vs MaxHLR and a soft penalty on high Q.
    If p.MaxHLR_MpH <= 0.0 Then Return 0.0
    Local r_hlr:Double = st.HLR_MpH / p.MaxHLR_MpH
    Local r_q:Double = st.Q_M3s / (2.0 * p.QinM3s)
    Local r:Double = r_hlr * 0.7 + r_q * 0.3
    If r < 0.0 Then r = 0.0
    If r > p.MaxSurchargeRisk Then r = p.MaxSurchargeRisk
    Return r
End Function

Function StepHydraulics:TSimState(p:THydraulicParams, cfg:TSimConfig, st:TSimState)
    Local next:TSimState = New TSimState

    ' Example: simple exponential decay of flow (clogging / off-peak)
    Local tau:Double = 6.0 * 3600.0   ' 6 hours characteristic time
    Local dq:Double = -(st.Q_M3s / tau) * cfg.DtSeconds

    next.Q_M3s = st.Q_M3s + dq
    If next.Q_M3s < 0.0 Then next.Q_M3s = 0.0

    next.TimeS = st.TimeS + cfg.DtSeconds
    next.HLR_MpH = ComputeHLR(p, next.Q_M3s)
    next.SurchargeRisk = ComputeSurchargeRisk(p, next)

    Return next
End Function
