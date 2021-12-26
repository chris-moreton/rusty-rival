pub mod magic_bitboards {
    use crate::types::types::{Bitboard, MagicVars, Square};

    pub fn magic(magic_vars: MagicVars, square: Square, to_squares_magic_index: u64) -> &Bitboard {
        return magic_vars.magic_moves.iter().nth(square as usize).unwrap().iter().nth(to_squares_magic_index as usize).unwrap();
    }

    pub const MAGIC_ROOK_VARS: MagicVars = MagicVars {
        occupancy_mask: OCCUPANCY_MASK_ROOK,
        magic_number: MAGIC_NUMBER_ROOK,
        magic_moves: MAGIC_MOVES_ROOK,
        magic_number_shifts: MAGIC_NUMBER_SHIFTS_ROOK
    };

    pub const MAGIC_BISHOP_VARS: MagicVars = MagicVars {
        occupancy_mask: OCCUPANCY_MASK_BISHOP,
        magic_number: MAGIC_NUMBER_BISHOP,
        magic_moves: MAGIC_MOVES_BISHOP,
        magic_number_shifts: MAGIC_NUMBER_SHIFTS_BISHOP
    };

    pub const OCCUPANCY_MASK_ROOK: Vec<Bitboard> = vec![
            0x101010101017e,
            0x202020202027c,
            0x404040404047a,
            0x8080808080876,
            0x1010101010106e,
            0x2020202020205e,
            0x4040404040403e,
            0x8080808080807e,
            0x1010101017e00,
            0x2020202027c00,
            0x4040404047a00,
            0x8080808087600,
            0x10101010106e00,
            0x20202020205e00,
            0x40404040403e00,
            0x80808080807e00,
            0x10101017e0100,
            0x20202027c0200,
            0x40404047a0400,
            0x8080808760800,
            0x101010106e1000,
            0x202020205e2000,
            0x404040403e4000,
            0x808080807e8000,
            0x101017e010100,
            0x202027c020200,
            0x404047a040400,
            0x8080876080800,
            0x1010106e101000,
            0x2020205e202000,
            0x4040403e404000,
            0x8080807e808000,
            0x1017e01010100,
            0x2027c02020200,
            0x4047a04040400,
            0x8087608080800,
            0x10106e10101000,
            0x20205e20202000,
            0x40403e40404000,
            0x80807e80808000,
            0x17e0101010100,
            0x27c0202020200,
            0x47a0404040400,
            0x8760808080800,
            0x106e1010101000,
            0x205e2020202000,
            0x403e4040404000,
            0x807e8080808000,
            0x7e010101010100,
            0x7c020202020200,
            0x7a040404040400,
            0x76080808080800,
            0x6e101010101000,
            0x5e202020202000,
            0x3e404040404000,
            0x7e808080808000,
            0x7e01010101010100,
            0x7c02020202020200,
            0x7a04040404040400,
            0x7608080808080800,
            0x6e10101010101000,
            0x5e20202020202000,
            0x3e40404040404000,
            0x7e80808080808000
      ];

}
