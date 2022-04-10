use rusty_rival::engine_constants::{
    BISHOP_VALUE_AVERAGE, KNIGHT_VALUE_AVERAGE, PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE, ROOK_VALUE_AVERAGE,
};
use rusty_rival::fen::get_position;
use rusty_rival::see::static_exchange_evaluation;
use rusty_rival::types::Score;
use rusty_rival::utils::hydrate_move_from_algebraic_move;

fn assert_see_score(fen: &str, ms: &str, score: Score) {
    let position = get_position(fen);
    let m = hydrate_move_from_algebraic_move(&position, ms.to_string());
    assert_eq!(static_exchange_evaluation(&position, m), score);
}

#[test]
fn it_gets_the_see_score() {
    assert_see_score(
        "4k3/p1ppr1b1/bnr1N3/4N1n1/1p2P1p1/7p/PPPBBPPP/R3K2R b KQ - 0 1",
        "d7e6",
        KNIGHT_VALUE_AVERAGE,
    );
    assert_see_score(
        "4k3/p1ppr1b1/bnr1p3/4N1n1/1p1NP1p1/7p/PPPBBPPP/R3K2R w KQ - 0 1",
        "d4e6",
        PAWN_VALUE_AVERAGE - KNIGHT_VALUE_AVERAGE,
    );
    assert_see_score(
        "4k3/p1pprpb1/bnr1p3/3QN1n1/1p1NP1p1/7p/PPPBBPPP/R3K2R w KQ - 0 1",
        "d5e6",
        PAWN_VALUE_AVERAGE - QUEEN_VALUE_AVERAGE,
    );

    // // winning rook capture - black rook can't recapture safely
    assert_see_score("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", "b4f4", PAWN_VALUE_AVERAGE);
    assert_see_score("4k2r/p1ppqpb1/bnr1p3/3PN1n1/1p2P1p1/2N2Q1p/PPPBBPPP/R3K2R w KQk - 0 1", "d5e6", 0);
    assert_see_score("5k2/5p1p/p3B1p1/P5P1/3K1P1P/8/8/8 b - - 0 1", "f7e6", BISHOP_VALUE_AVERAGE);

    // pawn promotes, king has to take queen
    assert_see_score(
        "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
        "g2f1",
        KNIGHT_VALUE_AVERAGE - PAWN_VALUE_AVERAGE,
    );

    // leaves king in check
    assert_see_score("8/7p/p5pb/4k3/P1pPn3/8/P5PP/1rB2RK1 b - d3 0 28", "h6c1", 0);
    assert_see_score("rnbqkb1r/ppppp1pp/7n/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3", "e5f6", 0);
    assert_see_score(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "e5d7",
        PAWN_VALUE_AVERAGE - KNIGHT_VALUE_AVERAGE,
    );
    assert_see_score(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "f3f6",
        KNIGHT_VALUE_AVERAGE - QUEEN_VALUE_AVERAGE,
    );
    assert_see_score("2rr3k/pp3pp1/1nnqbN1p/3pN3/1PpP4/2P3Q1/P1B4P/R4RK1 b - b3 0 1", "c4b3", 0);
    assert_see_score(
        "2rr1q1k/pp3pp1/1nn1b2p/3pN2N/2pP4/2P3Q1/PPB4P/R4RK1 w - - 2 2",
        "g3g7",
        PAWN_VALUE_AVERAGE - KNIGHT_VALUE_AVERAGE,
    );

    // can't recapture knight, because king in double check
    assert_see_score(
        "2rr1q2/pp3ppk/1nn1b1Np/3p3N/2pP4/2P3Q1/PPB4P/R4RK1 w - - 4 3",
        "g6f8",
        QUEEN_VALUE_AVERAGE,
    );
    assert_see_score(
        "3r3r/ppk2ppp/3q4/2b5/2P2Bn1/3R1Q2/P4PPP/5RK1 w - - 0 5",
        "d3d6",
        QUEEN_VALUE_AVERAGE - ROOK_VALUE_AVERAGE,
    );
}
