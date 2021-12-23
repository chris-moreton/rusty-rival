use crate::types::types::{Bitboard, Position};

pub mod bitboards {
    use crate::types::types::{Bitboard, Mover, Piece, Position};
    use crate::types::types::Mover::White;

    pub fn bit_list(bb: Bitboard) -> Vec<u32> {
        let mut bl = bit_list(bb, Vec::new());
        bl.reverse();
        return bl;

        fn bit_list(bb: Bitboard, mut r: Vec<u32>) -> Vec<u32> {
            if bb == 0 {
                return r
            }
            let b = bb.trailing_zeros();
            let new_bb = bb & !(1 << b);
            r.push(b);
            return bit_list(new_bb, r);
        }
    }

    pub fn bit_string(bb: Bitboard) -> String {
        return bit_string(bb, 63, String::new());

        fn bit_string(bb: Bitboard, square: i8, s: String) -> String {
            if square == -1 {
                return s;
            }
            let bit_mask: u64 = 1 << square;
            let append_char = if bit_mask == bit_mask & bb { "1" } else { "0" };
            return bit_string(bb ^ bit_mask, square - 1, s + append_char);
        }
    }

    pub fn bitboard_for_mover(position: &Position, piece: &Piece) -> Bitboard {
        bitboard_for_colour(position, &position.mover, piece)
    }

    fn bitboard_for_colour(position: &Position, mover: &Mover, piece: &Piece) -> Bitboard {
        match (mover, piece) {
            (Mover::White, Piece::King) => position.white_king_bitboard,
            (Mover::White, Piece::Queen) => position.white_queen_bitboard,
            (Mover::White, Piece::Rook) => position.white_rook_bitboard,
            (Mover::White, Piece::Knight) => position.white_knight_bitboard,
            (Mover::White, Piece::Bishop) => position.white_bishop_bitboard,
            (Mover::White, Piece::Pawn) => position.white_pawn_bitboard,
            (Mover::Black, Piece::King) => position.black_king_bitboard,
            (Mover::Black, Piece::Queen) => position.black_queen_bitboard,
            (Mover::Black, Piece::Rook) => position.black_rook_bitboard,
            (Mover::Black, Piece::Knight) => position.black_knight_bitboard,
            (Mover::Black, Piece::Bishop) => position.black_bishop_bitboard,
            (Mover::Black, Piece::Pawn) => position.black_pawn_bitboard,
        }
    }

    pub fn slider_bitboard_for_colour(position: &Position, mover: &Mover, piece: &Piece) -> Bitboard {
        match (mover, piece) {
            (Mover::White, Piece::Rook) => position.white_rook_bitboard | position.white_queen_bitboard,
            (Mover::White, Piece::Bishop) => position.white_bishop_bitboard | position.white_queen_bitboard,
            (Mover::Black, Piece::Rook) => position.black_rook_bitboard | position.black_queen_bitboard,
            (Mover::Black, Piece::Bishop) => position.black_bishop_bitboard | position.black_queen_bitboard,
            _ => panic!("Can't handle piece")
        }
    }

    pub fn enemy_bitboard(position: &Position) -> Bitboard {
        return if position.mover == White { position.black_pieces_bitboard } else { position.white_pieces_bitboard }
    }

    pub fn south_fill(bb: Bitboard) -> Bitboard {
        let a = bb | (bb >> 8);
        let b = a | (a >> 16);
        let c = b | (b >> 32);
        return c;
    }

    pub fn north_fill(bb: Bitboard) -> Bitboard {
        let a = bb | (bb << 8);
        let b = a | (a << 16);
        let c = b | (b << 32);
        return c;
    }

    const PROMOTION_SQUARES: Bitboard = 0b1111111100000000000000000000000000000000000000000000000011111111;

    fn empty_squares_bitboard(position: &Position) -> Bitboard {
        return !position.all_pieces_bitboard;
    }

    const fn every_eighth_bit_from(i: u8) -> Bitboard {
        return if i < 8 {
            1 << i
        } else {
            (1 << i) | every_eighth_bit_from(i - 8)
        }
    }

    const A1_BIT: u8 = 7;
    const B1_BIT: u8 = 6;
    const C1_BIT: u8 = 5;
    const D1_BIT: u8 = 4;
    const E1_BIT: u8 = 3;
    const F1_BIT: u8 = 2;
    const G1_BIT: u8 = 1;
    const H1_BIT: u8 = 0;

    const A2_BIT: u8 = A1_BIT + 8;
    const B2_BIT: u8 = B1_BIT + 8;
    const C2_BIT: u8 = C1_BIT + 8;
    const D2_BIT: u8 = D1_BIT + 8;
    const E2_BIT: u8 = E1_BIT + 8;
    const F2_BIT: u8 = F1_BIT + 8;
    const G2_BIT: u8 = G1_BIT + 8;
    const H2_BIT: u8 = H1_BIT + 8;

    const A3_BIT: u8 = A2_BIT + 8;
    const B3_BIT: u8 = B2_BIT + 8;
    const C3_BIT: u8 = C2_BIT + 8;
    const D3_BIT: u8 = D2_BIT + 8;
    const E3_BIT: u8 = E2_BIT + 8;
    const F3_BIT: u8 = F2_BIT + 8;
    const G3_BIT: u8 = G2_BIT + 8;
    const H3_BIT: u8 = H2_BIT + 8;

