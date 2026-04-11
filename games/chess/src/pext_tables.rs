//use magic_bitboard_proc_macro::generate_pext_tables;
use crate::bitboard::Bitboard8x8;

//generate_pext_tables!();


include!(concat!(env!("OUT_DIR"), "/generated_chess_pext_tables.rs"));