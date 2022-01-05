use rusty_rival::fen::{get_position, get_supplement};
use rusty_rival::moves::{allocate_magic_boxes};
use rusty_rival::perft::perft;

#[test]
#[ignore]
fn it_returns_the_total_number_of_moves_in_a_full_move_tree_of_a_given_depth_with_a_given_position_as_its_head_long_version() {
    let magic_box = &allocate_magic_boxes();

    // 11719118449 {

    let mut position = get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string());
    let mut supplement = get_supplement(position.clone());
    assert_eq!(perft(&mut position, 5, &mut supplement, magic_box), 11030083);

    let mut position = get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string());
    let mut supplement = get_supplement(position.clone());
    assert_eq!(perft(&mut position, 6, &mut supplement, magic_box), 178633661);

    let mut position = get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string());
    let mut supplement = get_supplement(position.clone());
    assert_eq!(perft(&mut position, 4, &mut supplement, magic_box), 193690690);

    let mut position = get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string());
    let mut supplement = get_supplement(position.clone());
    assert_eq!(perft(&mut position, 6, &mut supplement, magic_box), 178633661);

    let mut position = get_position(&"8/7p/p5pb/4k3/P1pPn3/8/P5PP/1rB2RK1 b - d3 0 28".to_string());
    let mut supplement = get_supplement(position.clone());
    assert_eq!(perft(&mut position, 5, &mut supplement, magic_box), 38633283);

    let mut position = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string());
    let mut supplement = get_supplement(position.clone());
    assert_eq!(perft(&mut position, 5, &mut supplement, magic_box), 77054993);

    let mut position = get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string());
    let mut supplement = get_supplement(position.clone());
    assert_eq!(perft(&mut position, 7, &mut supplement, magic_box), 3009794393);

    let mut position = get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string());
    let mut supplement = get_supplement(position.clone());
    assert_eq!(perft(&mut position, 5, &mut supplement, magic_box), 8031647685);

    // } 11,719,118,449
}
