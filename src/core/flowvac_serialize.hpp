#pragma once
#include <vector>
#include "flowvac_state_wire.hpp"
#include "endianness.hpp"

inline std::vector<std::uint8_t> serialize_flowvac_state_be(
    const FlowVacStateWire& s)
{
    std::vector<std::uint8_t> buf;
    buf.reserve(sizeof(FlowVacStateWire));

    // Context and flag bytes
    buf.push_back(s.context);
    buf.push_back(s.qr_ok);
    buf.push_back(s.reserved_flags1);
    buf.push_back(s.reserved_flags2);

    auto put_double = [&](double v) {
        std::uint64_t be = encode_double_be(v);
        for (int i = 7; i >= 0; --i)
            buf.push_back(static_cast<std::uint8_t>((be >> (8 * i)) & 0xFF));
    };

    put_double(s.C_PFBS_ngL);
    put_double(s.C_Ecoli_MPN_100mL);
    put_double(s.C_TP_mgL);
    put_double(s.C_TDS_mgL);
    put_double(s.Cref_PFBS_ngL);
    put_double(s.Cref_Ecoli_MPN_100mL);
    put_double(s.Cref_TP_mgL);
    put_double(s.Cref_TDS_mgL);
    put_double(s.Q_m3s);
    put_double(s.v_ms);
    put_double(s.E_avail_kWh);
    put_double(s.Kn_delta);
    put_double(s.cpvm_value);
    put_double(s.bio_stress_index);
    put_double(s.qr_delta_mass);
    put_double(s.qr_delta_energy);

    // Reserved bytes
    for (std::uint8_t b : s.reserved) buf.push_back(b);

    return buf;
}

inline bool deserialize_flowvac_state_be(
    const std::uint8_t* data, std::size_t len, FlowVacStateWire& out)
{
    if (len < sizeof(FlowVacStateWire)) return false;
    std::size_t pos = 0;

    out.context         = data[pos++];
    out.qr_ok           = data[pos++];
    out.reserved_flags1 = data[pos++];
    out.reserved_flags2 = data[pos++];

    auto get_double = [&]() -> double {
        std::uint64_t be = 0;
        for (int i = 0; i < 8; ++i)
            be = (be << 8) | static_cast<std::uint64_t>(data[pos++]);
        return decode_double_be(be);
    };

    out.C_PFBS_ngL             = get_double();
    out.C_Ecoli_MPN_100mL      = get_double();
    out.C_TP_mgL               = get_double();
    out.C_TDS_mgL              = get_double();
    out.Cref_PFBS_ngL          = get_double();
    out.Cref_Ecoli_MPN_100mL   = get_double();
    out.Cref_TP_mgL            = get_double();
    out.Cref_TDS_mgL           = get_double();
    out.Q_m3s                  = get_double();
    out.v_ms                   = get_double();
    out.E_avail_kWh            = get_double();
    out.Kn_delta               = get_double();
    out.cpvm_value             = get_double();
    out.bio_stress_index       = get_double();
    out.qr_delta_mass          = get_double();
    out.qr_delta_energy        = get_double();

    for (std::size_t i = 0; i < sizeof(out.reserved); ++i)
        out.reserved[i] = data[pos++];

    return true;
}
