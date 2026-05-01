SuperStrict

' ---------------------------------------------------------
' File: tools/energy_mass/energy_per_mass_explorer.bmx
' Role: Non-actuating research tool to estimate energy
'       per mass removed (J/kg) for a single contaminant
'       and emit RFC4180 CSV for Rust/ALN ingestion.
' ---------------------------------------------------------

Framework brl.standardio
Import brl.filesystem
Import brl.math

Type TEnergyMassParams
    Field NodeId:String
    Field Contaminant:String
    Field Cin_MGperL:Double
    Field Cout_MGperL:Double
    Field Flow_M3s:Double
    Field Power_kW:Double
    Field DurationS:Double
End Type

Type TSimConfigEM
    Field Steps:Int
    Field OutputCsv:String
End Type

Function Main:Int()
    Local p:TEnergyMassParams = LoadEnergyMassParams()
    Local cfg:TSimConfigEM = LoadSimConfigEM()

    If cfg.OutputCsv = "" Then
        cfg.OutputCsv = "output/energy_mass/" + p.NodeId + "_energy_mass_sim.csv"
    End If
    EnsureParentDir(cfg.OutputCsv)

    Local f:TStream = WriteFile(cfg.OutputCsv)
    If Not f Then
        Print "ERROR: Unable to open output CSV: " + cfg.OutputCsv
        Return 1
    End If

    f.WriteLine("node_id,contaminant,time_s,energy_j,cin_mgL,cout_mgL,flow_m3s,mass_removed_kg,j_per_kg")

    Local dt:Double = p.DurationS / Double(cfg.Steps)
    Local t:Double = 0.0
    Local eAccum:Double = 0.0

    Local i:Int
    For i = 0 Until cfg.Steps
        ' Simple constant-power, constant-flow assumption per step.
        Local eStep:Double = p.Power_kW * 1000.0 * dt
        eAccum :+ eStep

        Local mRemovedKg:Double = MassRemovedStepKg(p, dt)

        Local jPerKg:Double
        If mRemovedKg > 0.0 Then
            jPerKg = eAccum / mRemovedKg
        Else
            jPerKg = 0.0
        End If

        f.WriteLine( _
            p.NodeId + "," + _
            p.Contaminant + "," + _
            FormatDouble(t) + "," + _
            FormatDouble(eAccum) + "," + _
            FormatDouble(p.Cin_MGperL) + "," + _
            FormatDouble(p.Cout_MGperL) + "," + _
            FormatDouble(p.Flow_M3s) + "," + _
            FormatDouble(mRemovedKg) + "," + _
            FormatDouble(jPerKg) )

        t :+ dt
    Next

    f.Close()
    Print "Wrote energy/mass simulation CSV to: " + cfg.OutputCsv
    Return 0
End Function

Function LoadEnergyMassParams:TEnergyMassParams()
    Local p:TEnergyMassParams = New TEnergyMassParams
    p.NodeId = "CEIM_NODE_TEST_01"
    p.Contaminant = "PFBS"
    p.Cin_MGperL = 0.010      ' 10 ug/L
    p.Cout_MGperL = 0.002     ' 2 ug/L
    p.Flow_M3s = 0.035
    p.Power_kW = 2.5
    p.DurationS = 3600.0      ' 1 hour
    Return p
End Function

Function LoadSimConfigEM:TSimConfigEM()
    Local c:TSimConfigEM = New TSimConfigEM
    c.Steps = 60
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

Function MassRemovedStepKg:Double(p:TEnergyMassParams, dt:Double)
    ' CEIM-style mass removed: (Cin - Cout)*Q*dt, with mg/L and m3/s.
    ' Convert mg/L to kg/m3: 1 mg/L = 1e-3 g/L = 1e-6 kg/mL ~ 1e-3 kg/m3.
    Local cin_kgm3:Double = p.Cin_MGperL * 1.0e-3
    Local cout_kgm3:Double = p.Cout_MGperL * 1.0e-3
    Local q:Double = p.Flow_M3s
    Local m:Double = (cin_kgm3 - cout_kgm3) * q * dt
    If m < 0.0 Then m = 0.0
    Return m
End Function
