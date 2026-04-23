use bitboard_proc_macro::{BitboardDebug, BitboardDisplay, bitboard};
use bitboard::bitboard_table;


#[bitboard(width=6,height=5)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Bitboard6x5;

bitboard_table!(NEIGHBORS_ORTHO, neighbors_ortho, neighbors_ortho_mask, Bitboard6x5, Bitboard6x5::generate_neighbors_ortho_table());
