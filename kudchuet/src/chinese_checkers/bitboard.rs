
use std::fmt;

use bitboard_proc_macro::BitboardDebug;
use bitboard_proc_macro::bitboard;

use crate::chinese_checkers::ChineseCheckersPlayer;

const PLAYABLE: [[bool; 13]; 17] = [
	[false,false,false,false,false,false,true ,false,false ,false,false,false,false],
	[false,false,false,false,false,false,true ,true ,false ,false,false,false,false],
	[false,false,false,false,false,true ,true ,true ,false,false,false,false,false],
	[false,false,false,false,false,true ,true ,true ,true ,false,false,false,false],
	[true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ],
	[false,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ],
	[false,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,false],
	[false,false,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,false],
	[false,false,true ,true ,true ,true ,true ,true ,true ,true ,true ,false,false],
	[false,false,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,false],
	[false,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,false],
	[false,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ],
	[true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ,true ],
	[false,false,false,false,false,true ,true ,true ,true ,false,false,false,false],
	[false,false,false,false,false,true ,true,true ,false,false,false,false,false],
	[false,false,false,false,false,false,true,true,false ,false,false,false,false],
	[false,false,false,false,false,false,true,false,false ,false,false,false,false],
];

#[bitboard(width = 13, height = 17)]
#[derive(Default)]
#[derive(BitboardDebug, Copy)]
pub struct ChineseCheckerBoard;

