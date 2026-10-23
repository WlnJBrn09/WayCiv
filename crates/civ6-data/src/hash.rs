//! Civilization VI's type-name hash.
//!
//! Type names such as `"COMBAT_MELEE"` are identified everywhere by this hash.
//! It forwards to the engine's `Crypto::CRC::CalcString(str, 0xffffffff)`:
//! a reflected CRC-32 (polynomial `0xEDB88320`) with initial value
//! `0xffffffff` and no final XOR, i.e. the bitwise NOT of zlib's `crc32`.

const fn crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
            bit += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

static CRC32_TABLE: [u32; 256] = crc32_table();

/// Hashes a type name exactly as the game does. Usable in `const` contexts.
pub const fn make_hash(name: &str) -> u32 {
    let bytes = name.as_bytes();
    let mut crc = 0xffff_ffffu32;
    let mut i = 0;
    while i < bytes.len() {
        crc = CRC32_TABLE[((crc ^ bytes[i] as u32) & 0xff) as usize] ^ (crc >> 8);
        i += 1;
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::make_hash;

    #[test]
    fn matches_enum_values_from_dwarf() {
        assert_eq!(make_hash("COMBAT_MELEE"), 748_940_753);
        assert_eq!(make_hash("COMBAT_RANGED"), 784_649_805);
        assert_eq!(make_hash("COMBAT_BOMBARD"), 1_338_578_493);
        assert_eq!(make_hash("COMBAT_AIR"), 1_184_946_373);
        assert_eq!(make_hash("COMBAT_ICBM"), 1_640_240_290);
        assert_eq!(make_hash("COMBAT_RELIGIOUS"), 1_580_168_296);
        assert_eq!(make_hash("DISTRICT_DEFENSE_OUTER"), 1_839_557_181);
        assert_eq!(make_hash("DISTRICT_DEFENSE_GARRISON"), 1_587_009_065);
    }

    #[test]
    fn empty_string_is_the_initial_value() {
        assert_eq!(make_hash(""), 0xffff_ffff);
    }
}
