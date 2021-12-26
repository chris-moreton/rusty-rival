use rusty_rival::utils::utils::{from_square_mask, from_square_part, to_square_part};

#[test]
fn it_creates_a_move_with_the_from_part_only() {
    assert_eq!(from_square_mask(21), 0b00000000000101010000000000000000);
}

#[test]
fn it_gets_the_from_part_of_a_move() {
    assert_eq!(from_square_part(0b00000000000101010000000000111100), 21);
}

#[test]
fn it_gets_the_to_part_of_a_move() {
    assert_eq!(to_square_part(0b00000000000101010000000000111100), 60);
}