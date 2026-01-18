#pragma once
#include <cstdint>
#include <cstring>

// Detect endianness at compile-time if possible
constexpr bool is_little_endian() {
    const std::uint16_t v = 0x0102;
    return reinterpret_cast<const std::uint8_t*>(&v)[0] == 0x02;
}

inline std::uint64_t to_be64(std::uint64_t x) {
    if (!is_little_endian()) return x;
    return ((x & 0x00000000000000FFULL) << 56) |
           ((x & 0x000000000000FF00ULL) << 40) |
           ((x & 0x0000000000FF0000ULL) << 24) |
           ((x & 0x00000000FF000000ULL) << 8)  |
           ((x & 0x000000FF00000000ULL) >> 8)  |
           ((x & 0x0000FF0000000000ULL) >> 24) |
           ((x & 0x00FF000000000000ULL) >> 40) |
           ((x & 0xFF00000000000000ULL) >> 56);
}

inline std::uint64_t from_be64(std::uint64_t x) {
    return to_be64(x); // symmetric
}

inline std::uint64_t encode_double_be(double v) {
    std::uint64_t u;
    static_assert(sizeof(u) == sizeof(v), "double size mismatch");
    std::memcpy(&u, &v, sizeof(double));
    return to_be64(u);
}

inline double decode_double_be(std::uint64_t be) {
    std::uint64_t u = from_be64(be);
    double v;
    std::memcpy(&v, &u, sizeof(double));
    return v;
}
