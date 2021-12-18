pub use types::bitboard_for_mover;
pub use types::Position;
pub use types::Piece;

#[test]
fn it_returns_the_correct_bitboard_for_mover() {
    let p1 = Position {
        white_pawn_bitboard: 1,
        white_knight_bitboard: 2,
        white_bishop_bitboard: 3,
        white_queen_bitboard: 4,
        white_king_bitboard: 5,
        white_rook_bitboard: 6,
        black_pawn_bitboard: 7,
        black_knight_bitboard: 8,
        black_bishop_bitboard: 9,
        black_queen_bitboard: 10,
        black_king_bitboard: 11,
        black_rook_bitboard: 12,
        all_pieces_bitboard: 13,
        white_pieces_bitboard: 14,
        black_pieces_bitboard: 15,
        mover: Mover::White,
        en_passant_square: 1,
        white_king_castle_available: true,
        black_king_castle_available: true,
        white_queen_castle_available: true,
        black_queen_castle_available: true,
        half_moves: 0,
        move_number: 0,
    };

    assert_eq!(1, bitboard_for_mover(&p1, Piece::Pawn));
    assert_eq!(2, bitboard_for_mover(&p1, Piece::Knight));
    assert_eq!(3, bitboard_for_mover(&p1, Piece::Bishop));
    assert_eq!(4, bitboard_for_mover(&p1, Piece::Queen));
    assert_eq!(5, bitboard_for_mover(&p1, Piece::King));
    assert_eq!(6, bitboard_for_mover(&p1, Piece::Rook));

    let p2 = Position {
        white_pawn_bitboard: 1,
        white_knight_bitboard: 2,
        white_bishop_bitboard: 3,
        white_queen_bitboard: 4,
        white_king_bitboard: 5,
        white_rook_bitboard: 6,
        black_pawn_bitboard: 7,
        black_knight_bitboard: 8,
        black_bishop_bitboard: 9,
        black_queen_bitboard: 10,
        black_king_bitboard: 11,
        black_rook_bitboard: 12,
        all_pieces_bitboard: 13,
        white_pieces_bitboard: 14,
        black_pieces_bitboard: 15,
        mover: Mover::Black,
        en_passant_square: 1,
        white_king_castle_available: true,
        black_king_castle_available: true,
        white_queen_castle_available: true,
        black_queen_castle_available: true,
        half_moves: 0,
        move_number: 0,
    };

    assert_eq!(7, bitboard_for_mover(&p2, Piece::Pawn));
    assert_eq!(8, bitboard_for_mover(&p2, Piece::Knight));
    assert_eq!(9, bitboard_for_mover(&p2, Piece::Bishop));
    assert_eq!(10, bitboard_for_mover(&p2, Piece::Queen));
    assert_eq!(11, bitboard_for_mover(&p2, Piece::King));
    assert_eq!(12, bitboard_for_mover(&p2, Piece::Rook));
}