    const A4_BIT: u8 = A3_BIT + 8;
    const B4_BIT: u8 = B3_BIT + 8;
    const C4_BIT: u8 = C3_BIT + 8;
    const D4_BIT: u8 = D3_BIT + 8;
    const E4_BIT: u8 = E3_BIT + 8;
    const F4_BIT: u8 = F3_BIT + 8;
    const G4_BIT: u8 = G3_BIT + 8;
    const H4_BIT: u8 = H3_BIT + 8;

    const A5_BIT: u8 = A4_BIT + 8;
    const B5_BIT: u8 = B4_BIT + 8;
    const C5_BIT: u8 = C4_BIT + 8;
    const D5_BIT: u8 = D4_BIT + 8;
    const E5_BIT: u8 = E4_BIT + 8;
    const F5_BIT: u8 = F4_BIT + 8;
    const G5_BIT: u8 = G4_BIT + 8;
    const H5_BIT: u8 = H4_BIT + 8;

    const A6_BIT: u8 = A5_BIT + 8;
    const B6_BIT: u8 = B5_BIT + 8;
    const C6_BIT: u8 = C5_BIT + 8;
    const D6_BIT: u8 = D5_BIT + 8;
    const E6_BIT: u8 = E5_BIT + 8;
    const F6_BIT: u8 = F5_BIT + 8;
    const G6_BIT: u8 = G5_BIT + 8;
    const H6_BIT: u8 = H5_BIT + 8;

    const A7_BIT: u8 = A6_BIT + 8;
    const B7_BIT: u8 = B6_BIT + 8;
    const C7_BIT: u8 = C6_BIT + 8;
    const D7_BIT: u8 = D6_BIT + 8;
    const E7_BIT: u8 = E6_BIT + 8;
    const F7_BIT: u8 = F6_BIT + 8;
    const G7_BIT: u8 = G6_BIT + 8;
    const H7_BIT: u8 = H6_BIT + 8;

    const A8_BIT: u8 = A7_BIT + 8;
    const B8_BIT: u8 = B7_BIT + 8;
    const C8_BIT: u8 = C7_BIT + 8;
    const D8_BIT: u8 = D7_BIT + 8;
    const E8_BIT: u8 = E7_BIT + 8;
    const F8_BIT: u8 = F7_BIT + 8;
    const G8_BIT: u8 = G7_BIT + 8;
    const H8_BIT: u8 = H7_BIT + 8;

    pub fn exactly_one_bit_set(bb: Bitboard) -> bool {
        return bb != 0 && bb & (bb - 1) == 0;
    }

    const FILE_A_BITS: Bitboard = every_eighth_bit_from(A8_BIT);
    const FILE_B_BITS: Bitboard = every_eighth_bit_from(B8_BIT);
    const FILE_C_BITS: Bitboard = every_eighth_bit_from(C8_BIT);
    const FILE_D_BITS: Bitboard = every_eighth_bit_from(D8_BIT);
    const FILE_E_BITS: Bitboard = every_eighth_bit_from(E8_BIT);
    const FILE_F_BITS: Bitboard = every_eighth_bit_from(F8_BIT);
    const FILE_G_BITS: Bitboard = every_eighth_bit_from(G8_BIT);
    const FILE_H_BITS: Bitboard = every_eighth_bit_from(H8_BIT);

    pub fn set_bits(is: Vec<u8>) -> Bitboard {
        let mut x: Bitboard = 0;
        is.iter().for_each(|i| {
            x = x | 1 << i;
        });
        return x;
    }

    pub const fn two_bits(bit1: u8, bit2: u8) -> Bitboard {
        let mut x: Bitboard = 0;
        x = x | 1 << bit1;
        x = x | 1 << bit2;
        return x;
    }

    const ALL_64_BITS_SET: Bitboard = 18446744073709551615;
    
    const RANK_1_BITS: Bitboard = 0b0000000000000000000000000000000000000000000000000000000011111111;
    const RANK_2_BITS: Bitboard = RANK_1_BITS << 8;
    const RANK_3_BITS: Bitboard = RANK_2_BITS << 8;
    const RANK_4_BITS: Bitboard = RANK_3_BITS << 8;
    const RANK_5_BITS: Bitboard = RANK_4_BITS << 8;
    const RANK_6_BITS: Bitboard = RANK_5_BITS << 8;
    const RANK_7_BITS: Bitboard = RANK_6_BITS << 8;
    const RANK_8_BITS: Bitboard = RANK_7_BITS << 8;
    
    const F1G1_BITS: Bitboard = two_bits(F1_BIT, G1_BIT);
    const G1H1_BITS: Bitboard = two_bits(G1_BIT, H1_BIT);
    const A1B1_BITS: Bitboard = two_bits(A1_BIT, B1_BIT);
    const B1C1_BITS: Bitboard = two_bits(B1_BIT, C1_BIT);
    const F8G8_BITS: Bitboard = two_bits(F8_BIT, G8_BIT);
    const G8H8_BITS: Bitboard = two_bits(G8_BIT, H8_BIT);
    const A8B8_BITS: Bitboard = two_bits(A8_BIT, B8_BIT);
    const B8C8_BITS: Bitboard = two_bits(B8_BIT, C8_BIT);

}




