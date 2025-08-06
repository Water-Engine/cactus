pub mod board;
pub mod piece;

pub const STARTING_COLOR: Color = Color::White;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}