impl ChineseCheckerBoard {
	pub const PLAYABLE_MASK: Self = {
		let mut m = Self::EMPTY;
		let mut index=0;
		while index < Self::NB_SQUARES {
			let (x,y) = Self::coords_from_index(index);
			if PLAYABLE[y as usize][x as usize] {
				m.set_at_index(index);
			}
			index+=1;
		}
		m
	};
	/*pub const PLAYABLE_MASK_U128: u128 = Self::PLAYABLE_MASK.to_u128();

	pub const fn to_u128(&self) -> u128 {
		let mut result = 0u128;
		let mut bit_index = 0;
		let mut y=0;
		while y < 17 {
			let mut x=0;
			while x < 13 {
				if PLAYABLE[y][x] {
					if self.get(x as u8, y as u8) {
						result |= 1 << bit_index;
					}
					bit_index += 1;
				}
				x+=1;
			}
			y+=1;
		}
		result
	}

	pub const fn from_u128(mut value: u128) -> Self {
		let mut board = ChineseCheckerBoard::EMPTY;
		let mut y=0;
		while y < 17 {
			let mut x=0;
			while x < 13 {
				if PLAYABLE[y][x] {
					if (value & 1) != 0 {
						board.set(x as u8, y as u8);
					}
					value >>= 1;
				}
				x+=1;
			}
			y+=1;
		}
		board
	}*/
	pub fn is_playable(x: u8, y: u8) -> bool {
		if x >= 13 || y >= 17 {
			return false;
		}
		PLAYABLE[y as usize][x as usize]
	}
	pub fn is_playable_index(index: u8) -> bool {
		let (x,y) = Self::coords_from_index(index as usize);
		Self::is_playable(x,y)
	}
	pub const FINAL_BLUE:(u8, u8) = (6, 0);
	pub const fn initial_red() -> ChineseCheckerBoard {
		let mut red=ChineseCheckerBoard::EMPTY;
		red.set(6, 0);
		red.set(6, 1);
		red.set(7, 1);
		red.set(5, 2);
		red.set(6, 2);
		red.set(7, 2);
		red.set(5, 3);
		red.set(6, 3);
		red.set(7, 3);
		red.set(8, 3);
		red
	}
	pub const FINAL_RED:(u8, u8) = (6, 16);
	pub const fn initial_blue() -> ChineseCheckerBoard {
		let mut blue=ChineseCheckerBoard::EMPTY;
		blue.set(6, 16);
		blue.set(6, 15);
		blue.set(7, 15);
		blue.set(5, 14);
		blue.set(6, 14);
		blue.set(7, 14);
		blue.set(5, 13);
		blue.set(6, 13);
		blue.set(7, 13);
		blue.set(8, 13);
		blue
	}
	pub const FINAL_WHITE:(u8, u8) = (0, 4);
	pub const fn initial_black() -> ChineseCheckerBoard {
		let mut black=ChineseCheckerBoard::EMPTY;
		black.set(0, 4);
		black.set(1, 4);
		black.set(2, 4);
		black.set(3, 4);
		black.set(1, 5);
		black.set(2, 5);
		black.set(3, 5);
		black.set(1, 6);
		black.set(2, 6);
		black.set(2, 7);
		black
	}
	pub const FINAL_BLACK:(u8, u8) = (12, 12);
	pub const fn initial_white() -> ChineseCheckerBoard {
		let mut white=ChineseCheckerBoard::EMPTY;
		white.set(11, 9);
		white.set(10, 10);
		white.set(11, 10);
		white.set(10, 11);
		white.set(11, 11);
		white.set(12, 11);
		white.set(9, 12);
		white.set(10, 12);
		white.set(11, 12);
		white.set(12, 12);
		white
	}
	pub const FINAL_GREEN:(u8, u8) = (12, 4);
	pub const fn initial_yellow() -> ChineseCheckerBoard {
		let mut yellow=ChineseCheckerBoard::EMPTY;
		yellow.set(12, 4);
		yellow.set(11, 4);
		yellow.set(10, 4);
		yellow.set(9, 4);
		yellow.set(12, 5);
		yellow.set(11, 5);
		yellow.set(10, 5);
		yellow.set(11, 6);
		yellow.set(10, 6);
		yellow.set(11, 7);
		yellow
	}
	pub const FINAL_YELLOW:(u8, u8) = (0, 12);
	pub const fn initial_green() -> ChineseCheckerBoard {
		let mut green=ChineseCheckerBoard::EMPTY;
		green.set(0, 12);
		green.set(1, 12);
		green.set(2, 12);
		green.set(3, 12);
		green.set(1, 11);
		green.set(2, 11);
		green.set(3, 11);
		green.set(1, 10);
		green.set(2, 10);
		green.set(2, 9);
		green
	}
	pub const fn final_square(p: ChineseCheckersPlayer) -> (u8,u8) {
		match p {
			ChineseCheckersPlayer::Red => Self::FINAL_RED,
			ChineseCheckersPlayer::Blue => Self::FINAL_BLUE,
			ChineseCheckersPlayer::Green => Self::FINAL_GREEN,
			ChineseCheckersPlayer::Yellow => Self::FINAL_YELLOW,
			ChineseCheckersPlayer::Black => Self::FINAL_BLACK,
			ChineseCheckersPlayer::White => Self::FINAL_WHITE,
		}
	}
	pub const fn target_board(p: ChineseCheckersPlayer) -> Self {
		match p {
			ChineseCheckersPlayer::Red => Self::initial_blue(),
			ChineseCheckersPlayer::Blue => Self::initial_red(),
			ChineseCheckersPlayer::Green => Self::initial_yellow(),
			ChineseCheckersPlayer::Yellow => Self::initial_green(),
			ChineseCheckersPlayer::Black => Self::initial_white(),
			ChineseCheckersPlayer::White => Self::initial_black(),
		}
	}
	pub fn unowned_zones_board(p: ChineseCheckersPlayer) -> Self {
		match p {
			ChineseCheckersPlayer::Red => Self::initial_yellow() | Self::initial_green() | Self::initial_black() | Self::initial_white(),
			ChineseCheckersPlayer::Blue => Self::initial_yellow() | Self::initial_green() | Self::initial_black() | Self::initial_white(),
			ChineseCheckersPlayer::Green => Self::initial_blue() | Self::initial_red() | Self::initial_black() | Self::initial_white(),
			ChineseCheckersPlayer::Yellow => Self::initial_blue() | Self::initial_red() | Self::initial_black() | Self::initial_white(),
			ChineseCheckersPlayer::Black => Self::initial_blue() | Self::initial_red() | Self::initial_yellow() | Self::initial_green(),
			ChineseCheckersPlayer::White => Self::initial_blue() | Self::initial_red() | Self::initial_yellow() | Self::initial_green(),
		}
	}
	pub fn initial_zone(p: ChineseCheckersPlayer) -> Self {
		match p {
			ChineseCheckersPlayer::Red => Self::initial_red(),
			ChineseCheckersPlayer::Blue => Self::initial_blue(),
			ChineseCheckersPlayer::Green => Self::initial_green(),
			ChineseCheckersPlayer::Yellow => Self::initial_yellow(),
			ChineseCheckersPlayer::Black => Self::initial_black(),
			ChineseCheckersPlayer::White => Self::initial_white(),
		}
	}
}
impl ChineseCheckerBoard {
	pub const fn neighbours(x: u8, y: u8) -> Self {
		let mut board = ChineseCheckerBoard::EMPTY;

		let deltas = if y.is_multiple_of(2) { Self::DELTAS_EVEN } else { Self::DELTAS_ODD };
		let mut i = 0;
		while i < 6 {
			let nx = x as i8 + deltas[i].0;
			let ny = y as i8 + deltas[i].1;

			if nx >= 0 && nx < 13 && ny >= 0 && ny < 17 {
				if PLAYABLE[ny as usize][nx as usize] {
					board.set(nx as u8, ny as u8);
				}
			}
			i += 1;
		}
		board
	}
	
