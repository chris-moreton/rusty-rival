use crate::types::{Bitboard, Mover, Piece, Position, PositionSupplement, Square};
use crate::types::Mover::White;

#[inline(always)]
pub const fn bit(i: Square) -> Bitboard {
    1 << i
}

#[inline(always)]
pub fn bitboard_for_mover(position: &Position, piece: Piece) -> Bitboard {
    bitboard_for_colour(position, &position.mover, piece)
}

#[inline(always)]
pub fn bitboard_for_colour(position: &Position, mover: &Mover, piece: Piece) -> Bitboard {
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
        _ => panic!("Can't handle piece")
    }
}

#[inline(always)]
pub fn slider_bitboard_for_colour(position: &Position, mover: &Mover, piece: &Piece) -> Bitboard {
    match (mover, piece) {
        (Mover::White, Piece::Rook) => position.white_rook_bitboard | position.white_queen_bitboard,
        (Mover::White, Piece::Bishop) => position.white_bishop_bitboard | position.white_queen_bitboard,
        (Mover::Black, Piece::Rook) => position.black_rook_bitboard | position.black_queen_bitboard,
        (Mover::Black, Piece::Bishop) => position.black_bishop_bitboard | position.black_queen_bitboard,
        _ => panic!("Can't handle piece")
    }
}

#[inline(always)]
pub fn clear_bit(bitboard: Bitboard, square: Square) -> Bitboard {
    bitboard & !bit(square)
}

#[inline(always)]
pub fn test_bit(bitboard: Bitboard, square: Square) -> bool {
    bitboard & bit(square) != 0
}

#[inline(always)]
pub fn enemy_bitboard(position: &Position, supplement: &PositionSupplement) -> Bitboard {
    if position.mover == White { supplement.black_pieces_bitboard } else { supplement.white_pieces_bitboard }
}

#[inline(always)]
pub fn south_fill(bb: Bitboard) -> Bitboard {
    let a = bb | (bb >> 8);
    let b = a | (a >> 16);
    b | (b >> 32)
}

#[inline(always)]
pub fn north_fill(bb: Bitboard) -> Bitboard {
    let a = bb | (bb << 8);
    let b = a | (a << 16);
    b | (b << 32)
}

#[inline(always)]
pub fn empty_squares_bitboard(position: &PositionSupplement) -> Bitboard {
    !position.all_pieces_bitboard
}

const fn every_eighth_bit_from(i: Square) -> Bitboard {
    if i < 8 {
        1 << i
    } else {
        (1 << i) | every_eighth_bit_from(i - 8)
    }
}

pub const A1_BIT: Square = 7;
pub const B1_BIT: Square = 6;
pub const C1_BIT: Square = 5;
pub const D1_BIT: Square = 4;
pub const E1_BIT: Square = 3;
pub const F1_BIT: Square = 2;
pub const G1_BIT: Square = 1;
pub const H1_BIT: Square = 0;

pub const A2_BIT: Square = A1_BIT + 8;
pub const B2_BIT: Square = B1_BIT + 8;
pub const C2_BIT: Square = C1_BIT + 8;
pub const D2_BIT: Square = D1_BIT + 8;
pub const E2_BIT: Square = E1_BIT + 8;
pub const F2_BIT: Square = F1_BIT + 8;
pub const G2_BIT: Square = G1_BIT + 8;
pub const H2_BIT: Square = H1_BIT + 8;

pub const A3_BIT: Square = A2_BIT + 8;
pub const B3_BIT: Square = B2_BIT + 8;
pub const C3_BIT: Square = C2_BIT + 8;
pub const D3_BIT: Square = D2_BIT + 8;
pub const E3_BIT: Square = E2_BIT + 8;
pub const F3_BIT: Square = F2_BIT + 8;
pub const G3_BIT: Square = G2_BIT + 8;
pub const H3_BIT: Square = H2_BIT + 8;

pub const A4_BIT: Square = A3_BIT + 8;
pub const B4_BIT: Square = B3_BIT + 8;
pub const C4_BIT: Square = C3_BIT + 8;
pub const D4_BIT: Square = D3_BIT + 8;
pub const E4_BIT: Square = E3_BIT + 8;
pub const F4_BIT: Square = F3_BIT + 8;
pub const G4_BIT: Square = G3_BIT + 8;
pub const H4_BIT: Square = H3_BIT + 8;

pub const A5_BIT: Square = A4_BIT + 8;
pub const B5_BIT: Square = B4_BIT + 8;
pub const C5_BIT: Square = C4_BIT + 8;
pub const D5_BIT: Square = D4_BIT + 8;
pub const E5_BIT: Square = E4_BIT + 8;
pub const F5_BIT: Square = F4_BIT + 8;
pub const G5_BIT: Square = G4_BIT + 8;
pub const H5_BIT: Square = H4_BIT + 8;

