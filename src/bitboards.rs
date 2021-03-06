use crate::move_constants::EN_PASSANT_NOT_AVAILABLE;
use crate::types::{Bitboard, Mover, Piece, Position, Square, BLACK, WHITE};

#[inline(always)]
pub const fn bit(i: Square) -> Bitboard {
    1 << i
}

#[inline(always)]
pub const fn epsbit(i: Square) -> Bitboard {
    if i == EN_PASSANT_NOT_AVAILABLE {
        0
    } else {
        1 << i
    }
}

#[inline(always)]
pub fn bitboard_for_mover(position: &Position, piece: Piece) -> Bitboard {
    bitboard_for_colour(position, position.mover, piece)
}

#[inline(always)]
pub fn bitboard_for_colour(position: &Position, mover: Mover, piece: Piece) -> Bitboard {
    match (mover, piece) {
        (WHITE, Piece::King) => bit(position.pieces[WHITE as usize].king_square),
        (WHITE, Piece::Queen) => position.pieces[WHITE as usize].queen_bitboard,
        (WHITE, Piece::Rook) => position.pieces[WHITE as usize].rook_bitboard,
        (WHITE, Piece::Knight) => position.pieces[WHITE as usize].knight_bitboard,
        (WHITE, Piece::Bishop) => position.pieces[WHITE as usize].bishop_bitboard,
        (WHITE, Piece::Pawn) => position.pieces[WHITE as usize].pawn_bitboard,
        (BLACK, Piece::King) => bit(position.pieces[BLACK as usize].king_square),
        (BLACK, Piece::Queen) => position.pieces[BLACK as usize].queen_bitboard,
        (BLACK, Piece::Rook) => position.pieces[BLACK as usize].rook_bitboard,
        (BLACK, Piece::Knight) => position.pieces[BLACK as usize].knight_bitboard,
        (BLACK, Piece::Bishop) => position.pieces[BLACK as usize].bishop_bitboard,
        (BLACK, Piece::Pawn) => position.pieces[BLACK as usize].pawn_bitboard,
        _ => panic!("Can't handle piece"),
    }
}

