
use std::fmt::Display;
use std::hash::Hash;

use bitboard_proc_macro::{BitboardDebug, bitboard};
use bitboard::Bitboard;


#[bitboard(width=5, height=10)]
#[derive(Hash, BitboardDebug, Default)]
pub struct Bitboard5x10Checkers10;
impl Bitboard5x10Checkers10 {
	pub const fn black_pawn_advance_mask(sq: usize) -> Self {
		let (x, y) = Self::coords_from_index(sq);
		if y >= 9 { return Self(0); }

		let mut mask = 0;
		let is_even = y % 2 == 0;

		if is_even {
			mask |= 1 << (sq + 5);
			if x < 4 { mask |= 1 << (sq + 6); }
		} else {
			mask |= 1 << (sq + 5);
			if x > 0 { mask |= 1 << (sq + 4); }
		}

		Self(mask)
	}

	pub const fn white_pawn_advance_mask(sq: usize) -> Self {
		let (x, y) = Self::coords_from_index(sq);
		if y == 0 { return Self(0); }

		let mut mask = 0;
		let is_even = y % 2 == 0;

		if is_even {
			mask |= 1 << (sq - 5);
			if x < 4 { mask |= 1 << (sq - 4); }
		} else {
			mask |= 1 << (sq - 5);
			if x > 0 { mask |= 1 << (sq - 6); }
		}

		Self(mask)
	}
	pub const fn generate_queen_advance_table() -> [Self;Self::NB_SQUARES] {
		let mut res=[Self::empty();Self::NB_SQUARES];
		let mut sq=0;
		while sq < Self::NB_SQUARES {
			res[sq] = Self::from_storage(Self::white_pawn_advance_mask(sq).storage() | Self::black_pawn_advance_mask(sq).storage());
			sq +=1;
		}
		res
	}
	
	pub const fn generate_white_pawn_advance_table() -> [Self;Self::NB_SQUARES] {
		let mut res=[Self::empty();Self::NB_SQUARES];
		let mut sq=0;
		while sq < Self::NB_SQUARES {
			res[sq] = Self::white_pawn_advance_mask(sq);
			sq +=1;
		}
		res
	}
	pub const fn generate_black_pawn_advance_table() -> [Self;Self::NB_SQUARES] {
		let mut res=[Self::empty();Self::NB_SQUARES];
		let mut sq=0;
		while sq < Self::NB_SQUARES {
			res[sq] = Self::black_pawn_advance_mask(sq);
			sq +=1;
		}
		res
	}
	pub const fn compute_ray_between_mask_checkers(from: usize, to: usize) -> Self {
		let (fx_half, fy) = Self::coords_from_index(from);
		let (tx_half, ty) = Self::coords_from_index(to);

		// Reconstruction des X réels (0..9)
		let fx = (fx_half * 2 + (fy % 2 == 0) as u8) as i16;
		let tx = (tx_half * 2 + (ty % 2 == 0) as u8) as i16;
		let fy = fy as i16;
		let ty = ty as i16;

		let dx = tx - fx;
		let dy = ty - fy;

		// Vérification diagonale stricte
		if dx.abs() != dy.abs() || dx == 0 {
			return Self::empty();
		}

		let step_x = dx.signum();
		let step_y = dy.signum();

		let mut bb = Self::empty();
		let mut x = fx + step_x;
		let mut y = fy + step_y;

		// On avance jusqu'à la case 'to' exclue
		while x != tx || y != ty {
			// On ne remplit que si on est sur une case jouable (x+y impair)
			if (x + y) % 2 != 0 {
				let idx = Self::index_from_coords((x / 2) as u8, y as u8);
				bb = Self::from_storage(bb.storage() | (1 << idx));
			}
			x += step_x;
			y += step_y;
		}

		bb
	}
	pub const fn generate_ray_between_table_checkers() -> [[Self; Self::NB_SQUARES]; Self::NB_SQUARES] {
		let mut table = [[Self::from_storage(0); Self::NB_SQUARES]; Self::NB_SQUARES];
		let mut from = 0;

		while from < Self::NB_SQUARES {
			let mut to = 0;
			while to < Self::NB_SQUARES {
				table[from][to] = Self::compute_ray_between_mask_checkers(from, to);
				to += 1;
			}
			from += 1;
		}

		table
	}
	#[inline(always)]
	pub const fn compute_diag_inc_mask_checkers(index: usize) -> Self {
		let (x0_half, y0) = Self::coords_from_index(index);
		let mut bb = Self(0);
		
		// Reconstruction x réel (0..9)
		let x0 = (x0_half * 2 + (y0 % 2 == 0) as u8) as i16;
		let y0 = y0 as i16;

		// Direction 1 : x augmente, y augmente
		let mut x = x0 + 1;
		let mut y = y0 + 1;
		while x < 10 && y < 10 {
			if (x + y) % 2 != 0 {
				let idx = Self::index_from_coords((x / 2) as u8, y as u8);
				bb = Self::from_storage(bb.storage() | (1 << idx));
			}
			x += 1;
			y += 1;
		}

		// Direction 2 : x diminue, y diminue
		let mut x = x0 - 1;
		let mut y = y0 - 1;
		while x >= 0 && y >= 0 {
			if (x + y) % 2 != 0 {
				let idx = Self::index_from_coords((x / 2) as u8, y as u8);
				bb = Self::from_storage(bb.storage() | (1 << idx));
			}
			x -= 1;
			y -= 1;
		}

		bb
	}

