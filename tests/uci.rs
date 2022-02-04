use rusty_rival::uci::run_command;

#[test]
pub fn it_handles_dodgy_fens() {
    let mut fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
    let command = "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1";
    run_command(&mut fen, command);
}
