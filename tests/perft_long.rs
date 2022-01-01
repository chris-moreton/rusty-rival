use rusty_rival::fen::fen::get_position;
use rusty_rival::make_move::make_move::{default_position_history, make_move, switch_side};
use rusty_rival::moves::moves::{is_check, moves};
use rusty_rival::perft::perft;
use rusty_rival::types::types::Mover::White;
use rusty_rival::types::types::Position;

#[test]
fn it_returns_the_total_number_of_moves_in_a_full_move_tree_of_a_given_depth_with_a_given_position_as_its_head_long_version() {
    let mut history = default_position_history();

    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 4, &mut history), 3186478);

    // assert_eq!(perft(&mut get_position(&"rnbqkb1r/ppppp1pp/7n/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3".to_string()), 4, &mut history), 11139762);
    //
    // assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 3, &mut history), 43238);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 2, &mut history), 97862);
    // assert_eq!(perft(&mut get_position(&"5k2/5p1p/p3B1p1/P5P1/3K1P1P/8/8/8 b - - 0 1".to_string()), 3, &mut history), 20541);
    // assert_eq!(perft(&mut get_position(&"n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1".to_string()), 3, &mut history), 182838);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/1R2K2R b Kkq - 0 1".to_string()), 3, &mut history), 175927);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2R1K2R b Kkq - 0 1".to_string()), 3, &mut history), 178248);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/3RK2R b Kkq - 0 1".to_string()), 3, &mut history), 168357);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P4K1P/R6R b kq - 0 1".to_string()), 3, &mut history), 221267);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P2K3P/R6R b kq - 0 1".to_string()), 3, &mut history), 213344);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R2K3R b kq - 0 1".to_string()), 3, &mut history), 120873);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2KR3R b kq - 0 1".to_string()), 3, &mut history), 184127);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K1R1 b Qkq - 0 1".to_string()), 3, &mut history), 240619);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3KR2 b Qkq - 0 1".to_string()), 3, &mut history), 189825);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/Ppp1p3/3b4/7P/R3K2R b KQkq a3 0 1".to_string()), 3, &mut history), 154828);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/P2b4/7P/R3K2R b KQkq - 0 1".to_string()), 3, &mut history), 173400);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p2P/3b4/P7/R3K2R b KQkq - 0 1".to_string()), 3, &mut history), 165129);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b3P/P7/R3K2R b KQkq - 0 1".to_string()), 3, &mut history), 151137);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/1B6/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 3, &mut history), 249845);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p1B4p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 3, &mut history), 227059);
    // assert_eq!(perft(&mut get_position(&"r2Bk2r/p6p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 3, &mut history), 185525);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/8/1Bp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 3, &mut history), 186968);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 1, &mut history), 341);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 2, &mut history), 6666);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 3, &mut history), 150072);
    // assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string()), 6, &mut history), 986637);
    // assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - 0 1".to_string()), 6, &mut history), 966152);
    // assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - 0 1".to_string()), 7, &mut history), 8103790);
    // assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 6, &mut history), 178633661);
    // assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 7, &mut history), 3009794393);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 4, &mut history), 193690690);
    // assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 6, &mut history), 178633661);
    // assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 4, &mut history), 674624);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 3, &mut history), 4085603);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2KR3R b kq - 0 1".to_string()), 4, &mut history), 4238116);
    // assert_eq!(perft(&mut get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()), 4, &mut history), 4865609);
    // assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 5, &mut history), 11030083);
    // assert_eq!(perft(&mut get_position(&"8/7p/p5pb/4k3/P1pPn3/8/P5PP/1rB2RK1 b - d3 0 28".to_string()), 5, &mut history), 38633283);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 5, &mut history), 77054993);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 5, &mut history), 8031647685);
}