	pub const fn generate_diag_inc_table_checkers() -> [Self; Self::NB_SQUARES] {
		let mut arr = [Self(0); Self::NB_SQUARES];
		let mut i = 0;
		while i < Self::NB_SQUARES {
			arr[i] = Self::compute_diag_inc_mask_checkers(i);
			i += 1;
		}
		arr
	}

	/// Computes descending diagonal mask (top-left → bottom-right) for square at `index`.
	#[inline(always)]
	pub const fn compute_diag_dec_mask_checkers(index: usize) -> Self {
		let (x0_half, y0) = Self::coords_from_index(index);
		let mut bb = Self(0);
		let x0 = (x0_half * 2 + (y0 % 2 == 0) as u8) as i16;
		let y0 = y0 as i16;

		let mut x = x0 + 1;
		let mut y = y0 - 1;
		while x < 10 && y >= 0 {
			if (x + y) % 2 != 0 { 
				let idx = Self::index_from_coords((x / 2) as u8, y as u8);
				bb = Self::from_storage(bb.storage() | (1 << idx));
			}
			x += 1;
			y -= 1;
		}

		let mut x = x0 - 1;
		let mut y = y0 + 1;
		while x >= 0 && y < 10 {
			if (x + y) % 2 != 0 {
				let idx = Self::index_from_coords((x / 2) as u8, y as u8);
				bb = Self::from_storage(bb.storage() | (1 << idx));
			}
			x -= 1;
			y += 1;
		}
		
		bb
	}
	pub const fn generate_diag_dec_table_checkers() -> [Self; Self::NB_SQUARES] {
		let mut arr = [Self(0); Self::NB_SQUARES];
		let mut i = 0;
		while i < Self::NB_SQUARES {
			arr[i] = Self::compute_diag_dec_mask_checkers(i);
			i += 1;
		}
		arr
	}
	#[inline]
	const fn compute_ray_mask_checkers(index: usize, dx: isize, dy: isize) -> Self {
		let (x_half, y0) = Self::coords_from_index(index);
		let mut bb = 0u64;
		
		let mut x = (x_half * 2 + (y0 % 2 == 0) as u8) as isize;
		let mut y = y0 as isize;

		loop {
			x += dx;
			y += dy;

			if x < 0 || x >= 10 || y < 0 || y >= 10 {
				break;
			}

			if (x + y) % 2 != 0 {
				let idx = (y as usize * 5) + (x as usize / 2);
				bb |= 1 << idx;
			}
		}

		Self(bb)
	}
	/// Generates a table of ray for all squares.
	pub const fn generate_ray_table_checkers(dx: isize, dy: isize) -> [Self; Self::NB_SQUARES] {
		let mut arr = [Self(0); Self::NB_SQUARES];
		let mut i = 0;
		while i < Self::NB_SQUARES {
			arr[i] = Self::compute_ray_mask_checkers(i, dx, dy);
			i += 1;
		}
		arr
	}

