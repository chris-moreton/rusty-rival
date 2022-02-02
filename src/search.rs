use std::mem;
use std::process::exit;
use std::sync::mpsc::Sender;
use std::time::Instant;
use crate::make_move::make_move;
use crate::moves::moves;
use crate::types::Position;

pub fn search_zero(position: &Position, depth: u8, tx: Sender<String>) {
    search(position, depth, &tx);
    let mut i = 0;
    loop {
        i += 1;
        let val = String::from(i.to_string());
        tx.send(val).unwrap();
    }

}

pub fn search(position: &Position, depth: u8, tx: &Sender<String>) {
    for m in moves(position) {
        unsafe {
            let mut new_position = mem::MaybeUninit::<Position>::uninit();
            make_move(position, m, &mut *new_position.as_mut_ptr());
        }
    }
}