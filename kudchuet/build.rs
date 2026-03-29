// build.rs
use std::fs::File;
use std::io::Write;
use std::path::Path;

use bitboard_proc_macro::bitboard;
use bitboard::{Bitboard, IntegerStorage};
use num_traits::{PrimInt, ToPrimitive, Unsigned};



#[bitboard(width=8,height=8,col_major=false)]
#[derive(Default, Debug)]
pub struct Bitboard8x8;

#[bitboard(width=5,height=5,col_major=false)]
#[derive(Default, Debug)]
pub struct Bitboard5x5;

#[inline]
pub fn rook_mask(square: u8) -> Bitboard8x8 {
	Bitboard8x8::compute_row_mask(square as usize)&!Bitboard8x8::WEST_BORDER&!Bitboard8x8::EAST_BORDER|Bitboard8x8::compute_col_mask(square as usize)&!Bitboard8x8::NORTH_BORDER&!Bitboard8x8::SOUTH_BORDER
}
#[inline]
pub fn bishop_mask(square: u8) -> Bitboard8x8 {
	(Bitboard8x8::compute_diag_inc_mask(square as usize)|Bitboard8x8::compute_diag_dec_mask(square as usize))&!Bitboard8x8::BORDER
}

#[inline]
pub fn slide_mask(square: u8) -> Bitboard5x5 {
	Bitboard5x5::compute_diag_inc_mask(square as usize)|Bitboard5x5::compute_diag_dec_mask(square as usize)|Bitboard5x5::compute_col_mask(square as usize)|Bitboard5x5::compute_row_mask(square as usize)
}
impl Bitboard8x8
where
	<Self as Bitboard>::Storage: PrimInt + Unsigned,
{
	pub fn compute_attacks(offsets: &[(i8, i8)], square: u8, blockers: Self) -> Self
	{
		let rank = (square / Self::WIDTH) as i8;
		let file = (square % Self::HEIGHT) as i8;

		let blockers_u128 = blockers.storage().to_u128().unwrap();
		let mut attacks = 0u128;

		for &(dr, df) in offsets {
			let mut r = rank + dr;
			let mut f = file + df;

			while r >= 0 && r < Self::WIDTH as i8 && f >= 0 && f < Self::HEIGHT as i8 {
				let sq = (r * Self::WIDTH as i8 + f) as u128;
				attacks |= 1u128 << sq;

				if blockers_u128 & (1u128 << sq) != 0 {
					break;
				}

				r += dr;
				f += df;
			}
		}
		Self::from_storage(<<Self as Bitboard>::Storage as IntegerStorage>::from_u128(attacks))
	}
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
	generate_chess_pext_tables();
	generate_neutron_pext_tables();
}
fn generate_chess_pext_tables() {
	const ROOK_OFFSETS: &[(i8,i8)] = &[
		(1,0),(0,1),(-1,0),(0,-1)
	];
	const BISHOP_OFFSETS: &[(i8,i8)] = &[
		(1,1),(-1,1),(-1,-1),(1,-1)
	];

	let out_dir = std::env::var("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("generated_chess_pext_tables.rs");
	let mut f = File::create(&dest_path).unwrap();


	//let rook_masks = Bitboard8x8::generate_sliding_attacks_table(ROOK_OFFSETS);
	//let bishop_masks = Bitboard8x8::generate_sliding_attacks_table(BISHOP_OFFSETS);
	let mut rook_masks = [Bitboard8x8::default();Bitboard8x8::NB_SQUARES];
	for i in 0..Bitboard8x8::NB_SQUARES {
		rook_masks[i]=rook_mask(i as u8);
	}
	let mut bishop_masks = [Bitboard8x8::default();Bitboard8x8::NB_SQUARES];
	for i in 0..Bitboard8x8::NB_SQUARES {
		bishop_masks[i]=bishop_mask(i as u8);
	}
	
	let rook_pext = Bitboard8x8::generate_attack_tables_pext(rook_mask, |square, blockers| Bitboard8x8::compute_attacks(ROOK_OFFSETS, square, blockers));
	let bishop_pext = Bitboard8x8::generate_attack_tables_pext(bishop_mask, |square, blockers| Bitboard8x8::compute_attacks(BISHOP_OFFSETS, square, blockers));


	// Génération du code Rust
	writeln!(f, "pub static ROOK_MASKS: [Bitboard8x8; {}] = {:?};", Bitboard8x8::NB_SQUARES, rook_masks).unwrap();
	writeln!(f, "pub static BISHOP_MASKS: [Bitboard8x8; {}] = {:?};", Bitboard8x8::NB_SQUARES, bishop_masks).unwrap();
	for (sq, table) in rook_pext.iter().enumerate() {
		writeln!(
			f,
			"pub static ROOK_MOVES_SQ{}: [Bitboard8x8; {}] = {:?};",
			sq,
			table.len(),
			table
		).unwrap();
	}
	for (sq, table) in bishop_pext.iter().enumerate() {
		writeln!(
			f,
			"pub static BISHOP_MOVES_SQ{}: [Bitboard8x8; {}] = {:?};",
			sq,
			table.len(),
			table
		).unwrap();
	}
	// Génération d’un array global pour toutes les cases
	writeln!(f, "pub static ROOK_MOVES: [&'static [Bitboard8x8]; {}] = [", Bitboard8x8::NB_SQUARES).unwrap();
	for sq in 0..Bitboard8x8::NB_SQUARES {
		writeln!(f, "&ROOK_MOVES_SQ{},", sq).unwrap();
	}
	writeln!(f, "];").unwrap();

	writeln!(f, "pub static BISHOP_MOVES: [&'static [Bitboard8x8]; {}] = [", Bitboard8x8::NB_SQUARES).unwrap();
	for sq in 0..Bitboard8x8::NB_SQUARES {
		writeln!(f, "&BISHOP_MOVES_SQ{},", sq).unwrap();
	}
	writeln!(f, "];").unwrap();
	
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