	pub const fn generate_pawn_jumps_tables() -> [[(Self, u8, u8); 4]; Self::NB_SQUARES] {
		let mut tables = [[(Self(0), Self::NB_SQUARES as u8, Self::NB_SQUARES as u8); 4]; Self::NB_SQUARES];
		let mut sq = 0;
		while sq < Self::NB_SQUARES {
			let (x, y) = Self::coords_from_index(sq);
			let is_even = y % 2 == 0;

			if y >= 2 {
				// Haut-Gauche (Saut -11)
				if x > 0 { 
					// Correction : sq-5 si pair, sq-6 si impair
					let mid = if is_even { sq - 5 } else { sq - 6 };
					tables[sq][0] = (Self::from_index(sq - 11), (sq - 11) as u8, mid as u8); 
				}
				// Haut-Droite (Saut -9)
				if x < 4 { 
					// Correction : sq-4 si pair, sq-5 si impair
					let mid = if is_even { sq - 4 } else { sq - 5 };
					tables[sq][1] = (Self::from_index(sq - 9), (sq - 9) as u8, mid as u8); 
				}
			}

			if y <= 7 {
				// Bas-Gauche (Saut +9)
				if x > 0 { 
					// Correction : sq+5 si pair, sq+4 si impair
					let mid = if is_even { sq + 5 } else { sq + 4 };
					tables[sq][2] = (Self::from_index(sq + 9), (sq + 9) as u8, mid as u8); 
				}
				// Bas-Droite (Saut +11)
				if x < 4 { 
					// Correction : sq+6 si pair, sq+5 si impair
					let mid = if is_even { sq + 6 } else { sq + 5 };
					tables[sq][3] = (Self::from_index(sq + 11), (sq + 11) as u8, mid as u8); 
				}
			}
			sq += 1;
		}
		tables
	}
	pub const WHITE_ADVANCE: [Self;Self::NB_SQUARES] = Self::generate_white_pawn_advance_table();
	pub const BLACK_ADVANCE: [Self;Self::NB_SQUARES] = Self::generate_black_pawn_advance_table();
	pub const JUMPS: [[(Self, u8, u8); 4]; Self::NB_SQUARES] = Self::generate_pawn_jumps_tables();
	pub const QUEEN_NEIGHBORS: [Self;Self::NB_SQUARES] = Self::generate_queen_advance_table();
	pub const DIAG_INC_MASK: [Self;Self::NB_SQUARES] = Self::generate_diag_inc_table_checkers();
	pub const DIAG_DEC_MASK: [Self;Self::NB_SQUARES] = Self::generate_diag_dec_table_checkers();
	pub const BETWEEN_MASKS: [[Self; Self::NB_SQUARES]; Self::NB_SQUARES] = Self::generate_ray_between_table_checkers();
	pub const RAY_NE: [Self; 50] = Self::generate_ray_table_checkers(1, -1);
	pub const RAY_NW: [Self; 50] = Self::generate_ray_table_checkers(-1, -1);
	pub const RAY_SE: [Self; 50] = Self::generate_ray_table_checkers(1, 1);
	pub const RAY_SW: [Self; 50] = Self::generate_ray_table_checkers(-1, 1);
}

