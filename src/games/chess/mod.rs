mod bot;
mod color;
mod board;
mod chess_move;
mod builder;
mod instance;

use crate::game::Builder;

pub(crate) fn get() -> Box<dyn Builder> {
    builder::Builder::new()
}
