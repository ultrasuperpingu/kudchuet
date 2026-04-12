
use bitboard_proc_macro::{BitboardDebug, BitboardDisplay, bitboard};
//use bitboard::{Bitboard, bitboard_table};

#[bitboard(width=19, height=19)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Goban;
impl Goban {
	//const ODD : Self = Self::any(&self)
}