impl Display for Bitboard5x10Checkers10 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "  ")?;
		for c in 0..Self::WIDTH {
			write!(f, "  {:2}", c+1)?;
		}
		writeln!(f)?;
		for row in 0..Self::HEIGHT {
			if row % 2 == 1 {
				write!(f, "{:2}", (row)*Self::WIDTH+1)?;
			} else {
				write!(f, "  ")?;
			}
			for col in 0..Self::WIDTH * 2 {
				if (row+col) % 2 == 1 {
					write!(f, "|{}", if self.get(col/2, row) {'.'} else {' '})?;
				} else {
					write!(f, "| ")?;
				}
			}
			if row % 2 == 0 {
				writeln!(f, "|{}", (row+1)*Self::WIDTH)?;
			} else {
				writeln!(f, "|")?;
			}
			
		}
		write!(f, "  ")?;
		for c in Self::WIDTH * (Self::HEIGHT-1)..Self::WIDTH * (Self::HEIGHT-1)+Self::WIDTH {
			write!(f, " {:2} ", c+1)?;
		}
		writeln!(f)
		
	}
}


	#[test]
	fn test_bitboard() {
		println!("{}", Bitboard5x10Checkers10::from_index(20));
		println!("{}", Bitboard5x10Checkers10::black_pawn_advance_mask(20));
		println!("{:?}", Bitboard5x10Checkers10::black_pawn_advance_mask(20));
		println!("{}", Bitboard5x10Checkers10::white_pawn_advance_mask(20));
		println!("{:?}", Bitboard5x10Checkers10::white_pawn_advance_mask(20));
		println!("{}", Bitboard5x10Checkers10::from_index(25));
		println!("{}", Bitboard5x10Checkers10::black_pawn_advance_mask(25));
		println!("{:?}", Bitboard5x10Checkers10::black_pawn_advance_mask(25));
		println!("{}", Bitboard5x10Checkers10::white_pawn_advance_mask(25));
		println!("{:?}", Bitboard5x10Checkers10::white_pawn_advance_mask(25));
		//println!("{}", Bitboard5x10Checkers10::compute_ray_between_mask2(15, 48));
		//println!("{}", Bitboard5x10Checkers10::compute_ray_mask2(15, 1, -1));
		//println!("{}", Bitboard5x10Checkers10::compute_ray_mask2(15, 1, 1));
		//println!("{}", Bitboard5x10Checkers10::compute_ray_mask2(18, -1, 1));
		//println!("{}", Bitboard5x10Checkers10::compute_diag_dec_mask2(18));
		//println!("{}", Bitboard5x10Checkers10::compute_diag_inc_mask2(18));
	}

	#[test]
	fn test_jumps() {
		let index = 27;
		println!("{}", Bitboard5x10Checkers10::from_index(index));
		println!("{}", Bitboard5x10Checkers10::JUMPS[index][0].0 | Bitboard5x10Checkers10::from_index(index));
		println!("{} {}", Bitboard5x10Checkers10::JUMPS[index][0].1, Bitboard5x10Checkers10::JUMPS[index][0].2);
		println!("{}",Bitboard5x10Checkers10::from_index(Bitboard5x10Checkers10::JUMPS[index][0].2 as usize));
		println!("{}", Bitboard5x10Checkers10::JUMPS[index][1].0 | Bitboard5x10Checkers10::from_index(index));
		println!("{} {}", Bitboard5x10Checkers10::JUMPS[index][1].1, Bitboard5x10Checkers10::JUMPS[index][1].2);
		println!("{}", Bitboard5x10Checkers10::JUMPS[index][2].0 | Bitboard5x10Checkers10::from_index(index));
		println!("{} {}", Bitboard5x10Checkers10::JUMPS[index][2].1, Bitboard5x10Checkers10::JUMPS[index][2].2);
		println!("{}", Bitboard5x10Checkers10::JUMPS[index][3].0 | Bitboard5x10Checkers10::from_index(index));
		println!("{} {}", Bitboard5x10Checkers10::JUMPS[index][3].1, Bitboard5x10Checkers10::JUMPS[index][3].2);
		//println!("{}", Bitboard5x10Checkers10::NORTH_BORDER);

	}
	#[test]
	fn test_rays() {
		let index = 27;
		println!("{}", Bitboard5x10Checkers10::from_index(index));
		println!("{}", Bitboard5x10Checkers10::RAY_NE[index]);
		println!("{}", Bitboard5x10Checkers10::RAY_NW[index]);
		println!("{}", Bitboard5x10Checkers10::RAY_SE[index]);
		println!("{}", Bitboard5x10Checkers10::RAY_SW[index]);
	}