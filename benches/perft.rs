use rusty_rival::fen::get_position;
use rusty_rival::make_move::{default_position_history};
use rusty_rival::moves::{allocate_magic_boxes};
use rusty_rival::perft::perft;

#[bench]
fn it_returns_the_total_number_of_moves_in_a_full_move_tree_of_a_given_depth_with_a_given_position_as_its_head() {
    let mut history = default_position_history();
    let magic_box = &allocate_magic_boxes();

    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string()), 0, &mut history, magic_box), 8);
    assert_eq!(perft(&mut get_position(&"5k2/5p1p/p3B1p1/P5P1/3K1P1P/8/8/8 b - - 0 1".to_string()), 0, &mut history, magic_box), 9);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 0, &mut history, magic_box), 14);
    assert_eq!(perft(&mut get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()), 0, &mut history, magic_box), 20);
    assert_eq!(perft(&mut get_position(&"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string()), 0, &mut history, magic_box), 20);

    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 0, &mut history, magic_box), 17);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3KR2 b Qkq - 0 1".to_string()), 0, &mut history, magic_box), 17);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/8/1Bp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 0, &mut history, magic_box), 19);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K1R1 b Qkq - 0 1".to_string()), 0, &mut history, magic_box), 19);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/1R2K2R b Kkq - 0 1".to_string()), 0, &mut history, magic_box), 20);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2R1K2R b Kkq - 0 1".to_string()), 0, &mut history, magic_box), 20);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/3RK2R b Kkq - 0 1".to_string()), 0, &mut history, magic_box), 20);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P4K1P/R6R b kq - 0 1".to_string()), 0, &mut history, magic_box), 20);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p2P/3b4/P7/R3K2R b KQkq - 0 1".to_string()), 0, &mut history, magic_box), 20);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b3P/P7/R3K2R b KQkq - 0 1".to_string()), 0, &mut history, magic_box), 20);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P2K3P/R6R b kq - 0 1".to_string()), 0, &mut history, magic_box), 20);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R2K3R b kq - 0 1".to_string()), 0, &mut history, magic_box), 20);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2KR3R b kq - 0 1".to_string()), 0, &mut history, magic_box), 20);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/Ppp1p3/3b4/7P/R3K2R b KQkq a3 0 1".to_string()), 0, &mut history, magic_box), 21);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/P2b4/7P/R3K2R b KQkq - 0 1".to_string()), 0, &mut history, magic_box), 21);
    assert_eq!(perft(&mut get_position(&"r3k2r/p1B4p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 0, &mut history, magic_box), 21);
    assert_eq!(perft(&mut get_position(&"r2Bk2r/p6p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 0, &mut history, magic_box), 21);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/1B6/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 0, &mut history, magic_box), 22);

    assert_eq!(perft(&mut get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()), 1, &mut history, magic_box), 400);

    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string()), 0, &mut history, magic_box), 8);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k5/6Pp/8/8 w - - 0 1".to_string()), 0, &mut history, magic_box), 4);
    assert_eq!(perft(&mut get_position(&"8/8/p7/1P6/K1k3pP/6P1/8/8 w - - 0 1".to_string()), 0, &mut history, magic_box), 5);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1Pk5/K5pP/6P1/8/8 w - - 0 1".to_string()), 0, &mut history, magic_box), 5);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P1k4/K5pP/6P1/8/8 w - - 0 1".to_string()), 0, &mut history, magic_box), 6);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K2k2pP/6P1/8/8 w - - 0 1".to_string()), 0, &mut history, magic_box), 6);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K5pP/2k3P1/8/8 w - - 0 1".to_string()), 0, &mut history, magic_box), 4);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K5pP/3k2P1/8/8 w - - 0 1".to_string()), 0, &mut history, magic_box), 6);
    assert_eq!(perft(&mut get_position(&"8/8/8/pP6/K1k3pP/6P1/8/8 w - a6 0 1".to_string()), 0, &mut history, magic_box), 5);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string()), 1, &mut history, magic_box), 41);

    assert_eq!(perft(&mut get_position(&"8/8/8/KP6/5pPk/8/4P3/8 b - g3 0 1".to_string()), 1, &mut history, magic_box), 46);
    assert_eq!(perft(&mut get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 0, &mut history, magic_box), 48);
    assert_eq!(perft(&mut get_position(&"8/2p5/8/KP6/5pPk/8/4P3/8 b - g3 0 1".to_string()), 1, &mut history, magic_box), 57);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP6/5pPk/8/4P3/8 b - g3 0 1".to_string()), 1, &mut history, magic_box), 64);
    assert_eq!(perft(&mut get_position(&"4k3/8/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1".to_string()), 1, &mut history, magic_box), 100);
    assert_eq!(perft(&mut get_position(&"5k2/5p1p/p3B1p1/P5P1/3K1P1P/8/8/8 b - - 0 1".to_string()), 1, &mut history, magic_box), 169);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1".to_string()), 1, &mut history, magic_box), 177);
    assert_eq!(perft(&mut get_position(&"8/8/3p4/KPp4r/1R3p1k/4P3/6P1/8 w - c6 0 1".to_string()), 1, &mut history, magic_box), 190);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 1, &mut history, magic_box), 191);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/R4p1k/8/4P1P1/8 b - - 0 1".to_string()), 1, &mut history, magic_box), 202);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/1P5r/KR3p1k/8/4P1P1/8 b - - 0 1".to_string()), 1, &mut history, magic_box), 224);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3 0 1".to_string()), 1, &mut history, magic_box), 226);
    assert_eq!(perft(&mut get_position(&"8/2p5/K2p4/1P5r/1R3p1k/8/4P1P1/8 b - - 0 1".to_string()), 1, &mut history, magic_box), 240);
    assert_eq!(perft(&mut get_position(&"8/3K4/2p5/p2b2r1/5k2/8/8/1q6 b - - 1 67".to_string()), 1, &mut history, magic_box), 279);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R2K3R b kq - 0 1".to_string()), 1, &mut history, magic_box), 300);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/1R2K2R b Kkq - 0 1".to_string()), 1, &mut history, magic_box), 377);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/3RK2R b Kkq - 0 1".to_string()), 1, &mut history, magic_box), 365);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b3P/P7/R3K2R b KQkq - 0 1".to_string()), 1, &mut history, magic_box), 339);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/Ppp1p3/3b4/7P/R3K2R b KQkq a3 0 1".to_string()), 1, &mut history, magic_box), 357);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p2P/3b4/P7/R3K2R b KQkq - 0 1".to_string()), 1, &mut history, magic_box), 358);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/P2b4/7P/R3K2R b KQkq - 0 1".to_string()), 1, &mut history, magic_box), 376);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2R1K2R b Kkq - 0 1".to_string()), 1, &mut history, magic_box), 380);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2KR3R b kq - 0 1".to_string()), 1, &mut history, magic_box), 385);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3KR2 b Qkq - 0 1".to_string()), 1, &mut history, magic_box), 395);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/8/1Bp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 1, &mut history, magic_box), 395);
    assert_eq!(perft(&mut get_position(&"r2Bk2r/p6p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 1, &mut history, magic_box), 403);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P4K1P/R6R b kq - 0 1".to_string()), 1, &mut history, magic_box), 437);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P2K3P/R6R b kq - 0 1".to_string()), 1, &mut history, magic_box), 437);
    assert_eq!(perft(&mut get_position(&"r3k2r/p1B4p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 1, &mut history, magic_box), 438);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K1R1 b Qkq - 0 1".to_string()), 1, &mut history, magic_box), 454);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/1B6/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 1, &mut history, magic_box), 470);
    assert_eq!(perft(&mut get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 1, &mut history, magic_box), 2039);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3 0 1".to_string()), 2, &mut history, magic_box), 3702);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 2, &mut history, magic_box), 2812);

    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string()), 2, &mut history, magic_box), 325);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string()), 3, &mut history, magic_box), 2002);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string()), 4, &mut history, magic_box), 16763);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string()), 5, &mut history, magic_box), 118853);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - 0 1".to_string()), 0, &mut history, magic_box), 5);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - 0 1".to_string()), 1, &mut history, magic_box), 39);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - 0 1".to_string()), 2, &mut history, magic_box), 237);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - 0 1".to_string()), 3, &mut history, magic_box), 2002);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - 0 1".to_string()), 4, &mut history, magic_box), 14062);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - 0 1".to_string()), 5, &mut history, magic_box), 120995);

    assert_eq!(perft(&mut get_position(&"5k2/5p1p/p3B1p1/P5P1/3K1P1P/8/8/8 b - - 0 1".to_string()), 3, &mut history, magic_box), 20541);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 3, &mut history, magic_box), 43238);
    assert_eq!(perft(&mut get_position(&"n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1".to_string()), 3, &mut history, magic_box), 182838);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/1R2K2R b Kkq - 0 1".to_string()), 3, &mut history, magic_box), 175927);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2R1K2R b Kkq - 0 1".to_string()), 3, &mut history, magic_box), 178248);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/3RK2R b Kkq - 0 1".to_string()), 3, &mut history, magic_box), 168357);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P4K1P/R6R b kq - 0 1".to_string()), 3, &mut history, magic_box), 221267);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P2K3P/R6R b kq - 0 1".to_string()), 3, &mut history, magic_box), 213344);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R2K3R b kq - 0 1".to_string()), 3, &mut history, magic_box), 120873);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2KR3R b kq - 0 1".to_string()), 3, &mut history, magic_box), 184127);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K1R1 b Qkq - 0 1".to_string()), 3, &mut history, magic_box), 240619);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3KR2 b Qkq - 0 1".to_string()), 3, &mut history, magic_box), 189825);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/Ppp1p3/3b4/7P/R3K2R b KQkq a3 0 1".to_string()), 3, &mut history, magic_box), 154828);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/P2b4/7P/R3K2R b KQkq - 0 1".to_string()), 3, &mut history, magic_box), 173400);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p2P/3b4/P7/R3K2R b KQkq - 0 1".to_string()), 3, &mut history, magic_box), 165129);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b3P/P7/R3K2R b KQkq - 0 1".to_string()), 3, &mut history, magic_box), 151137);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/1B6/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 3, &mut history, magic_box), 249845);
    assert_eq!(perft(&mut get_position(&"r3k2r/p1B4p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 3, &mut history, magic_box), 227059);
    assert_eq!(perft(&mut get_position(&"r2Bk2r/p6p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 3, &mut history, magic_box), 185525);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/8/1Bp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string()), 3, &mut history, magic_box), 186968);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 1, &mut history, magic_box), 341);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 2, &mut history, magic_box), 6666);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 3, &mut history, magic_box), 150072);

    assert_eq!(perft(&mut get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 2, &mut history, magic_box), 97862);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string()), 6, &mut history, magic_box), 986637);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - 0 1".to_string()), 6, &mut history, magic_box), 966152);
    assert_eq!(perft(&mut get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - - 0 1".to_string()), 7, &mut history, magic_box), 8103790);
    assert_eq!(perft(&mut get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string()), 4, &mut history, magic_box), 674624);
    assert_eq!(perft(&mut get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 3, &mut history, magic_box), 4085603);
    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2KR3R b kq - 0 1".to_string()), 4, &mut history, magic_box), 4238116);
    assert_eq!(perft(&mut get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()), 4, &mut history, magic_box), 4865609);

    assert_eq!(perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 4, &mut history, magic_box), 3186478);
    assert_eq!(perft(&mut get_position(&"rnbqkb1r/ppppp1pp/7n/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3".to_string()), 4, &mut history, magic_box), 11139762);

}