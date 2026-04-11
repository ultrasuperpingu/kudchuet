use bitboard_proc_macro::{BitboardDebug, BitboardDisplay, bitboard};
use bitboard::{Bitboard, bitboard_table};

use crate::rules::CaptureList;



#[bitboard(width=5,height=5)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Bitboard5x5;
bitboard_table!(NEIGHBORS_ORTHO, neighbors_ortho, neighbors_ortho_mask, Bitboard5x5, Bitboard5x5::generate_neighbors_ortho_table());
bitboard_table!(NEIGHBORS_8, neighbors_8, neighbors_8_mask, Bitboard5x5, Bitboard5x5::generate_neighbors_8_table());

impl Bitboard5x5 {

	pub const fn generate_tiger_capture_table() -> [CaptureList; Self::NB_SQUARES] {
		let mut table = [CaptureList { moves: [(0,0); 8], len: 0 }; Self::NB_SQUARES];

		let mut from = 0;
		while from < Self::NB_SQUARES {

			let mut list = CaptureList { moves: [(0,0); 8], len: 0 };

			let neighbors_from = Self::NEIGHBORS_BAGH_CHAL[from].storage();
			let mut mid = 0;

			while mid < Self::NB_SQUARES {
				if (neighbors_from & (1 << mid)) != 0 {

					let neighbors_mid = Self::NEIGHBORS_BAGH_CHAL[mid].storage();
					let mut to = 0;

					while to < Self::NB_SQUARES {
						if (neighbors_mid & (1 << to)) != 0 {

							if to != from {

								let d1 = mid as i32 - from as i32;
								let d2 = to as i32 - mid as i32;

								if d1 == d2 {
									list.moves[list.len as usize] = (mid as u8, to as u8);
									list.len += 1;
								}
							}
						}
						to += 1;
					}
				}
				mid += 1;
			}

			table[from] = list;
			from += 1;
		}

		table
	}
	pub const fn neighbors_bagh_chal(square: usize) -> Bitboard5x5 {
		if square&1 == 1 {
			Bitboard5x5::NEIGHBORS_ORTHO[square]
		} else {
			Bitboard5x5::NEIGHBORS_8[square]
		}
	}
	pub const fn generate_neighbors_bagh_chal_table() -> [Bitboard5x5;Bitboard5x5::NB_SQUARES] {
		let mut res=[Bitboard5x5::empty();Bitboard5x5::NB_SQUARES];
		let mut sq=0;
		while sq < Self::NB_SQUARES {
			res[sq] = Self::neighbors_bagh_chal(sq);
			sq +=1;
		}
		res
	}
	pub const NEIGHBORS_BAGH_CHAL: [Bitboard5x5;Bitboard5x5::NB_SQUARES] = Self::generate_neighbors_bagh_chal_table();
	pub const TIGER_CAPTURES: [CaptureList;Bitboard5x5::NB_SQUARES] = Self::generate_tiger_capture_table();
}