pub const A6_BIT: Square = A5_BIT + 8;
pub const B6_BIT: Square = B5_BIT + 8;
pub const C6_BIT: Square = C5_BIT + 8;
pub const D6_BIT: Square = D5_BIT + 8;
pub const E6_BIT: Square = E5_BIT + 8;
pub const F6_BIT: Square = F5_BIT + 8;
pub const G6_BIT: Square = G5_BIT + 8;
pub const H6_BIT: Square = H5_BIT + 8;

pub const A7_BIT: Square = A6_BIT + 8;
pub const B7_BIT: Square = B6_BIT + 8;
pub const C7_BIT: Square = C6_BIT + 8;
pub const D7_BIT: Square = D6_BIT + 8;
pub const E7_BIT: Square = E6_BIT + 8;
pub const F7_BIT: Square = F6_BIT + 8;
pub const G7_BIT: Square = G6_BIT + 8;
pub const H7_BIT: Square = H6_BIT + 8;

pub const A8_BIT: Square = A7_BIT + 8;
pub const B8_BIT: Square = B7_BIT + 8;
pub const C8_BIT: Square = C7_BIT + 8;
pub const D8_BIT: Square = D7_BIT + 8;
pub const E8_BIT: Square = E7_BIT + 8;
pub const F8_BIT: Square = F7_BIT + 8;
pub const G8_BIT: Square = G7_BIT + 8;
pub const H8_BIT: Square = H7_BIT + 8;

#[inline(always)]
pub fn exactly_one_bit_set(bb: Bitboard) -> bool {
    bb != 0 && bb & (bb - 1) == 0
}

pub const FILE_A_BITS: Bitboard = every_eighth_bit_from(A8_BIT);
pub const FILE_B_BITS: Bitboard = every_eighth_bit_from(B8_BIT);
pub const FILE_C_BITS: Bitboard = every_eighth_bit_from(C8_BIT);
pub const FILE_D_BITS: Bitboard = every_eighth_bit_from(D8_BIT);
pub const FILE_E_BITS: Bitboard = every_eighth_bit_from(E8_BIT);
pub const FILE_F_BITS: Bitboard = every_eighth_bit_from(F8_BIT);
pub const FILE_G_BITS: Bitboard = every_eighth_bit_from(G8_BIT);
pub const FILE_H_BITS: Bitboard = every_eighth_bit_from(H8_BIT);

pub const fn two_bits(bit1: Square, bit2: Square) -> Bitboard {
    let mut x: Bitboard = 0;
    x |= 1 << bit1;
    x |= 1 << bit2;
    x
}

pub const ALL_64_BITS_SET: Bitboard = 18446744073709551615;

pub const RANK_1_BITS: Bitboard = 0b0000000000000000000000000000000000000000000000000000000011111111;
pub const RANK_2_BITS: Bitboard = RANK_1_BITS << 8;
pub const RANK_3_BITS: Bitboard = RANK_2_BITS << 8;
pub const RANK_4_BITS: Bitboard = RANK_3_BITS << 8;
pub const RANK_5_BITS: Bitboard = RANK_4_BITS << 8;
pub const RANK_6_BITS: Bitboard = RANK_5_BITS << 8;
pub const RANK_7_BITS: Bitboard = RANK_6_BITS << 8;
pub const RANK_8_BITS: Bitboard = RANK_7_BITS << 8;

pub const F1G1_BITS: Bitboard = two_bits(F1_BIT, G1_BIT);
pub const G1H1_BITS: Bitboard = two_bits(G1_BIT, H1_BIT);
pub const A1B1_BITS: Bitboard = two_bits(A1_BIT, B1_BIT);
pub const B1C1_BITS: Bitboard = two_bits(B1_BIT, C1_BIT);
pub const F8G8_BITS: Bitboard = two_bits(F8_BIT, G8_BIT);
pub const G8H8_BITS: Bitboard = two_bits(G8_BIT, H8_BIT);
pub const A8B8_BITS: Bitboard = two_bits(A8_BIT, B8_BIT);
pub const B8C8_BITS: Bitboard = two_bits(B8_BIT, C8_BIT);

pub const MIDDLE_FILES_8_BIT: Bitboard = 0b0000000000000000000000000000000000000000000000000000000000011000;
pub const NONMID_FILES_8_BIT: Bitboard = 0b0000000000000000000000000000000000000000000000000000000011100111;

pub const LOW_32_BITS: Bitboard = RANK_1_BITS | RANK_2_BITS | RANK_3_BITS | RANK_4_BITS;

pub const DARK_SQUARES_BITS: Bitboard = 0b0101010110101010010101011010101001010101101010100101010110101010;
pub const LIGHT_SQUARES_BITS: Bitboard = !DARK_SQUARES_BITS;

