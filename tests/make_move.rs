use std::mem;
use rusty_rival::fen::{get_position, move_from_algebraic_move};
use rusty_rival::make_move::{make_move};
use rusty_rival::move_constants::{PIECE_MASK_KING, PIECE_MASK_PAWN, PIECE_MASK_ROOK};
use rusty_rival::types::{Position};

#[test]
pub fn it_makes_a_move() {
    let mut new_position = mem::MaybeUninit::<Position>::uninit();

    unsafe {
        let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("e2e3".to_string(), PIECE_MASK_PAWN), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("e2e7".to_string(), PIECE_MASK_PAWN), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"rnbqkbnr/ppppPppp/8/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("e1g1".to_string(), PIECE_MASK_KING), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1RK1 b kq - 1 1".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("h1g1".to_string(), PIECE_MASK_ROOK), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK1R1 b kqQ - 1 1".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("e2e3".to_string(), PIECE_MASK_PAWN), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQK2R b KQkq - 0 1".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("e8c8".to_string(), PIECE_MASK_KING), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"2kr3r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQ - 1 2".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("e8d8".to_string(), PIECE_MASK_KING), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"r2k3r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQ - 1 2".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("h8g8".to_string(), PIECE_MASK_ROOK), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"r3k1r1/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQq - 1 2".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"2kr3r/pppppp1p/2n1b3/2bn1q2/4Pp2/8/PPPP1PPP/RNBQK2R b KQ e3 15 1".to_string());
        make_move(original_position, move_from_algebraic_move("f4e3".to_string(), PIECE_MASK_PAWN), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"2kr3r/pppppp1p/2n1b3/2bn1q2/8/4p3/PPPP1PPP/RNBQK2R w KQ - 0 2".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"2kr3r/ppppppPp/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK2R w KQ - 12 1".to_string());
        make_move(original_position, move_from_algebraic_move("g7h8r".to_string(), PIECE_MASK_PAWN), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"2kr3R/pppppp1p/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK2R b KQ - 0 1".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"2kr3R/pppp1p1p/2n1b3/2bn1q2/8/4p3/PPPP1PpP/RNBQK2R b KQ - 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("g2g1q".to_string(), PIECE_MASK_PAWN), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"2kr3R/pppp1p1p/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK1qR w KQ - 0 2".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("e2e4".to_string(), PIECE_MASK_PAWN), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("a7a6".to_string(), PIECE_MASK_PAWN), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"8/8/p7/1P6/K1k3pP/6P1/8/8 w - - 0 2".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("a7a5".to_string(), PIECE_MASK_PAWN), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"8/8/8/pP6/K1k3pP/6P1/8/8 w - a6 0 2".to_string()), *new_position.as_ptr());

        let original_position = &get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string());
        make_move(original_position, move_from_algebraic_move("c4c5".to_string(), PIECE_MASK_KING), &mut *new_position.as_mut_ptr());
        assert_eq!(get_position(&"8/p7/8/1Pk5/K5pP/6P1/8/8 w - - 1 2".to_string()), *new_position.as_ptr());
    }
}