	pub const fn generate_neighbors_table() -> [ChineseCheckerBoard;ChineseCheckerBoard::NB_SQUARES] {
		let mut table = [ChineseCheckerBoard::EMPTY; ChineseCheckerBoard::NB_SQUARES];
		let mut y = 0;
		while y < 17 {
			let mut x = 0;
			while x < 13 {
				if PLAYABLE[y][x] {
					let index = ChineseCheckerBoard::index_from_coords(x as u8, y as u8);
					table[index] = ChineseCheckerBoard::neighbours(x as u8, y as u8);
				}
				x += 1;
			}
			y += 1;
		}
		table
	}
	const DELTAS_EVEN: [(i8, i8); 6] = [(-1, 0), (1, 0), (0, -1), (1, -1), (0, 1), (1, 1)];
	const DELTAS_ODD: [(i8, i8); 6]  = [(-1, 0), (1, 0), (-1, -1), (0, -1), (-1, 1), (0, 1)];

	pub const NEIGHBOURS: [ChineseCheckerBoard;ChineseCheckerBoard::NB_SQUARES] = Self::generate_neighbors_table();
	pub const fn jumps(x: u8, y: u8) -> Self {
		let mut board = ChineseCheckerBoard::EMPTY;
		
		let deltas = if y.is_multiple_of(2) { Self::DELTAS_EVEN } else { Self::DELTAS_ODD };
		let mut i = 0;
		while i < 6 {
			let nx = x as i8 + deltas[i].0;
			let ny = y as i8 + deltas[i].1;

			if nx >= 0 && nx < 13 && ny >= 0 && ny < 17 {
				if PLAYABLE[ny as usize][nx as usize] {
					let delta_jump = if ny % 2 == 0 { Self::DELTAS_EVEN } else { Self::DELTAS_ODD }[i];

					let jump_x_i8 = nx + delta_jump.0;
					let jump_y_i8 = ny + delta_jump.1;

					if jump_x_i8 < 0 || jump_x_i8 >= 13 || jump_y_i8 < 0 || jump_y_i8 >= 17 {
						i+=1;
						continue;
					}

					let jump_x = jump_x_i8 as u8;
					let jump_y = jump_y_i8 as u8;

					if PLAYABLE[jump_y as usize][jump_x as usize]
					{
						board.set(jump_x, jump_y);
					}
				}
			}
			i += 1;
		}
		board
	}
	pub const fn generate_jump_table() -> [Self; Self::NB_SQUARES] {
		let mut table = [Self::EMPTY; Self::NB_SQUARES];
		let mut y = 0;
		while y < 17 {
			let mut x = 0;
			while x < 13 {
				if PLAYABLE[y][x] {
					let index = ChineseCheckerBoard::index_from_coords(x as u8, y as u8);
					table[index] = Self::jumps(x as u8, y as u8);
				}
				x += 1;
			}
			y += 1;
		}
		table
	}
	#[inline]
	pub fn compute_middle(from_x:u8, from_y:u8, jump_x:u8, jump_y:u8, jump_index:usize, from_index:usize) -> usize {
		let dx = jump_x as i8 - from_x as i8;
		let dy = jump_y as i8 - from_y as i8;

		if from_y.is_multiple_of(2) {
			if dy == 0 || dx == dy {
				(from_index + jump_index) / 2
			} else {
				(from_index + jump_index) / 2 + 1
			}
		} else {
			(from_index + jump_index) / 2
		}
	}
	pub const fn generate_middle_table() -> [[u16;ChineseCheckerBoard::NB_SQUARES];ChineseCheckerBoard::NB_SQUARES] {
		let mut table = [[0; ChineseCheckerBoard::NB_SQUARES]; ChineseCheckerBoard::NB_SQUARES];
		let mut y = 0;
		while y < 17 {
			let mut x = 0;
			while x < 13 {
				if PLAYABLE[y][x] {
					let deltas = if y % 2 == 0 { Self::DELTAS_EVEN } else { Self::DELTAS_ODD };
					let mut i = 0;
					while i < 6 {
						let nx = x as i8 + deltas[i].0;
						let ny = y as i8 + deltas[i].1;

						if nx >= 0 && nx < 13 && ny >= 0 && ny < 17 {
							if PLAYABLE[ny as usize][nx as usize] {
								let delta_jump = if ny % 2 == 0 { Self::DELTAS_EVEN } else { Self::DELTAS_ODD }[i];

								let jump_x_i8 = nx + delta_jump.0;
								let jump_y_i8 = ny + delta_jump.1;

								if jump_x_i8 < 0 || jump_x_i8 >= 13 || jump_y_i8 < 0 || jump_y_i8 >= 17 {
									i+=1;
									continue;
								}

								let jump_x = jump_x_i8 as u8;
								let jump_y = jump_y_i8 as u8;

								if PLAYABLE[jump_y as usize][jump_x as usize]
								{
									let from_index=ChineseCheckerBoard::index_from_coords(x as u8, y as u8);
									let jump_index=ChineseCheckerBoard::index_from_coords(jump_x, jump_y);
									let jumped_index=ChineseCheckerBoard::index_from_coords(nx as u8, ny as u8);
									table[from_index][jump_index]=jumped_index as u16;
									//println!("{}-{}={}",from_index, jump_index, jumped_index);
								}
							}
						}
						i += 1;
					}
				}
				x += 1;
			}
			y += 1;
		}
		table
	}

}

