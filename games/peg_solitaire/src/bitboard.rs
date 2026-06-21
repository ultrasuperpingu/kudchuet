use bitboard_proc_macro::{BitboardDebug, BitboardDisplay, bitboard};
#[bitboard(width = 7, height = 7)]
#[derive(BitboardDebug, BitboardDisplay)]
pub struct SolitaireBoard;
impl SolitaireBoard {
	pub const NEIGHBORS: [Self; Self::NB_SQUARES] = Self::generate_neighbors_table();
	pub const JUMPS: [Self; Self::NB_SQUARES] = Self::generate_jump_table();
	pub const MASK: Self = Self(0b0011100_0011100_1111111_1111111_1111111_0011100_0011100);
	pub const fn initial_state() -> Self {
		Self(0b0011100_0011100_1111111_1110111_1111111_0011100_0011100)
	}
	pub const fn generate_neighbors_table() -> [Self; Self::NB_SQUARES] {
		let mut table = Self::generate_neighbors_ortho_table();
		let mut i = 0;
		while i < Self::NB_SQUARES {
			table[i].and_assign_const(&Self::MASK);
			i += 1;
		}
		table
	}
	pub const fn jumps(x: u8, y: u8) -> Self {
		let mut board = SolitaireBoard::EMPTY;
		let offsets = [(0, 2), (0, -2), (2, 0), (-2, 0)];
		let mut i = 0;
		while i < offsets.len() {
			let jx = x as i16 + offsets[i].0;
			let jy = y as i16 + offsets[i].1;
			if jx >= 0 && jx < Self::WIDTH as i16 && jy >= 0 && jy < Self::HEIGHT as i16 {
				let index = Self::index_from_coords(jx as u8, jy as u8);
				board.or_assign_const(&Self::from_index(index).and_const(&Self::MASK));
			}
			i += 1;
		}

		board
	}
	pub const fn generate_jump_table() -> [Self; Self::NB_SQUARES] {
		let mut table = [Self::EMPTY; Self::NB_SQUARES];
		let mut i = 0;
		while i < Self::NB_SQUARES {
			if Self::MASK.get_at_index(i) {
				let (x, y) = Self::coords_from_index(i);
				table[i] = Self::jumps(x, y);
			}
			i += 1;
		}
		table
	}
	/*pub const fn generate_middle_table() -> [[u16; Self::NB_SQUARES]; Self::NB_SQUARES] {
		let mut table = [[0; Self::NB_SQUARES]; Self::NB_SQUARES];

		let mut i = 0;
		while i < Self::NB_SQUARES {
			if Self::MASK.get_at_index(i) {
				let mut j = 0;
				while j < Self::NB_SQUARES {
					if Self::MASK.get_at_index(j) {
						table[i][j] = (i + j) as u16 / 2;
					}
					j += 1;
				}
			}
			i += 1;
		}

		table
	}*/
}