// middleFiles8Bit = setBits [d1Bit,e1Bit]
// nonMidFiles8Bit = setBits [a1Bit,b1Bit,c1Bit,f1Bit,g1Bit,h1Bit]
//
// low32Bits = (.|.) rank1Bits ((.|.) rank2Bits ((.|.) rank3Bits rank4Bits))
//
// darkSquaresBits :: Bitboard
// darkSquaresBits = setBits [a1Bit,a3Bit,a5Bit,a7Bit,b2Bit,b4Bit,b6Bit,b8Bit,c1Bit,c3Bit,c5Bit,c7Bit,d2Bit,d4Bit,d6Bit,d8Bit,e1Bit,e3Bit,e5Bit,e7Bit,f2Bit,f4Bit,f6Bit,f8Bit,g1Bit,g3Bit,g5Bit,g7Bit,h2Bit,h4Bit,h6Bit,h8Bit] :: Bitboard
// lightSquaresBits :: Bitboard
// lightSquaresBits = setBits [a2Bit,a4Bit,a6Bit,a8Bit,b1Bit,b3Bit,b5Bit,b7Bit,c2Bit,c4Bit,c6Bit,c8Bit,d1Bit,d3Bit,d5Bit,d7Bit,e2Bit,e4Bit,e6Bit,e8Bit,f1Bit,f3Bit,f5Bit,f7Bit,g2Bit,g4Bit,g6Bit,g8Bit,h1Bit,h3Bit,h5Bit,h7Bit] :: Bitboard
//
// knightMovesBitboards :: Int -> Bitboard
// knightMovesBitboards 0 = 0x20400
// knightMovesBitboards 1 = 0x50800
// knightMovesBitboards 2 = 0xa1100
// knightMovesBitboards 3 = 0x142200
// knightMovesBitboards 4 = 0x284400
// knightMovesBitboards 5 = 0x508800
// knightMovesBitboards 6 = 0xa01000
// knightMovesBitboards 7 = 0x402000
// knightMovesBitboards 8 = 0x2040004
// knightMovesBitboards 9 = 0x5080008
// knightMovesBitboards 10 = 0xa110011
// knightMovesBitboards 11 = 0x14220022
// knightMovesBitboards 12 = 0x28440044
// knightMovesBitboards 13 = 0x50880088
// knightMovesBitboards 14 = 0xa0100010
// knightMovesBitboards 15 = 0x40200020
// knightMovesBitboards 16 = 0x204000402
// knightMovesBitboards 17 = 0x508000805
// knightMovesBitboards 18 = 0xa1100110a
// knightMovesBitboards 19 = 0x1422002214
// knightMovesBitboards 20 = 0x2844004428
// knightMovesBitboards 21 = 0x5088008850
// knightMovesBitboards 22 = 0xa0100010a0
// knightMovesBitboards 23 = 0x4020002040
// knightMovesBitboards 24 = 0x20400040200
// knightMovesBitboards 25 = 0x50800080500
// knightMovesBitboards 26 = 0xa1100110a00
// knightMovesBitboards 27 = 0x142200221400
// knightMovesBitboards 28 = 0x284400442800
// knightMovesBitboards 29 = 0x508800885000
// knightMovesBitboards 30 = 0xa0100010a000
// knightMovesBitboards 31 = 0x402000204000
// knightMovesBitboards 32 = 0x2040004020000
// knightMovesBitboards 33 = 0x5080008050000
// knightMovesBitboards 34 = 0xa1100110a0000
// knightMovesBitboards 35 = 0x14220022140000
// knightMovesBitboards 36 = 0x28440044280000
// knightMovesBitboards 37 = 0x50880088500000
// knightMovesBitboards 38 = 0xa0100010a00000
// knightMovesBitboards 39 = 0x40200020400000
// knightMovesBitboards 40 = 0x204000402000000
// knightMovesBitboards 41 = 0x508000805000000
// knightMovesBitboards 42 = 0xa1100110a000000
// knightMovesBitboards 43 = 0x1422002214000000
// knightMovesBitboards 44 = 0x2844004428000000
// knightMovesBitboards 45 = 0x5088008850000000
// knightMovesBitboards 46 = -0x5fefffef60000000
// knightMovesBitboards 47 = 0x4020002040000000
// knightMovesBitboards 48 = 0x400040200000000
// knightMovesBitboards 49 = 0x800080500000000
// knightMovesBitboards 50 = 0x1100110a00000000
// knightMovesBitboards 51 = 0x2200221400000000
// knightMovesBitboards 52 = 0x4400442800000000
// knightMovesBitboards 53 = -0x77ff77b000000000
// knightMovesBitboards 54 = 0x100010a000000000
// knightMovesBitboards 55 = 0x2000204000000000
// knightMovesBitboards 56 = 0x4020000000000
// knightMovesBitboards 57 = 0x8050000000000
// knightMovesBitboards 58 = 0x110a0000000000
// knightMovesBitboards 59 = 0x22140000000000
// knightMovesBitboards 60 = 0x44280000000000
// knightMovesBitboards 61 = 0x88500000000000
// knightMovesBitboards 62 = 0x10a00000000000
// knightMovesBitboards 63 = 0x20400000000000
//
// kingMovesBitboards :: Int -> Bitboard
// kingMovesBitboards 0 = 0x302
// kingMovesBitboards 1 = 0x705
// kingMovesBitboards 2 = 0xe0a
// kingMovesBitboards 3 = 0x1c14
// kingMovesBitboards 4 = 0x3828
// kingMovesBitboards 5 = 0x7050
// kingMovesBitboards 6 = 0xe0a0
// kingMovesBitboards 7 = 0xc040
// kingMovesBitboards 8 = 0x30203
// kingMovesBitboards 9 = 0x70507
// kingMovesBitboards 10 = 0xe0a0e
// kingMovesBitboards 11 = 0x1c141c
// kingMovesBitboards 12 = 0x382838
// kingMovesBitboards 13 = 0x705070
// kingMovesBitboards 14 = 0xe0a0e0
// kingMovesBitboards 15 = 0xc040c0
// kingMovesBitboards 16 = 0x3020300
// kingMovesBitboards 17 = 0x7050700
// kingMovesBitboards 18 = 0xe0a0e00
// kingMovesBitboards 19 = 0x1c141c00
// kingMovesBitboards 20 = 0x38283800
// kingMovesBitboards 21 = 0x70507000
// kingMovesBitboards 22 = 0xe0a0e000
// kingMovesBitboards 23 = 0xc040c000
// kingMovesBitboards 24 = 0x302030000
// kingMovesBitboards 25 = 0x705070000
// kingMovesBitboards 26 = 0xe0a0e0000
// kingMovesBitboards 27 = 0x1c141c0000
// kingMovesBitboards 28 = 0x3828380000
// kingMovesBitboards 29 = 0x7050700000
// kingMovesBitboards 30 = 0xe0a0e00000
// kingMovesBitboards 31 = 0xc040c00000
// kingMovesBitboards 32 = 0x30203000000
// kingMovesBitboards 33 = 0x70507000000
// kingMovesBitboards 34 = 0xe0a0e000000
// kingMovesBitboards 35 = 0x1c141c000000
// kingMovesBitboards 36 = 0x382838000000
// kingMovesBitboards 37 = 0x705070000000
// kingMovesBitboards 38 = 0xe0a0e0000000
// kingMovesBitboards 39 = 0xc040c0000000
// kingMovesBitboards 40 = 0x3020300000000
// kingMovesBitboards 41 = 0x7050700000000
// kingMovesBitboards 42 = 0xe0a0e00000000
// kingMovesBitboards 43 = 0x1c141c00000000
// kingMovesBitboards 44 = 0x38283800000000
// kingMovesBitboards 45 = 0x70507000000000
// kingMovesBitboards 46 = 0xe0a0e000000000
// kingMovesBitboards 47 = 0xc040c000000000
// kingMovesBitboards 48 = 0x302030000000000
// kingMovesBitboards 49 = 0x705070000000000
// kingMovesBitboards 50 = 0xe0a0e0000000000
// kingMovesBitboards 51 = 0x1c141c0000000000
// kingMovesBitboards 52 = 0x3828380000000000
// kingMovesBitboards 53 = 0x7050700000000000
// kingMovesBitboards 54 = -0x1f5f200000000000
// kingMovesBitboards 55 = -0x3fbf400000000000
// kingMovesBitboards 56 = 0x203000000000000
// kingMovesBitboards 57 = 0x507000000000000
// kingMovesBitboards 58 = 0xa0e000000000000
// kingMovesBitboards 59 = 0x141c000000000000
// kingMovesBitboards 60 = 0x2838000000000000
// kingMovesBitboards 61 = 0x5070000000000000
// kingMovesBitboards 62 = -0x5f20000000000000
// kingMovesBitboards 63 = 0x40c0000000000000
//
// whitePawnMovesForward :: Int -> Bitboard
// whitePawnMovesForward 0 = 0x100
// whitePawnMovesForward 1 = 0x200
// whitePawnMovesForward 2 = 0x400
// whitePawnMovesForward 3 = 0x800
// whitePawnMovesForward 4 = 0x1000
// whitePawnMovesForward 5 = 0x2000
// whitePawnMovesForward 6 = 0x4000
// whitePawnMovesForward 7 = 0x8000
// whitePawnMovesForward 8 = 0x10000
// whitePawnMovesForward 9 = 0x20000
// whitePawnMovesForward 10 = 0x40000
// whitePawnMovesForward 11 = 0x80000
// whitePawnMovesForward 12 = 0x100000
// whitePawnMovesForward 13 = 0x200000
// whitePawnMovesForward 14 = 0x400000
// whitePawnMovesForward 15 = 0x800000
// whitePawnMovesForward 16 = 0x1000000
// whitePawnMovesForward 17 = 0x2000000
// whitePawnMovesForward 18 = 0x4000000
// whitePawnMovesForward 19 = 0x8000000
// whitePawnMovesForward 20 = 0x10000000
// whitePawnMovesForward 21 = 0x20000000
// whitePawnMovesForward 22 = 0x40000000
// whitePawnMovesForward 23 = 0x80000000
// whitePawnMovesForward 24 = 0x100000000
// whitePawnMovesForward 25 = 0x200000000
// whitePawnMovesForward 26 = 0x400000000
// whitePawnMovesForward 27 = 0x800000000
// whitePawnMovesForward 28 = 0x1000000000
// whitePawnMovesForward 29 = 0x2000000000
// whitePawnMovesForward 30 = 0x4000000000
// whitePawnMovesForward 31 = 0x8000000000
// whitePawnMovesForward 32 = 0x10000000000
// whitePawnMovesForward 33 = 0x20000000000
// whitePawnMovesForward 34 = 0x40000000000
// whitePawnMovesForward 35 = 0x80000000000
// whitePawnMovesForward 36 = 0x100000000000
// whitePawnMovesForward 37 = 0x200000000000
// whitePawnMovesForward 38 = 0x400000000000
// whitePawnMovesForward 39 = 0x800000000000
// whitePawnMovesForward 40 = 0x1000000000000
// whitePawnMovesForward 41 = 0x2000000000000
// whitePawnMovesForward 42 = 0x4000000000000
// whitePawnMovesForward 43 = 0x8000000000000
// whitePawnMovesForward 44 = 0x10000000000000
// whitePawnMovesForward 45 = 0x20000000000000
// whitePawnMovesForward 46 = 0x40000000000000
// whitePawnMovesForward 47 = 0x80000000000000
// whitePawnMovesForward 48 = 0x100000000000000
// whitePawnMovesForward 49 = 0x200000000000000
// whitePawnMovesForward 50 = 0x400000000000000
// whitePawnMovesForward 51 = 0x800000000000000
// whitePawnMovesForward 52 = 0x1000000000000000
// whitePawnMovesForward 53 = 0x2000000000000000
// whitePawnMovesForward 54 = 0x4000000000000000
// whitePawnMovesForward 55 = 1 `shiftL` 63
// whitePawnMovesForward 56 = 0x0
// whitePawnMovesForward 57 = 0x0
// whitePawnMovesForward 58 = 0x0
// whitePawnMovesForward 59 = 0x0
// whitePawnMovesForward 60 = 0x0
// whitePawnMovesForward 61 = 0x0
// whitePawnMovesForward 62 = 0x0
// whitePawnMovesForward 63 = 0x0
//
// whitePawnMovesCapture :: Int -> Bitboard
// whitePawnMovesCapture 0 = 0x200
// whitePawnMovesCapture 1 = 0x500
// whitePawnMovesCapture 2 = 0xa00
// whitePawnMovesCapture 3 = 0x1400
// whitePawnMovesCapture 4 = 0x2800
// whitePawnMovesCapture 5 = 0x5000
// whitePawnMovesCapture 6 = 0xa000
// whitePawnMovesCapture 7 = 0x4000
// whitePawnMovesCapture 8 = 0x20000
// whitePawnMovesCapture 9 = 0x50000
// whitePawnMovesCapture 10 = 0xa0000
// whitePawnMovesCapture 11 = 0x140000
// whitePawnMovesCapture 12 = 0x280000
// whitePawnMovesCapture 13 = 0x500000
// whitePawnMovesCapture 14 = 0xa00000
// whitePawnMovesCapture 15 = 0x400000
// whitePawnMovesCapture 16 = 0x2000000
// whitePawnMovesCapture 17 = 0x5000000
// whitePawnMovesCapture 18 = 0xa000000
// whitePawnMovesCapture 19 = 0x14000000
// whitePawnMovesCapture 20 = 0x28000000
// whitePawnMovesCapture 21 = 0x50000000
// whitePawnMovesCapture 22 = 0xa0000000
// whitePawnMovesCapture 23 = 0x40000000
// whitePawnMovesCapture 24 = 0x200000000
// whitePawnMovesCapture 25 = 0x500000000
// whitePawnMovesCapture 26 = 0xa00000000
// whitePawnMovesCapture 27 = 0x1400000000
// whitePawnMovesCapture 28 = 0x2800000000
// whitePawnMovesCapture 29 = 0x5000000000
// whitePawnMovesCapture 30 = 0xa000000000
// whitePawnMovesCapture 31 = 0x4000000000
// whitePawnMovesCapture 32 = 0x20000000000
// whitePawnMovesCapture 33 = 0x50000000000
// whitePawnMovesCapture 34 = 0xa0000000000
// whitePawnMovesCapture 35 = 0x140000000000
// whitePawnMovesCapture 36 = 0x280000000000
// whitePawnMovesCapture 37 = 0x500000000000
// whitePawnMovesCapture 38 = 0xa00000000000
// whitePawnMovesCapture 39 = 0x400000000000
// whitePawnMovesCapture 40 = 0x2000000000000
// whitePawnMovesCapture 41 = 0x5000000000000
// whitePawnMovesCapture 42 = 0xa000000000000
// whitePawnMovesCapture 43 = 0x14000000000000
// whitePawnMovesCapture 44 = 0x28000000000000
// whitePawnMovesCapture 45 = 0x50000000000000
// whitePawnMovesCapture 46 = 0xa0000000000000
// whitePawnMovesCapture 47 = 0x40000000000000
// whitePawnMovesCapture 48 = 0x200000000000000
// whitePawnMovesCapture 49 = 0x500000000000000
// whitePawnMovesCapture 50 = 0xa00000000000000
// whitePawnMovesCapture 51 = 0x1400000000000000
// whitePawnMovesCapture 52 = 0x2800000000000000
// whitePawnMovesCapture 53 = 0x5000000000000000
// whitePawnMovesCapture 54 = -0x6000000000000000
// whitePawnMovesCapture 55 = 0x4000000000000000
// whitePawnMovesCapture 56 = 0x0
// whitePawnMovesCapture 57 = 0x0
// whitePawnMovesCapture 58 = 0x0
// whitePawnMovesCapture 59 = 0x0
// whitePawnMovesCapture 60 = 0x0
// whitePawnMovesCapture 61 = 0x0
// whitePawnMovesCapture 62 = 0x0
// whitePawnMovesCapture 63 = 0x0
//
// blackPawnMovesForward :: Int -> Bitboard
// blackPawnMovesForward 0 = 0x0
// blackPawnMovesForward 1 = 0x0
// blackPawnMovesForward 2 = 0x0
// blackPawnMovesForward 3 = 0x0
// blackPawnMovesForward 4 = 0x0
// blackPawnMovesForward 5 = 0x0
// blackPawnMovesForward 6 = 0x0
// blackPawnMovesForward 7 = 0x0
// blackPawnMovesForward 8 = 0x1
// blackPawnMovesForward 9 = 0x2
// blackPawnMovesForward 10 = 0x4
// blackPawnMovesForward 11 = 0x8
// blackPawnMovesForward 12 = 0x10
// blackPawnMovesForward 13 = 0x20
// blackPawnMovesForward 14 = 0x40
// blackPawnMovesForward 15 = 0x80
// blackPawnMovesForward 16 = 0x100
// blackPawnMovesForward 17 = 0x200
// blackPawnMovesForward 18 = 0x400
// blackPawnMovesForward 19 = 0x800
// blackPawnMovesForward 20 = 0x1000
// blackPawnMovesForward 21 = 0x2000
// blackPawnMovesForward 22 = 0x4000
// blackPawnMovesForward 23 = 0x8000
// blackPawnMovesForward 24 = 0x10000
// blackPawnMovesForward 25 = 0x20000
// blackPawnMovesForward 26 = 0x40000
// blackPawnMovesForward 27 = 0x80000
// blackPawnMovesForward 28 = 0x100000
// blackPawnMovesForward 29 = 0x200000
// blackPawnMovesForward 30 = 0x400000
// blackPawnMovesForward 31 = 0x800000
// blackPawnMovesForward 32 = 0x1000000
// blackPawnMovesForward 33 = 0x2000000
// blackPawnMovesForward 34 = 0x4000000
// blackPawnMovesForward 35 = 0x8000000
// blackPawnMovesForward 36 = 0x10000000
// blackPawnMovesForward 37 = 0x20000000
// blackPawnMovesForward 38 = 0x40000000
// blackPawnMovesForward 39 = 0x80000000
// blackPawnMovesForward 40 = 0x100000000
// blackPawnMovesForward 41 = 0x200000000
// blackPawnMovesForward 42 = 0x400000000
// blackPawnMovesForward 43 = 0x800000000
// blackPawnMovesForward 44 = 0x1000000000
// blackPawnMovesForward 45 = 0x2000000000
// blackPawnMovesForward 46 = 0x4000000000
// blackPawnMovesForward 47 = 0x8000000000
// blackPawnMovesForward 48 = 0x10000000000
// blackPawnMovesForward 49 = 0x20000000000
// blackPawnMovesForward 50 = 0x40000000000
// blackPawnMovesForward 51 = 0x80000000000
// blackPawnMovesForward 52 = 0x100000000000
// blackPawnMovesForward 53 = 0x200000000000
// blackPawnMovesForward 54 = 0x400000000000
// blackPawnMovesForward 55 = 0x800000000000
// blackPawnMovesForward 56 = 0x1000000000000
// blackPawnMovesForward 57 = 0x2000000000000
// blackPawnMovesForward 58 = 0x4000000000000
// blackPawnMovesForward 59 = 0x8000000000000
// blackPawnMovesForward 60 = 0x10000000000000
// blackPawnMovesForward 61 = 0x20000000000000
// blackPawnMovesForward 62 = 0x40000000000000
// blackPawnMovesForward 63 = 0x80000000000000
//
// blackPawnMovesCapture :: Int -> Bitboard
// blackPawnMovesCapture 0 = 0x0
// blackPawnMovesCapture 1 = 0x0
// blackPawnMovesCapture 2 = 0x0
// blackPawnMovesCapture 3 = 0x0
// blackPawnMovesCapture 4 = 0x0
// blackPawnMovesCapture 5 = 0x0
// blackPawnMovesCapture 6 = 0x0
// blackPawnMovesCapture 7 = 0x0
// blackPawnMovesCapture 8 = 0x2
// blackPawnMovesCapture 9 = 0x5
// blackPawnMovesCapture 10 = 0xa
// blackPawnMovesCapture 11 = 0x14
// blackPawnMovesCapture 12 = 0x28
// blackPawnMovesCapture 13 = 0x50
// blackPawnMovesCapture 14 = 0xa0
// blackPawnMovesCapture 15 = 0x40
// blackPawnMovesCapture 16 = 0x200
// blackPawnMovesCapture 17 = 0x500
// blackPawnMovesCapture 18 = 0xa00
// blackPawnMovesCapture 19 = 0x1400
// blackPawnMovesCapture 20 = 0x2800
// blackPawnMovesCapture 21 = 0x5000
// blackPawnMovesCapture 22 = 0xa000
// blackPawnMovesCapture 23 = 0x4000
// blackPawnMovesCapture 24 = 0x20000
// blackPawnMovesCapture 25 = 0x50000
// blackPawnMovesCapture 26 = 0xa0000
// blackPawnMovesCapture 27 = 0x140000
// blackPawnMovesCapture 28 = 0x280000
// blackPawnMovesCapture 29 = 0x500000
// blackPawnMovesCapture 30 = 0xa00000
// blackPawnMovesCapture 31 = 0x400000
// blackPawnMovesCapture 32 = 0x2000000
// blackPawnMovesCapture 33 = 0x5000000
// blackPawnMovesCapture 34 = 0xa000000
// blackPawnMovesCapture 35 = 0x14000000
// blackPawnMovesCapture 36 = 0x28000000
// blackPawnMovesCapture 37 = 0x50000000
// blackPawnMovesCapture 38 = 0xa0000000
// blackPawnMovesCapture 39 = 0x40000000
// blackPawnMovesCapture 40 = 0x200000000
// blackPawnMovesCapture 41 = 0x500000000
// blackPawnMovesCapture 42 = 0xa00000000
// blackPawnMovesCapture 43 = 0x1400000000
// blackPawnMovesCapture 44 = 0x2800000000
// blackPawnMovesCapture 45 = 0x5000000000
// blackPawnMovesCapture 46 = 0xa000000000
// blackPawnMovesCapture 47 = 0x4000000000
// blackPawnMovesCapture 48 = 0x20000000000
// blackPawnMovesCapture 49 = 0x50000000000
// blackPawnMovesCapture 50 = 0xa0000000000
// blackPawnMovesCapture 51 = 0x140000000000
// blackPawnMovesCapture 52 = 0x280000000000
// blackPawnMovesCapture 53 = 0x500000000000
// blackPawnMovesCapture 54 = 0xa00000000000
// blackPawnMovesCapture 55 = 0x400000000000
// blackPawnMovesCapture 56 = 0x2000000000000
// blackPawnMovesCapture 57 = 0x5000000000000
// blackPawnMovesCapture 58 = 0xa000000000000
// blackPawnMovesCapture 59 = 0x14000000000000
// blackPawnMovesCapture 60 = 0x28000000000000
// blackPawnMovesCapture 61 = 0x50000000000000
// blackPawnMovesCapture 62 = 0xa0000000000000
// blackPawnMovesCapture 63 = 0x40000000000000
//
// whitePassedPawnMask :: Int -> Bitboard
// whitePassedPawnMask 0 = 0
// whitePassedPawnMask 1 = 0
// whitePassedPawnMask 2 = 0
// whitePassedPawnMask 3 = 0
// whitePassedPawnMask 4 = 0
// whitePassedPawnMask 5 = 0
// whitePassedPawnMask 6 = 0
// whitePassedPawnMask 7 = 0
// whitePassedPawnMask 8 = 0x0003030303030000
// whitePassedPawnMask 9 = 0x0007070707070000
// whitePassedPawnMask 10 = 0x000E0E0E0E0E0000
// whitePassedPawnMask 11 = 0x001C1C1C1C1C0000
// whitePassedPawnMask 12 = 0x0038383838380000
// whitePassedPawnMask 13 = 0x0070707070700000
// whitePassedPawnMask 14 = 0x00E0E0E0E0E00000
// whitePassedPawnMask 15 = 0x00C0C0C0C0C00000
// whitePassedPawnMask 16 = 0x0003030303000000
// whitePassedPawnMask 17 = 0x0007070707000000
// whitePassedPawnMask 18 = 0x000E0E0E0E000000
// whitePassedPawnMask 19 = 0x001C1C1C1C000000
// whitePassedPawnMask 20 = 0x0038383838000000
// whitePassedPawnMask 21 = 0x0070707070000000
// whitePassedPawnMask 22 = 0x00E0E0E0E0000000
// whitePassedPawnMask 23 = 0x00C0C0C0C0000000
// whitePassedPawnMask 24 = 0x0003030300000000
// whitePassedPawnMask 25 = 0x0007070700000000
// whitePassedPawnMask 26 = 0x000E0E0E00000000
// whitePassedPawnMask 27 = 0x001C1C1C00000000
// whitePassedPawnMask 28 = 0x0038383800000000
// whitePassedPawnMask 29 = 0x0070707000000000
// whitePassedPawnMask 30 = 0x00E0E0E000000000
// whitePassedPawnMask 31 = 0x00C0C0C000000000
// whitePassedPawnMask 32 = 0x0003030000000000
// whitePassedPawnMask 33 = 0x0007070000000000
// whitePassedPawnMask 34 = 0x000E0E0000000000
// whitePassedPawnMask 35 = 0x001C1C0000000000
// whitePassedPawnMask 36 = 0x0038380000000000
// whitePassedPawnMask 37 = 0x0070700000000000
// whitePassedPawnMask 38 = 0x00E0E00000000000
// whitePassedPawnMask 39 = 0x00C0C00000000000
// whitePassedPawnMask 40 = 0x0003000000000000
// whitePassedPawnMask 41 = 0x0007000000000000
// whitePassedPawnMask 42 = 0x000E000000000000
// whitePassedPawnMask 43 = 0x001C000000000000
// whitePassedPawnMask 44 = 0x0038000000000000
// whitePassedPawnMask 45 = 0x0070000000000000
// whitePassedPawnMask 46 = 0x00E0000000000000
// whitePassedPawnMask 47 = 0x00C0000000000000
// whitePassedPawnMask 48 = 0
// whitePassedPawnMask 49 = 0
// whitePassedPawnMask 50 = 0
// whitePassedPawnMask 51 = 0
// whitePassedPawnMask 52 = 0
// whitePassedPawnMask 53 = 0
// whitePassedPawnMask 54 = 0
// whitePassedPawnMask 55 = 0
// whitePassedPawnMask 56 = 0
// whitePassedPawnMask 57 = 0
// whitePassedPawnMask 58 = 0
// whitePassedPawnMask 59 = 0
// whitePassedPawnMask 60 = 0
// whitePassedPawnMask 61 = 0
// whitePassedPawnMask 62 = 0
// whitePassedPawnMask 63  = 0
//
// blackPassedPawnMask :: Int -> Bitboard
// blackPassedPawnMask 0 = 0
// blackPassedPawnMask 1 = 0
// blackPassedPawnMask 2 = 0
// blackPassedPawnMask 3 = 0
// blackPassedPawnMask 4 = 0
// blackPassedPawnMask 5 = 0
// blackPassedPawnMask 6 = 0
// blackPassedPawnMask 7 = 0
// blackPassedPawnMask 8 = 0
// blackPassedPawnMask 9 = 0
// blackPassedPawnMask 10 = 0
// blackPassedPawnMask 11 = 0
// blackPassedPawnMask 12 = 0
// blackPassedPawnMask 13 = 0
// blackPassedPawnMask 14 = 0
// blackPassedPawnMask 15 = 0
// blackPassedPawnMask 16 = 0x0000000000000300
// blackPassedPawnMask 17 = 0x0000000000000700
// blackPassedPawnMask 18 = 0x0000000000000E00
// blackPassedPawnMask 19 = 0x0000000000001C00
// blackPassedPawnMask 20 = 0x0000000000003800
// blackPassedPawnMask 21 = 0x0000000000007000
// blackPassedPawnMask 22 = 0x000000000000E000
// blackPassedPawnMask 23 = 0x000000000000C000
// blackPassedPawnMask 24 = 0x0000000000030300
// blackPassedPawnMask 25 = 0x0000000000070700
// blackPassedPawnMask 26 = 0x00000000000E0E00
// blackPassedPawnMask 27 = 0x00000000001C1C00
// blackPassedPawnMask 28 = 0x0000000000383800
// blackPassedPawnMask 29 = 0x0000000000707000
// blackPassedPawnMask 30 = 0x0000000000E0E000
// blackPassedPawnMask 31 = 0x0000000000C0C000
// blackPassedPawnMask 32 = 0x0000000003030300
// blackPassedPawnMask 33 = 0x0000000007070700
// blackPassedPawnMask 34 = 0x000000000E0E0E00
// blackPassedPawnMask 35 = 0x000000001C1C1C00
// blackPassedPawnMask 36 = 0x0000000038383800
// blackPassedPawnMask 37 = 0x0000000070707000
// blackPassedPawnMask 38 = 0x00000000E0E0E000
// blackPassedPawnMask 39 = 0x00000000C0C0C000
// blackPassedPawnMask 40 = 0x0000000303030300
// blackPassedPawnMask 41 = 0x0000000707070700
// blackPassedPawnMask 42 = 0x0000000E0E0E0E00
// blackPassedPawnMask 43 = 0x0000001C1C1C1C00
// blackPassedPawnMask 44 = 0x0000003838383800
// blackPassedPawnMask 45 = 0x0000007070707000
// blackPassedPawnMask 46 = 0x000000E0E0E0E000
// blackPassedPawnMask 47 = 0x000000C0C0C0C000
// blackPassedPawnMask 48 = 0x0000030303030300
// blackPassedPawnMask 49 = 0x0000070707070700
// blackPassedPawnMask 50 = 0x00000E0E0E0E0E00
// blackPassedPawnMask 51 = 0x00001C1C1C1C1C00
// blackPassedPawnMask 52 = 0x0000383838383800
// blackPassedPawnMask 53 = 0x0000707070707000
// blackPassedPawnMask 54 = 0x0000E0E0E0E0E000
// blackPassedPawnMask 55 = 0x0000C0C0C0C0C000
// blackPassedPawnMask 56 = 0
// blackPassedPawnMask 57 = 0
// blackPassedPawnMask 58 = 0
// blackPassedPawnMask 59 = 0
// blackPassedPawnMask 60 = 0
// blackPassedPawnMask 61 = 0
// blackPassedPawnMask 62 = 0
// blackPassedPawnMask 63  = 0
//
// castlePrivWhiteKing :: Bitboard
// castlePrivWhiteKing = 1
//
// castlePrivWhiteQueen :: Bitboard
// castlePrivWhiteQueen = 2
//
// castlePrivBlackKing :: Bitboard
// castlePrivBlackKing = 4
//
// castlePrivBlackQueen :: Bitboard
// castlePrivBlackQueen = 8
//
// castlePrivBlackNone :: Bitboard
// castlePrivBlackNone = (.&.) (complement castlePrivBlackKing) (complement castlePrivBlackQueen)
//
// castlePrivWhiteNone :: Bitboard
// castlePrivWhiteNone = (.&.) (complement castlePrivWhiteKing) (complement castlePrivWhiteQueen)
//
// emptyCastleSquaresWhiteKing :: Bitboard
// emptyCastleSquaresWhiteKing = (.|.) (1 `shiftL` 1) (1 `shiftL` 2)
//
// emptyCastleSquaresWhiteQueen :: Bitboard
// emptyCastleSquaresWhiteQueen = (.|.) (1 `shiftL` 4) ((.|.) (1 `shiftL` 5) (1 `shiftL` 6))
//
// emptyCastleSquaresBlackKing :: Bitboard
// emptyCastleSquaresBlackKing = (.|.) (1 `shiftL` 57) (1 `shiftL` 58)
//
// emptyCastleSquaresBlackQueen :: Bitboard
// emptyCastleSquaresBlackQueen = (.|.) (1 `shiftL` 62) ((.|.) (1 `shiftL` 61) (1 `shiftL` 60))
//
// noCheckCastleSquaresWhiteKing :: Bitboard
// noCheckCastleSquaresWhiteKing = (.|.) (1 `shiftL` 2) (1 `shiftL` 3)
//
// noCheckCastleSquaresWhiteQueen :: Bitboard
// noCheckCastleSquaresWhiteQueen = (.|.) (1 `shiftL` 3) (1 `shiftL` 4)
//
// noCheckCastleSquaresBlackKing :: Bitboard
// noCheckCastleSquaresBlackKing = (.|.) (1 `shiftL` 58) (1 `shiftL` 59)
//
// noCheckCastleSquaresBlackQueen :: Bitboard
// noCheckCastleSquaresBlackQueen = (.|.) (1 `shiftL` 59) (1 `shiftL` 60)
