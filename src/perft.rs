use crate::make_move::make_move::{make_move, switch_side};
use crate::moves::moves::{is_check, moves};
use crate::types::types::Position;

pub fn perft(position: Position, depth: u8) -> u64 {

    let new_positions: Vec<Position> = moves(&position).into_iter().map(|m| {
        let mut new_position = position.clone();
        make_move(&mut new_position, m);
        new_position
    }).collect();

    let not_in_check_positions: Vec<Position> = new_positions.into_iter().filter(|p| {
        !is_check(&p, &switch_side(p.mover))
    }).collect();

    if depth == 0 {
        not_in_check_positions.len() as u64
    } else {
        let mut count = 0;
        not_in_check_positions.into_iter().for_each(|p| {
            count += perft(p, depth - 1);
        });

        count
    }
}