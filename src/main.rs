mod bitboard;
mod masks;
mod movegen;
mod position;
#[cfg(test)]
mod tests;

use bitboard::Bitboard;
use movegen::{Move, MoveFlags, MoveGen};
use position::{Color, PieceType, Position};

fn main() {
    movegen::bishop_mask(0).visualize(); // a1 — corner
    movegen::bishop_mask(27).visualize(); // d4 — center
    movegen::bishop_mask(46).visualize(); // h8 — corner
}
