use bitboard_proc_macro::{BitboardDebug, BitboardDisplay, bitboard};
use bitboard::{Bitboard, bitboard_table};

#[bitboard(width=5,height=5)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Bitboard5x5;
bitboard_table!(NEIGHBORS_ORTHO, neighbors_ortho, neighbors_ortho_mask, Bitboard5x5, Bitboard5x5::generate_neighbors_ortho_table());
bitboard_table!(NEIGHBORS_8, neighbors_8, neighbors_8_mask, Bitboard5x5, Bitboard5x5::generate_neighbors_8_table());

