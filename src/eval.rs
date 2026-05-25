use crate::position::Position;

pub const PIECE_VALUES: [i32; 5] = [100, 320, 330, 500, 900];

pub fn evaluate_for_white(position: &Position) -> i32 {
    let mut white_material = 0i32;
    let mut black_material = 0i32;
    for i in 0..5 {
        white_material += position.pieces[i].0.count_ones() as i32 * PIECE_VALUES[i];
    }
    for i in 0..5 {
        black_material += position.pieces[i + 6].0.count_ones() as i32 * PIECE_VALUES[i];
    }
    white_material - black_material
}
