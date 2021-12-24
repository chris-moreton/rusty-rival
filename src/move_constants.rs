pub mod move_constants {
    use crate::types::types::Move;

    pub const PROMOTION_QUEEN_MOVE_MASK: Move = 192;
    pub const PROMOTION_ROOK_MOVE_MASK: Move = 64;
    pub const PROMOTION_BISHOP_MOVE_MASK: Move = 128;
    pub const PROMOTION_KNIGHT_MOVE_MASK: Move = 256;
    pub const PROMOTION_FULL_MOVE_MASK: Move = 448;
    pub const EN_PASSANT_NOT_AVAILABLE: i8 = -1;

}