impl fmt::Display for ChineseCheckerBoard {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for y in 0..17 {

			if y % 2 == 0 {
				write!(f, " ")?;
			}

			for x in 0..13 {
				if !ChineseCheckerBoard::is_playable(x, y) {
					write!(f, "  ")?;
					continue;
				}

				if self.get(x, y) {
					write!(f, "● ")?; // ou "X "
				} else {
					write!(f, ". ")?;
				}
			}
			writeln!(f)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use bitboard::{BitIter, Bitboard};

	use super::*;

	#[test]
	fn test_neighbours() {
		// Coin supérieur du plateau (initial red position)
		let top = ChineseCheckerBoard::neighbours(6, 0);
		println!("{}", top);
		// Neighbours jouables selon le masque PLAYABLE
		assert!(top.get(6, 1));
		assert!(top.get(7, 1));
		assert_eq!(top.count(), 2);

		// Centre du plateau, ligne 8 (large zone jouable)
		let center = ChineseCheckerBoard::neighbours(6, 8);
		println!("{}", center);
		// Devrait avoir 6 voisins jouables (pour cette case centrale)
		for bit in center.iter_bits() {
			let (x, y) = ChineseCheckerBoard::coords_from_index(bit as usize);
			assert!(ChineseCheckerBoard::is_playable(x, y));
		}

		// Coin inférieur droit (initial white position)
		let bottom_right = ChineseCheckerBoard::neighbours(12, 11);
		for bit in bottom_right.iter_bits() {
			let (x, y) = ChineseCheckerBoard::coords_from_index(bit as usize);
			assert!(ChineseCheckerBoard::is_playable(x, y));
		}
		for bit in ChineseCheckerBoard::initial_yellow().iter_bits() {
			let (x, y) = ChineseCheckerBoard::coords_from_index(bit as usize);
			println!("{}", ChineseCheckerBoard::neighbours(x, y));
		}
		
		println!("Neighbours tests passed!");
	}
	
	#[test]
	pub fn test_middle_table() {
		let mut y = 0;
		while y < 17 {
			let mut x = 0;
			while x < 13 {
				if PLAYABLE[y][x] {
					let deltas = if y % 2 == 0 { ChineseCheckerBoard::DELTAS_EVEN } else { ChineseCheckerBoard::DELTAS_ODD };
					let mut i = 0;
					while i < 6 {
						let nx = x as i8 + deltas[i].0;
						let ny = y as i8 + deltas[i].1;

						if nx >= 0 && nx < 13 && ny >= 0 && ny < 17 {
							if PLAYABLE[ny as usize][nx as usize] {
								let delta_jump = if ny % 2 == 0 { ChineseCheckerBoard::DELTAS_EVEN } else { ChineseCheckerBoard::DELTAS_ODD }[i];

								let jump_x_i8 = nx as i8 + delta_jump.0;
								let jump_y_i8 = ny as i8 + delta_jump.1;

								if jump_x_i8 < 0 || jump_x_i8 >= 13 || jump_y_i8 < 0 || jump_y_i8 >= 17 {
									i+=1;
									continue;
								}

								let jump_x = jump_x_i8 as u8;
								let jump_y = jump_y_i8 as u8;

								if PLAYABLE[jump_y as usize][jump_x as usize]
								{
									let from_index=ChineseCheckerBoard::index_from_coords(x as u8, y as u8);
									let jump_index=ChineseCheckerBoard::index_from_coords(jump_x, jump_y);
									let jumped_index=ChineseCheckerBoard::index_from_coords(nx as u8, ny as u8);
									let (from_x, from_y) = ChineseCheckerBoard::coords_from_index(from_index);
									let (jump_x, jump_y) = ChineseCheckerBoard::coords_from_index(jump_index);

									let mid = ChineseCheckerBoard::compute_middle(from_x, from_y, jump_x, jump_y, jump_index, from_index);
									println!("{}-{}={} or {}",from_index, jump_index, jumped_index, mid);
									assert!(jumped_index == mid);
								}
							}
						}
						i += 1;
					}
				}
				x += 1;
			}
			y += 1;
		}
	}
	
}