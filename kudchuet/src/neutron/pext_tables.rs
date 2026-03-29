//use magic_bitboard_proc_macro::generate_pext_tables;
use crate::common::bitboards::Bitboard5x5;

//generate_pext_tables!();


include!(concat!(env!("OUT_DIR"), "/generated_neutron_pext_tables.rs"));