mod bot;
mod color;
mod board;
mod chess_move;
mod builder;
mod instance;
//mod logic;

use crate::game::Builder;

pub(crate) fn get() -> Box<dyn Builder> {
    builder::Builder::new()
}
