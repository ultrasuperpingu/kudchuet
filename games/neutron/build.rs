// build.rs
use std::fs::File;
use std::io::Write;
use std::path::Path;

use bitboard_proc_macro::bitboard;
use bitboard::{Bitboard, IntegerStorage};
use num_traits::{PrimInt, ToPrimitive, Unsigned};

#[bitboard(width=5,height=5)]
#[derive(Default, Debug)]
pub struct Bitboard5x5;

#[inline]
pub fn slide_mask(square: u8) -> Bitboard5x5 {
	Bitboard5x5::compute_diag_inc_mask(square as usize)|Bitboard5x5::compute_diag_dec_mask(square as usize)|Bitboard5x5::compute_col_mask(square as usize)|Bitboard5x5::compute_row_mask(square as usize)
}


impl Bitboard5x5
where
	<Self as Bitboard>::Storage: PrimInt + Unsigned,
{
	pub fn compute_slides(offsets: &[(i8, i8)], square: u8, blockers: Self) -> Self
	{
		let sq_x = (square / Self::WIDTH) as i8;
		let sq_y = (square % Self::HEIGHT) as i8;

		let mut attacks = Bitboard5x5::empty();

		for &(dx, dy) in offsets {
			let mut x = sq_x + dx;
			let mut y = sq_y + dy;
			let mut last=None;
			while x >= 0 && x < Self::WIDTH as i8 && y >= 0 && y < Self::HEIGHT as i8 {
				let sq = (x * Self::WIDTH as i8 + y) as u8;
				let sq_mask = Bitboard5x5::from_index(sq as usize);
				// reach a blocker: last if exists
				if !(blockers & sq_mask).is_empty() {
					if let Some(last) = last {
						attacks |= last;
					}
					break;
				}
				
				last = Some(sq_mask);
				x += dx;
				y += dy;
				// reach border: as this square as no blocker, its a move position
				if x < 0 || x >= Self::WIDTH as i8 || y < 0 || y >= Self::HEIGHT as i8 {
					attacks |= sq_mask;
					break;
				}
			}
		}
		attacks
	}
}

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	generate_neutron_pext_tables();
}


fn generate_neutron_pext_tables() {
	const SLIDE_OFFSETS: &[(i8,i8)] = &[
		(1,1),(-1,1),(-1,-1),(1,-1),
		(1,0),(0,1),(-1,0),(0,-1)
	];

	let out_dir = std::env::var("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("generated_neutron_pext_tables.rs");
	let mut f = File::create(&dest_path).unwrap();


	let mut slide_masks = [Bitboard5x5::default();Bitboard5x5::NB_SQUARES];
	for i in 0..Bitboard5x5::NB_SQUARES {
		slide_masks[i]=slide_mask(i as u8);
	}
	
	let slide_pext = Bitboard5x5::generate_attack_tables_pext(slide_mask, |square, blockers| Bitboard5x5::compute_slides(SLIDE_OFFSETS, square, blockers));


	// Génération du code Rust
	writeln!(f, "pub static SLIDE_MASKS: [Bitboard5x5; {}] = {:?};", Bitboard5x5::NB_SQUARES, slide_masks).unwrap();
	for (sq, table) in slide_pext.iter().enumerate() {
		writeln!(
			f,
			"pub static SLIDE_MOVES_SQ{}: [Bitboard5x5; {}] = {:?};",
			sq,
			table.len(),
			table
		).unwrap();
	}
	writeln!(f, "pub static SLIDE_MOVES: [&'static [Bitboard5x5]; {}] = [", Bitboard5x5::NB_SQUARES).unwrap();
	for sq in 0..Bitboard5x5::NB_SQUARES {
		writeln!(f, "&SLIDE_MOVES_SQ{},", sq).unwrap();
	}
	writeln!(f, "];").unwrap();
	
}