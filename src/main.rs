mod bitboard;
mod masks;
mod movegen;
mod position;

use bitboard::Bitboard;
use movegen::{Move, MoveFlags, MoveGen};
use position::{Color, PieceType, Position};

fn main() {
    let pos = Position::start();

    let occ = pos.occupancy();
    occ.visualize();

    let white_occ = pos.occupancy_for(Color::White);
    white_occ.visualize();

    let knights = pos.get_piece_bitboard(Color::White, PieceType::Knight);
    knights.visualize();

    let mut bb = Bitboard::new();
    bb.set_bit(10);
    bb.clear_bit(10);
    let _ = bb.is_empty();
    let _ = bb.pop_lsb();

    let knight_atk = MoveGen::knight_attacks(1);
    knight_atk.visualize();

    let pawn_atk = MoveGen::pawn_attacks(8, Color::White);
    pawn_atk.visualize();

    let mv = Move::new(1, 18, MoveFlags::QUIET);
    println!("from={} to={} flags={}", mv.from(), mv.to(), mv.flags());
}
