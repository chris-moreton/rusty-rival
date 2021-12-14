type Square = u64;
type Bitboard = u64;
type Move = u64;
type MoveList = ConsList<Move>;
type Path = ConsList<Move>;
type MagicFunc = fn(Square, Int) -> Bitboard;
