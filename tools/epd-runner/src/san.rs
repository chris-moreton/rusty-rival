//! SAN resolution on top of the engine crate's move generator: a SAN string
//! is matched against the legal moves of the position, so disambiguation,
//! captures, promotions and both castling spellings come for free.

use rusty_rival::fen::algebraic_move_from_move;
use rusty_rival::make_move::{make_move_in_place, unmake_move};
use rusty_rival::move_constants::{
    PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_ROOK,
};
use rusty_rival::moves::{generate_moves, is_check};
use rusty_rival::types::{Move, Position};
use rusty_rival::utils::is_capture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastleSide {
    King,
    Queen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveFacts {
    pub piece: char,
    pub from: String,
    pub to: String,
    pub capture: bool,
    pub promotion: Option<char>,
    pub castle: Option<CastleSide>,
}

/// Legal moves of the position (the generator is pseudo-legal).
pub fn legal_moves(position: &Position) -> Vec<Move> {
    let mut scratch = *position;
    let mover = scratch.mover;
    let mut out = Vec::new();
    for m in generate_moves(position) {
        let unmake = make_move_in_place(&mut scratch, m);
        let legal = !is_check(&scratch, mover);
        unmake_move(&mut scratch, m, &unmake);
        if legal {
            out.push(m);
        }
    }
    out
}

pub fn move_to_uci(m: Move) -> String {
    algebraic_move_from_move(m)
}

pub fn facts(position: &Position, m: Move) -> MoveFacts {
    let uci = algebraic_move_from_move(m);
    let from = uci[0..2].to_string();
    let to = uci[2..4].to_string();
    let promotion = uci.chars().nth(4).map(|c| c.to_ascii_uppercase());
    let piece = match m & PIECE_MASK_FULL {
        PIECE_MASK_PAWN => 'P',
        PIECE_MASK_KNIGHT => 'N',
        PIECE_MASK_BISHOP => 'B',
        PIECE_MASK_ROOK => 'R',
        PIECE_MASK_QUEEN => 'Q',
        PIECE_MASK_KING => 'K',
        _ => '?',
    };
    let castle = if piece == 'K' && from.starts_with('e') {
        match to.chars().next() {
            Some('g') => Some(CastleSide::King),
            Some('c') => Some(CastleSide::Queen),
            _ => None,
        }
    } else {
        None
    };
    MoveFacts {
        piece,
        from,
        to,
        capture: is_capture(position, m),
        promotion,
        castle,
    }
}

#[derive(Debug)]
struct SanSpec {
    castle: Option<CastleSide>,
    piece: char,
    dest: String,
    from_file: Option<char>,
    from_rank: Option<char>,
    capture: bool,
    promotion: Option<char>,
}

fn valid_square(s: &str) -> bool {
    let mut chars = s.chars();
    matches!((chars.next(), chars.next(), chars.next()), (Some('a'..='h'), Some('1'..='8'), None))
}

fn parse_san(san: &str) -> Result<SanSpec, String> {
    let trimmed = san.trim().trim_end_matches(['+', '#', '!', '?']);
    let castle_form = trimmed.replace('0', "O").to_ascii_uppercase();
    let no_move = |castle| SanSpec {
        castle: Some(castle),
        piece: 'K',
        dest: String::new(),
        from_file: None,
        from_rank: None,
        capture: false,
        promotion: None,
    };
    if castle_form == "O-O-O" {
        return Ok(no_move(CastleSide::Queen));
    }
    if castle_form == "O-O" {
        return Ok(no_move(CastleSide::King));
    }
    let mut chars: Vec<char> = trimmed.chars().collect();
    if chars.is_empty() {
        return Err("empty SAN".to_string());
    }
    let piece = if "KQRBN".contains(chars[0]) { chars.remove(0) } else { 'P' };
    let mut promotion = None;
    if piece == 'P' && chars.len() >= 3 {
        let last = chars[chars.len() - 1];
        if "QRBNqrbn".contains(last) {
            promotion = Some(last.to_ascii_uppercase());
            chars.pop();
            if chars.last() == Some(&'=') {
                chars.pop();
            }
        }
    }
    if chars.len() < 2 {
        return Err(format!("no destination square in '{}'", san));
    }
    let dest: String = chars[chars.len() - 2..].iter().collect();
    if !valid_square(&dest) {
        return Err(format!("'{}' is not a square in '{}'", dest, san));
    }
    let mut capture = false;
    let mut from_file = None;
    let mut from_rank = None;
    for c in &chars[..chars.len() - 2] {
        match c {
            'x' | ':' => capture = true,
            'a'..='h' if from_file.is_none() => from_file = Some(*c),
            '1'..='8' if from_rank.is_none() => from_rank = Some(*c),
            '-' => {}
            _ => return Err(format!("unexpected '{}' in '{}'", c, san)),
        }
    }
    Ok(SanSpec {
        castle: None,
        piece,
        dest,
        from_file,
        from_rank,
        capture,
        promotion,
    })
}

/// The one legal move the SAN string names, or an error when it names none
/// or several. An explicit `x` requires a capture; a missing `x` on a
/// capture is tolerated, since some suites omit it.
pub fn resolve_san(position: &Position, san: &str) -> Result<Move, String> {
    let spec = parse_san(san)?;
    let mut lenient = Vec::new();
    let mut strict = Vec::new();
    for m in legal_moves(position) {
        let f = facts(position, m);
        let matches = match spec.castle {
            Some(side) => f.castle == Some(side),
            None => {
                f.piece == spec.piece
                    && f.to == spec.dest
                    && spec.from_file.is_none_or(|c| f.from.starts_with(c))
                    && spec.from_rank.is_none_or(|c| f.from.ends_with(c))
                    && f.promotion == spec.promotion
            }
        };
        if matches {
            lenient.push(m);
            if f.capture == spec.capture {
                strict.push(m);
            }
        }
    }
    let candidates = if spec.capture { strict } else { lenient };
    match candidates.len() {
        1 => Ok(candidates[0]),
        0 => Err(format!("'{}' matches no legal move", san)),
        _ => Err(format!("'{}' is ambiguous", san)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_rival::fen::get_position;

    fn uci(fen: &str, san: &str) -> String {
        move_to_uci(resolve_san(&get_position(fen), san).unwrap())
    }

    #[test]
    fn resolves_pawn_moves_captures_and_promotions() {
        assert_eq!(uci("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", "e4"), "e2e4");
        assert_eq!(uci("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2", "exd5"), "e4d5");
        assert_eq!(uci("8/1P6/8/8/8/8/8/k6K w - - 0 1", "b8=Q+"), "b7b8q");
        assert_eq!(uci("8/1P6/8/8/8/8/8/k6K w - - 0 1", "b8N"), "b7b8n");
        assert_eq!(uci("rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 2", "dxe3"), "d4e3");
    }

    #[test]
    fn resolves_piece_moves_with_disambiguation() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        assert_eq!(uci(fen, "Rad1"), "a1d1");
        assert_eq!(uci(fen, "Rhf1"), "h1f1");
        assert_eq!(uci(fen, "Kf1"), "e1f1");
        let fen = "k7/8/8/8/8/8/8/K2N1N2 w - - 0 1";
        assert_eq!(uci(fen, "Nde3"), "d1e3");
        assert_eq!(uci(fen, "Nfe3"), "f1e3");
        assert!(resolve_san(&get_position(fen), "Ne3").is_err(), "ambiguous knight move must fail");
    }

    #[test]
    fn resolves_both_castling_spellings() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        assert_eq!(uci(fen, "O-O"), "e1g1");
        assert_eq!(uci(fen, "0-0-0"), "e1c1");
        assert_eq!(uci(fen, "O-O+"), "e1g1");
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1";
        assert_eq!(uci(fen, "O-O-O"), "e8c8");
    }

    #[test]
    fn suffixes_are_ignored_and_illegal_moves_fail() {
        let fen = "1k1r4/pp1b1R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 b - - 0 1";
        assert_eq!(uci(fen, "Qd1+"), "d6d1");
        assert_eq!(uci(fen, "Qd1#!"), "d6d1");
        assert!(resolve_san(&get_position(fen), "Qa1").is_err());
    }

    #[test]
    fn explicit_capture_of_an_empty_square_and_duplicate_origins_fail() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        assert!(
            resolve_san(&get_position(fen), "Kxe2").is_err(),
            "x with an empty target must not resolve"
        );
        assert!(resolve_san(&get_position(fen), "Raad1").is_err(), "duplicate origin file must fail");
        assert!(resolve_san(&get_position(fen), "R11d1").is_err(), "duplicate origin rank must fail");
        // A missing x on a real capture is tolerated.
        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
        assert_eq!(uci(fen, "ed5"), "e4d5");
    }
}