pub const KNIGHT_MOVES_BITBOARDS: &[Bitboard] = &[
    0x20400, 0x50800, 0xa1100, 0x142200, 0x284400, 0x508800, 0xa01000, 0x402000,
    0x2040004, 0x5080008, 0xa110011, 0x14220022, 0x28440044, 0x50880088, 0xa0100010, 0x40200020,
    0x204000402, 0x508000805, 0xa1100110a, 0x1422002214, 0x2844004428, 0x5088008850, 0xa0100010a0, 0x4020002040,
    0x20400040200, 0x50800080500, 0xa1100110a00, 0x142200221400, 0x284400442800, 0x508800885000, 0xa0100010a000, 0x402000204000,
    0x2040004020000, 0x5080008050000, 0xa1100110a0000, 0x14220022140000, 0x28440044280000, 0x50880088500000, 0xa0100010a00000, 0x40200020400000,
    0x204000402000000, 0x508000805000000, 0xa1100110a000000,
    0b0001010000100010000000000010001000010100000000000000000000000000,
    0b0010100001000100000000000100010000101000000000000000000000000000,
    0b0101000010001000000000001000100001010000000000000000000000000000,
    -0x5fefffef60000000_i64 as u64,
    0b0100000000100000000000000010000001000000000000000000000000000000,
    0x400040200000000, 0x800080500000000, 0x1100110a00000000, 0x2200221400000000, 0x4400442800000000,
    -0x77ff77b000000000_i64 as u64,
    0x100010a000000000, 0x2000204000000000,
    0x4020000000000, 0x8050000000000, 0x110a0000000000, 0x22140000000000, 0x44280000000000, 0x88500000000000, 0x10a00000000000, 0x20400000000000
];

pub const KING_MOVES_BITBOARDS: &[Bitboard] = &[
    0x302,
    0x705,
    0xe0a,
    0x1c14,
    0x3828,
    0x7050,
    0xe0a0,
    0xc040,
    0x30203,
    0x70507,
    0xe0a0e,
    0x1c141c,
    0x382838,
    0x705070,
    0xe0a0e0,
    0xc040c0,
    0x3020300,
    0x7050700,
    0xe0a0e00,
    0x1c141c00,
    0x38283800,
    0x70507000,
    0xe0a0e000,
    0xc040c000,
    0x302030000,
    0x705070000,
    0xe0a0e0000,
    0x1c141c0000,
    0x3828380000,
    0x7050700000,
    0xe0a0e00000,
    0xc040c00000,
    0x30203000000,
    0x70507000000,
    0xe0a0e000000,
    0x1c141c000000,
    0x382838000000,
    0x705070000000,
    0xe0a0e0000000,
    0xc040c0000000,
    0x3020300000000,
    0x7050700000000,
    0xe0a0e00000000,
    0x1c141c00000000,
    0x38283800000000,
    0x70507000000000,
    0xe0a0e000000000,
    0xc040c000000000,
    0x302030000000000,
    0x705070000000000,
    0xe0a0e0000000000,
    0x1c141c0000000000,
    0x3828380000000000,
    0x7050700000000000,
    -0x1f5f200000000000_i64 as u64,
    -0x3fbf400000000000_i64 as u64,
    0x203000000000000,
    0x507000000000000,
    0xa0e000000000000,
    0x141c000000000000,
    0x2838000000000000,
    0x5070000000000000,
    -0x5f20000000000000_i64 as u64,
    0x40c0000000000000,
];

pub const WHITE_PAWN_MOVES_CAPTURE: &[Bitboard] = &[
    0x200, 0x500, 0xa00, 0x1400, 0x2800, 0x5000, 0xa000, 0x4000, 0x20000, 0x50000, 0xa0000, 0x140000, 0x280000, 0x500000, 0xa00000, 0x400000, 0x2000000, 0x5000000, 0xa000000, 0x14000000, 0x28000000, 0x50000000, 0xa0000000, 0x40000000, 0x200000000, 0x500000000, 0xa00000000, 0x1400000000, 0x2800000000, 0x5000000000, 0xa000000000, 0x4000000000, 0x20000000000, 0x50000000000, 0xa0000000000, 0x140000000000, 0x280000000000, 0x500000000000, 0xa00000000000, 0x400000000000, 0x2000000000000, 0x5000000000000, 0xa000000000000, 0x14000000000000, 0x28000000000000, 0x50000000000000, 0xa0000000000000, 0x40000000000000, 0x200000000000000, 0x500000000000000, 0xa00000000000000, 0x1400000000000000, 0x2800000000000000, 0x5000000000000000, -0x6000000000000000_i64 as u64, 0x4000000000000000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
];