#[inline(always)]
pub fn slider_bitboard_for_colour(position: &Position, mover: Mover, piece: &Piece) -> Bitboard {
    match (mover, piece) {
        (WHITE, Piece::Rook) => position.pieces[WHITE as usize].rook_bitboard | position.pieces[WHITE as usize].queen_bitboard,
        (WHITE, Piece::Bishop) => position.pieces[WHITE as usize].bishop_bitboard | position.pieces[WHITE as usize].queen_bitboard,
        (BLACK, Piece::Rook) => position.pieces[BLACK as usize].rook_bitboard | position.pieces[BLACK as usize].queen_bitboard,
        (BLACK, Piece::Bishop) => position.pieces[BLACK as usize].bishop_bitboard | position.pieces[BLACK as usize].queen_bitboard,
        _ => panic!("Can't handle piece"),
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

pub static KNIGHT_MOVES_BITBOARDS: &[Bitboard] = &[
    0x20400,
    0x50800,
    0xa1100,
    0x142200,
    0x284400,
    0x508800,
    0xa01000,
    0x402000,
    0x2040004,
    0x5080008,
    0xa110011,
    0x14220022,
    0x28440044,
    0x50880088,
    0xa0100010,
    0x40200020,
    0x204000402,
    0x508000805,
    0xa1100110a,
    0x1422002214,
    0x2844004428,
    0x5088008850,
    0xa0100010a0,
    0x4020002040,
    0x20400040200,
    0x50800080500,
    0xa1100110a00,
    0x142200221400,
    0x284400442800,
    0x508800885000,
    0xa0100010a000,
    0x402000204000,
    0x2040004020000,
    0x5080008050000,
    0xa1100110a0000,
    0x14220022140000,
    0x28440044280000,
    0x50880088500000,
    0xa0100010a00000,
    0x40200020400000,
    0x204000402000000,
    0x508000805000000,
    0xa1100110a000000,
    0b0001010000100010000000000010001000010100000000000000000000000000,
    0b0010100001000100000000000100010000101000000000000000000000000000,
    0b0101000010001000000000001000100001010000000000000000000000000000,
    -0x5fefffef60000000_i64 as u64,
    0b0100000000100000000000000010000001000000000000000000000000000000,
    0x400040200000000,
    0x800080500000000,
    0x1100110a00000000,
    0x2200221400000000,
    0x4400442800000000,
    -0x77ff77b000000000_i64 as u64,
    0x100010a000000000,
    0x2000204000000000,
    0x4020000000000,
    0x8050000000000,
    0x110a0000000000,
    0x22140000000000,
    0x44280000000000,
    0x88500000000000,
    0x10a00000000000,
    0x20400000000000,
];

pub static KING_MOVES_BITBOARDS: &[Bitboard] = &[
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

pub static WHITE_PAWN_MOVES_CAPTURE: [Bitboard; 64] = [
    0x200,
    0x500,
    0xa00,
    0x1400,
    0x2800,
    0x5000,
    0xa000,
    0x4000,
    0x20000,
    0x50000,
    0xa0000,
    0x140000,
    0x280000,
    0x500000,
    0xa00000,
    0x400000,
    0x2000000,
    0x5000000,
    0xa000000,
    0x14000000,
    0x28000000,
    0x50000000,
    0xa0000000,
    0x40000000,
    0x200000000,
    0x500000000,
    0xa00000000,
    0x1400000000,
    0x2800000000,
    0x5000000000,
    0xa000000000,
    0x4000000000,
    0x20000000000,
    0x50000000000,
    0xa0000000000,
    0x140000000000,
    0x280000000000,
    0x500000000000,
    0xa00000000000,
    0x400000000000,
    0x2000000000000,
    0x5000000000000,
    0xa000000000000,
    0x14000000000000,
    0x28000000000000,
    0x50000000000000,
    0xa0000000000000,
    0x40000000000000,
    0x200000000000000,
    0x500000000000000,
    0xa00000000000000,
    0x1400000000000000,
    0x2800000000000000,
    0x5000000000000000,
    -0x6000000000000000_i64 as u64,
    0x4000000000000000,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
];

pub static BLACK_PAWN_MOVES_CAPTURE: [Bitboard; 64] = [
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x2,
    0x5,
    0xa,
    0x14,
    0x28,
    0x50,
    0xa0,
    0x40,
    0x200,
    0x500,
    0xa00,
    0x1400,
    0x2800,
    0x5000,
    0xa000,
    0x4000,
    0x20000,
    0x50000,
    0xa0000,
    0x140000,
    0x280000,
    0x500000,
    0xa00000,
    0x400000,
    0x2000000,
    0x5000000,
    0xa000000,
    0x14000000,
    0x28000000,
    0x50000000,
    0xa0000000,
    0x40000000,
    0x200000000,
    0x500000000,
    0xa00000000,
    0x1400000000,
    0x2800000000,
    0x5000000000,
    0xa000000000,
    0x4000000000,
    0x20000000000,
    0x50000000000,
    0xa0000000000,
    0x140000000000,
    0x280000000000,
    0x500000000000,
    0xa00000000000,
    0x400000000000,
    0x2000000000000,
    0x5000000000000,
    0xa000000000000,
    0x14000000000000,
    0x28000000000000,
    0x50000000000000,
    0xa0000000000000,
    0x40000000000000,
];

pub static WHITE_PAWN_MOVES_FORWARD: [Bitboard; 64] = [
    0x100,
    0x200,
    0x400,
    0x800,
    0x1000,
    0x2000,
    0x4000,
    0x8000,
    0x10000,
    0x20000,
    0x40000,
    0x80000,
    0x100000,
    0x200000,
    0x400000,
    0x800000,
    0x1000000,
    0x2000000,
    0x4000000,
    0x8000000,
    0x10000000,
    0x20000000,
    0x40000000,
    0x80000000,
    0x100000000,
    0x200000000,
    0x400000000,
    0x800000000,
    0x1000000000,
    0x2000000000,
    0x4000000000,
    0x8000000000,
    0x10000000000,
    0x20000000000,
    0x40000000000,
    0x80000000000,
    0x100000000000,
    0x200000000000,
    0x400000000000,
    0x800000000000,
    0x1000000000000,
    0x2000000000000,
    0x4000000000000,
    0x8000000000000,
    0x10000000000000,
    0x20000000000000,
    0x40000000000000,
    0x80000000000000,
    0x100000000000000,
    0x200000000000000,
    0x400000000000000,
    0x800000000000000,
    0x1000000000000000,
    0x2000000000000000,
    0x4000000000000000,
    1 << 63,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
];

pub static BLACK_PAWN_MOVES_FORWARD: [Bitboard; 64] = [
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x1,
    0x2,
    0x4,
    0x8,
    0x10,
    0x20,
    0x40,
    0x80,
    0x100,
    0x200,
    0x400,
    0x800,
    0x1000,
    0x2000,
    0x4000,
    0x8000,
    0x10000,
    0x20000,
    0x40000,
    0x80000,
    0x100000,
    0x200000,
    0x400000,
    0x800000,
    0x1000000,
    0x2000000,
    0x4000000,
    0x8000000,
    0x10000000,
    0x20000000,
    0x40000000,
    0x80000000,
    0x100000000,
    0x200000000,
    0x400000000,
    0x800000000,
    0x1000000000,
    0x2000000000,
    0x4000000000,
    0x8000000000,
    0x10000000000,
    0x20000000000,
    0x40000000000,
    0x80000000000,
    0x100000000000,
    0x200000000000,
    0x400000000000,
    0x800000000000,
    0x1000000000000,
    0x2000000000000,
    0x4000000000000,
    0x8000000000000,
    0x10000000000000,
    0x20000000000000,
    0x40000000000000,
    0x80000000000000,
];

pub static ROOK_RAYS: [Bitboard; 64] = [
    72340172838076927,
    144680345676153599,
    289360691352306943,
    578721382704613631,
    1157442765409227007,
    2314885530818453759,
    4629771061636907263,
    9259542123273814271,
    72340172838141951,
    144680345676218114,
    289360691352370948,
    578721382704676616,
    1157442765409287952,
    2314885530818510624,
    4629771061636955968,
    9259542123273846656,
    72340172854787841,
    144680345692733954,
    289360691368756228,
    578721382720800776,
    1157442765424889872,
    2314885530833068064,
    4629771061649424448,
    9259542123282137216,
    72340177116135681,
    144680349920788994,
    289360695563387908,
    578721386848585736,
    1157442769418981392,
    2314885534559772704,
    4629771064841355328,
    9259542125404520576,
    72341268021182721,
    144681432302879234,
    289361769389097988,
    578722443561535496,
    1157443791906410512,
    2314886488596160544,
    4629771881975660608,
    9259542668734660736,
    72620539713224961,
    144958522117980674,
    289636668770878468,
    578992962076674056,
    1157705548688265232,
    2315130721911447584,
    4629981068357812288,
    9259681761250541696,
    144114092876038401,
    215893514783949314,
    360010910506681348,
    648245701952145416,
    1224715284843073552,
    2377654450624929824,
    4683532782188642368,
    9295289445316067456,
    18446463702556279041,
    18375251637271921154,
    18375816794872218628,
    18376947110072813576,
    18379207740474003472,
    18383729001276383264,
    18392771522881142848,
    18410856566090662016,
];

pub static BISHOP_RAYS: [Bitboard; 64] = [
    9241421688590303745,
    36099303471056130,
    141012904249860,
    550848566280,
    6480472080,
    1108177604640,
    283691315142720,
    72624976668147841,
    4620710844295151874,
    9241421688590369285,
    36099303487964170,
    141017232967700,
    1659000852520,
    283693466787920,
    72624976676536480,
    145249953336295746,
    2310355422147576452,
    4620710844311930120,
    9241421692918827537,
    36100411639731234,
    424704218245188,
    72625527497707656,
    145249955483787280,
    290499906672607780,
    1155177711073920072,
    2310355426442807312,
    4620711952397242656,
    9241705379771195969,
    108724279870768258,
    145390965703608324,
    290500456430440456,
    580999813349385240,
    577588855570712624,
    1155178810653020192,
    2310639096282816576,
    4693335786603561344,
    9386671573207122433,
    326599072704627714,
    581140551354550276,
    1161999627766208536,
    288794436426018864,
    577870347820343360,
    1227798289695391872,
    2455596579390849024,
    4911193158781632768,
    9822386317546488321,
    1198028557088457730,
    2323999528796559396,
    144399430222622792,
    360856452331487360,
    721712908957941760,
    1443425817932595200,
    2886851635848478720,
    5773703267401990400,
    11547405435292353025,
    4648069013213553730,
    72765989572331652,
    145531428313006080,
    291063956137574400,
    582127916553338880,
    1164255828828487680,
    2328510558145413120,
    4656739641314115840,
    9314046665258451585,
];

pub const WHITE_PASSED_PAWN_MASK: [Bitboard; 64] = [
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0x0003030303030000,
    0x0007070707070000,
    0x000E0E0E0E0E0000,
    0x001C1C1C1C1C0000,
    0x0038383838380000,
    0x0070707070700000,
    0x00E0E0E0E0E00000,
    0x00C0C0C0C0C00000,
    0x0003030303000000,
    0x0007070707000000,
    0x000E0E0E0E000000,
    0x001C1C1C1C000000,
    0x0038383838000000,
    0x0070707070000000,
    0x00E0E0E0E0000000,
    0x00C0C0C0C0000000,
    0x0003030300000000,
    0x0007070700000000,
    0x000E0E0E00000000,
    0x001C1C1C00000000,
    0x0038383800000000,
    0x0070707000000000,
    0x00E0E0E000000000,
    0x00C0C0C000000000,
    0x0003030000000000,
    0x0007070000000000,
    0x000E0E0000000000,
    0x001C1C0000000000,
    0x0038380000000000,
    0x0070700000000000,
    0x00E0E00000000000,
    0x00C0C00000000000,
    0x0003000000000000,
    0x0007000000000000,
    0x000E000000000000,
    0x001C000000000000,
    0x0038000000000000,
    0x0070000000000000,
    0x00E0000000000000,
    0x00C0000000000000,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

pub const BLACK_PASSED_PAWN_MASK: [Bitboard; 64] = [
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0x0000000000000300,
    0x0000000000000700,
    0x0000000000000E00,
    0x0000000000001C00,
    0x0000000000003800,
    0x0000000000007000,
    0x000000000000E000,
    0x000000000000C000,
    0x0000000000030300,
    0x0000000000070700,
    0x00000000000E0E00,
    0x00000000001C1C00,
    0x0000000000383800,
    0x0000000000707000,
    0x0000000000E0E000,
    0x0000000000C0C000,
    0x0000000003030300,
    0x0000000007070700,
    0x000000000E0E0E00,
    0x000000001C1C1C00,
    0x0000000038383800,
    0x0000000070707000,
    0x00000000E0E0E000,
    0x00000000C0C0C000,
    0x0000000303030300,
    0x0000000707070700,
    0x0000000E0E0E0E00,
    0x0000001C1C1C1C00,
    0x0000003838383800,
    0x0000007070707000,
    0x000000E0E0E0E000,
    0x000000C0C0C0C000,
    0x0000030303030300,
    0x0000070707070700,
    0x00000E0E0E0E0E00,
    0x00001C1C1C1C1C00,
    0x0000383838383800,
    0x0000707070707000,
    0x0000E0E0E0E0E000,
    0x0000C0C0C0C0C000,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

pub static BAD_BISHOP_PAWN_SQUARES_WHITE: [Bitboard; 64] = [
    bit(G2_BIT) | bit(F3_BIT),
    bit(F2_BIT) | bit(E3_BIT) | bit(H2_BIT),
    bit(E2_BIT) | bit(D3_BIT) | bit(F2_BIT) | bit(H3_BIT),
    bit(D2_BIT) | bit(C3_BIT) | bit(F2_BIT) | bit(G3_BIT),
    bit(C2_BIT) | bit(B3_BIT) | bit(E2_BIT) | bit(F3_BIT),
    bit(B2_BIT) | bit(A3_BIT) | bit(D2_BIT) | bit(E3_BIT),
    bit(A2_BIT) | bit(C2_BIT) | bit(D3_BIT),
    bit(B2_BIT) | bit(C3_BIT),
    bit(G3_BIT) | bit(F4_BIT),
    bit(F3_BIT) | bit(E4_BIT) | bit(H3_BIT),
    bit(E3_BIT) | bit(D4_BIT) | bit(F3_BIT) | bit(H4_BIT),
    bit(D3_BIT) | bit(C4_BIT) | bit(F3_BIT) | bit(G4_BIT),
    bit(C3_BIT) | bit(B4_BIT) | bit(E3_BIT) | bit(F4_BIT),
    bit(B3_BIT) | bit(A4_BIT) | bit(D3_BIT) | bit(E4_BIT),
    bit(A3_BIT) | bit(C3_BIT) | bit(D4_BIT),
    bit(B3_BIT) | bit(C4_BIT),
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

pub static BAD_BISHOP_PAWN_SQUARES_BLACK: [Bitboard; 64] = [
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    bit(G6_BIT) | bit(F5_BIT),
    bit(F6_BIT) | bit(E5_BIT) | bit(H6_BIT),
    bit(E6_BIT) | bit(D5_BIT) | bit(F6_BIT) | bit(H5_BIT),
    bit(D6_BIT) | bit(C5_BIT) | bit(F6_BIT) | bit(G5_BIT),
    bit(C6_BIT) | bit(B5_BIT) | bit(E6_BIT) | bit(F5_BIT),
    bit(B6_BIT) | bit(A5_BIT) | bit(D6_BIT) | bit(E5_BIT),
    bit(A6_BIT) | bit(C6_BIT) | bit(D5_BIT),
    bit(B6_BIT) | bit(C4_BIT),
    bit(G7_BIT) | bit(F6_BIT),
    bit(F7_BIT) | bit(E6_BIT) | bit(H7_BIT),
    bit(E7_BIT) | bit(D6_BIT) | bit(F7_BIT) | bit(H6_BIT),
    bit(D7_BIT) | bit(C6_BIT) | bit(F7_BIT) | bit(G6_BIT),
    bit(C7_BIT) | bit(B6_BIT) | bit(E7_BIT) | bit(F6_BIT),
    bit(B7_BIT) | bit(A6_BIT) | bit(D7_BIT) | bit(E6_BIT),
    bit(A7_BIT) | bit(C7_BIT) | bit(D6_BIT),
    bit(B7_BIT) | bit(C6_BIT),
];

pub static PAWN_MOVES_FORWARD: [[Bitboard; 64]; 2] = [WHITE_PAWN_MOVES_FORWARD, BLACK_PAWN_MOVES_FORWARD];
pub static PAWN_MOVES_CAPTURE: [[Bitboard; 64]; 2] = [WHITE_PAWN_MOVES_CAPTURE, BLACK_PAWN_MOVES_CAPTURE];
pub static DOUBLE_MOVE_RANK_BITS: [Bitboard; 2] = [RANK_4_BITS, RANK_5_BITS];
pub static EN_PASSANT_CAPTURE_RANK: [Bitboard; 2] = [RANK_6_BITS, RANK_3_BITS];

pub static CASTLE_PRIV_WHITE_KING: Bitboard = 1;
pub static CASTLE_PRIV_WHITE_QUEEN: Bitboard = 2;
pub static CASTLE_PRIV_BLACK_KING: Bitboard = 4;
pub static CASTLE_PRIV_BLACK_QUEEN: Bitboard = 8;
pub static CASTLE_PRIV_BLACK_NONE: Bitboard = !CASTLE_PRIV_BLACK_KING & !CASTLE_PRIV_BLACK_QUEEN;
pub static CASTLE_PRIV_WHITE_NONE: Bitboard = !CASTLE_PRIV_WHITE_KING & !CASTLE_PRIV_WHITE_QUEEN;

pub static EMPTY_CASTLE_SQUARES_WHITE_KING: Bitboard = bit(1) | bit(2);
pub static EMPTY_CASTLE_SQUARES_WHITE_QUEEN: Bitboard = bit(4) | bit(5) | bit(6);
pub static EMPTY_CASTLE_SQUARES_BLACK_KING: Bitboard = bit(57) | bit(58);
pub static EMPTY_CASTLE_SQUARES_BLACK_QUEEN: Bitboard = bit(60) | bit(61) | bit(62);

pub static NO_CHECK_CASTLE_SQUARES_WHITE_KING: Bitboard = bit(2) | bit(3);
pub static NO_CHECK_CASTLE_SQUARES_WHITE_QUEEN: Bitboard = bit(3) | bit(4);
pub static NO_CHECK_CASTLE_SQUARES_BLACK_KING: Bitboard = bit(58) | bit(59);
pub static NO_CHECK_CASTLE_SQUARES_BLACK_QUEEN: Bitboard = bit(59) | bit(60);

pub static EMPTY_CASTLE_SQUARES: [[Bitboard; 2]; 2] = [
    [EMPTY_CASTLE_SQUARES_WHITE_KING, EMPTY_CASTLE_SQUARES_BLACK_KING],
    [EMPTY_CASTLE_SQUARES_WHITE_QUEEN, EMPTY_CASTLE_SQUARES_BLACK_QUEEN],
];
pub static NO_CHECK_CASTLE_SQUARES: [[Bitboard; 2]; 2] = [
    [NO_CHECK_CASTLE_SQUARES_WHITE_KING, NO_CHECK_CASTLE_SQUARES_BLACK_KING],
    [NO_CHECK_CASTLE_SQUARES_WHITE_QUEEN, NO_CHECK_CASTLE_SQUARES_BLACK_QUEEN],
];
