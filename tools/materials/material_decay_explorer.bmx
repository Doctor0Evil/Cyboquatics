SuperStrict

' ---------------------------------------------------------
' File: tools/materials/material_decay_explorer.bmx
' Role: Non-actuating research tool to explore simple
'       first-order material decay and export trajectories
'       as RFC4180 CSV for Rust/ALN material kernels.
' ---------------------------------------------------------

Framework brl.standardio
Import brl.filesystem
Import brl.math

Type TMaterialParams
    Field SubstrateId:String
    Field InitialMassKg:Double
    Field DecayRatePerDay:Double    ' k in 1/day
    Field MicroResidueFrac:Double   ' fraction of lost mass becoming micro-residue
    Field HorizonDays:Double
End Type

Type TSimConfigMat
    Field Steps:Int
    Field OutputCsv:String
End Type

Type TMatState
    Field TimeDays:Double
    Field MassKg:Double
    Field LostMassKg:Double
    Field MicroResidueKg:Double
End Type

Function Main:Int()
    Local p:TMaterialParams = LoadMaterialParams()
    Local cfg:TSimConfigMat = LoadSimConfigMat()

    If cfg.OutputCsv = "" Then
        cfg.OutputCsv = "output/materials/" + p.SubstrateId + "_decay_sim.csv"
    End If
    EnsureParentDir(cfg.OutputCsv)

    Local f:TStream = WriteFile(cfg.OutputCsv)
    If Not f Then
        Print "ERROR: Unable to open output CSV: " + cfg.OutputCsv
        Return 1
    End If

    f.WriteLine("substrate_id,time_days,mass_kg,lost_mass_kg,micro_residue_kg,fraction_remaining")

    Local dtDays:Double = p.HorizonDays / Double(cfg.Steps)
    Local st:TMatState = New TMatState
    st.TimeDays = 0.0
    st.MassKg = p.InitialMassKg
    st.LostMassKg = 0.0
    st.MicroResidueKg = 0.0

    Local i:Int
    For i = 0 Until cfg.Steps
        Local fracRemain:Double = 0.0
        If p.InitialMassKg > 0.0 Then
            fracRemain = st.MassKg / p.InitialMassKg
        End If

        f.WriteLine( _
            p.SubstrateId + "," + _
            FormatDouble(st.TimeDays) + "," + _
            FormatDouble(st.MassKg) + "," + _
            FormatDouble(st.LostMassKg) + "," + _
            FormatDouble(st.MicroResidueKg) + "," + _
            FormatDouble(fracRemain) )

        st = StepDecay(p, st, dtDays)
    Next

    f.Close()
    Print "Wrote material decay simulation CSV to: " + cfg.OutputCsv
    Return 0
End Function

Function LoadMaterialParams:TMaterialParams()
    Local p:TMaterialParams = New TMaterialParams
    p.SubstrateId = "FLOWVAC_SUBSTRATE_TEST_01"
    p.InitialMassKg = 10.0
    p.DecayRatePerDay = 0.02         ' ~ t90 around 115 days
    p.MicroResidueFrac = 0.05
    p.HorizonDays = 365.0
    Return p
End Function

Function LoadSimConfigMat:TSimConfigMat()
    Local c:TSimConfigMat = New TSimConfigMat
    c.Steps = 365
    c.OutputCsv = ""
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
    Return Replace(String(v), ",", "")
End Function

Function StepDecay:TMatState(p:TMaterialParams, st:TMatState, dtDays:Double)
    Local next:TMatState = New TMatState
    next.TimeDays = st.TimeDays + dtDays

    ' First-order decay: dM/dt = -k M
    Local k:Double = p.DecayRatePerDay
    Local decay:Double = st.MassKg * (1.0 - Exp(-k * dtDays))

    If decay < 0.0 Then decay = 0.0
    If decay > st.MassKg Then decay = st.MassKg

    next.MassKg = st.MassKg - decay
    next.LostMassKg = st.LostMassKg + decay

    Local microAdd:Double = decay * p.MicroResidueFrac
    next.MicroResidueKg = st.MicroResidueKg + microAdd

    Return next
End Function