pub const BLACK_PAWN_MOVES_CAPTURE: &[Bitboard] = &[
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x5, 0xa, 0x14, 0x28, 0x50, 0xa0, 0x40, 0x200, 0x500, 0xa00, 0x1400, 0x2800, 0x5000, 0xa000, 0x4000, 0x20000, 0x50000, 0xa0000, 0x140000, 0x280000, 0x500000, 0xa00000, 0x400000, 0x2000000, 0x5000000, 0xa000000, 0x14000000, 0x28000000, 0x50000000, 0xa0000000, 0x40000000, 0x200000000, 0x500000000, 0xa00000000, 0x1400000000, 0x2800000000, 0x5000000000, 0xa000000000, 0x4000000000, 0x20000000000, 0x50000000000, 0xa0000000000, 0x140000000000, 0x280000000000, 0x500000000000, 0xa00000000000, 0x400000000000, 0x2000000000000, 0x5000000000000, 0xa000000000000, 0x14000000000000, 0x28000000000000, 0x50000000000000, 0xa0000000000000, 0x40000000000000
];

pub const WHITE_PAWN_MOVES_FORWARD: &[Bitboard] = &[
    0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000, 0x4000, 0x8000, 0x10000, 0x20000, 0x40000, 0x80000, 0x100000, 0x200000, 0x400000, 0x800000, 0x1000000, 0x2000000, 0x4000000, 0x8000000, 0x10000000, 0x20000000, 0x40000000, 0x80000000, 0x100000000, 0x200000000, 0x400000000, 0x800000000, 0x1000000000, 0x2000000000, 0x4000000000, 0x8000000000, 0x10000000000, 0x20000000000, 0x40000000000, 0x80000000000, 0x100000000000, 0x200000000000, 0x400000000000, 0x800000000000, 0x1000000000000, 0x2000000000000, 0x4000000000000, 0x8000000000000, 0x10000000000000, 0x20000000000000, 0x40000000000000, 0x80000000000000, 0x100000000000000, 0x200000000000000, 0x400000000000000, 0x800000000000000, 0x1000000000000000, 0x2000000000000000, 0x4000000000000000, 1 << 63, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
];

pub const BLACK_PAWN_MOVES_FORWARD: &[Bitboard] = &[
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000, 0x4000, 0x8000, 0x10000, 0x20000, 0x40000, 0x80000, 0x100000, 0x200000, 0x400000, 0x800000, 0x1000000, 0x2000000, 0x4000000, 0x8000000, 0x10000000, 0x20000000, 0x40000000, 0x80000000, 0x100000000, 0x200000000, 0x400000000, 0x800000000, 0x1000000000, 0x2000000000, 0x4000000000, 0x8000000000, 0x10000000000, 0x20000000000, 0x40000000000, 0x80000000000, 0x100000000000, 0x200000000000, 0x400000000000, 0x800000000000, 0x1000000000000, 0x2000000000000, 0x4000000000000, 0x8000000000000, 0x10000000000000, 0x20000000000000, 0x40000000000000, 0x80000000000000,
];

pub const CASTLE_PRIV_WHITE_KING: Bitboard = 1;
pub const CASTLE_PRIV_WHITE_QUEEN: Bitboard = 2;
pub const CASTLE_PRIV_BLACK_KING: Bitboard = 4;
pub const CASTLE_PRIV_BLACK_QUEEN: Bitboard = 8;
pub const CASTLE_PRIV_BLACK_NONE: Bitboard = !CASTLE_PRIV_BLACK_KING & !CASTLE_PRIV_BLACK_QUEEN;
pub const CASTLE_PRIV_WHITE_NONE: Bitboard = !CASTLE_PRIV_WHITE_KING & !CASTLE_PRIV_WHITE_QUEEN;

pub const EMPTY_CASTLE_SQUARES_WHITE_KING: Bitboard = bit(1) | bit(2);
pub const EMPTY_CASTLE_SQUARES_WHITE_QUEEN: Bitboard = bit(4) | bit(5) | bit(6);
pub const EMPTY_CASTLE_SQUARES_BLACK_KING: Bitboard = bit(57) | bit(58);
pub const EMPTY_CASTLE_SQUARES_BLACK_QUEEN: Bitboard = bit(60) | bit(61) | bit(62);
pub const NO_CHECK_CASTLE_SQUARES_WHITE_KING: Bitboard = bit(2) | bit(3);
pub const NO_CHECK_CASTLE_SQUARES_WHITE_QUEEN: Bitboard = bit(3) | bit(4);
pub const NO_CHECK_CASTLE_SQUARES_BLACK_KING: Bitboard = bit(58) | bit(59);
pub const NO_CHECK_CASTLE_SQUARES_BLACK_QUEEN: Bitboard = bit(59) | bit(60);
