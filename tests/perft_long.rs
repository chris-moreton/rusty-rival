use rusty_rival::fen::get_position;
use rusty_rival::perft::perft;

#[test]
fn it_returns_the_total_number_of_moves_in_a_full_move_tree_of_a_given_depth_with_a_given_position_as_its_head_long_version() {

    // 11719118449 {

    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 5), 11030083);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 6), 178633661);
    assert_eq!(perft(&mut get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 4), 193690690);
    assert_eq!(perft(&mut get_position(&"8/7p/p5pb/4k3/P1pPn3/8/P5PP/1rB2RK1 b - d3 0 28".to_string()), 5), 38633283);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 5), 77054993);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 7), 3009794393);
    // assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 6), 178633661);
    // assert_eq!(perft(&mut get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 5), 8031647685);

    // } 11719118449
}