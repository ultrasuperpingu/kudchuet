use std::hash::Hash;

use std::fmt::{self, Display};

use bitboard::{BitIter, Bitboard};

use kudchuet::Player;
use super::bitboard::BitboardAbalone;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
	Empty,
	Black,
	White,
}

impl Cell {
	pub fn from_player(p: Player) -> Cell {
		match p {
			Player::PLAYER1 => Cell::Black,
			Player::PLAYER2 => Cell::White,
			_ => unreachable!(),
		}
	}
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Hex {
	pub q: i8,
	pub r: i8,
}

impl Display for Hex {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let line_idx = (4 - self.r) as u8;
		let col = (self.q + 4) as u8 + 1;
		let letter = (b'a' + line_idx) as char;

		write!(f, "{}{}", letter, col)
	}
}


impl Hex {
	pub const DIRS: [Hex; 6] = [
		Hex { q: 1,  r: 0  }, //E
		Hex { q: 1,  r: -1 }, //SE
		Hex { q: 0,  r: -1 }, //SW
		Hex { q: -1, r: 0  }, //W
		Hex { q: -1, r: 1  }, //NW
		Hex { q: 0,  r: 1  }, //NE
	];

	pub const fn add(self, other: Hex) -> Hex {
		Hex { q: self.q + other.q, r: self.r + other.r }
	}
}
pub const fn generate_hexes() -> [Hex; 61] {
	let mut arr = [Hex { q: 0, r: 0 }; 61];
	let mut i = 0;

	let mut q = -4;
	while q <= 4 {
		let mut r = -4;
		while r <= 4 {
			let s = -q - r;
			if s >= -4 && s <= 4 {
				arr[i] = Hex { q, r };
				i += 1;
			}
			r += 1;
		}
		q += 1;
	}

	arr
}

pub const HEXES: [Hex; 61] = generate_hexes();

pub const fn generate_idx_table() -> [[i8; 9]; 9] {
	let mut table = [[-1i8; 9]; 9];
	let mut i = 0;

	while i < 61 {
		let h = HEXES[i];
		let q = (h.q + 4) as usize;
		let r = (h.r + 4) as usize;
		table[q][r] = i as i8;
		i += 1;
	}

	table
}

pub const IDX: [[i8; 9]; 9] = generate_idx_table();


pub const fn generate_ray_table(dir: usize) -> [BitboardAbalone; 61] {
	let mut table = [BitboardAbalone::empty(); 61];
	let mut i = 0;

	while i < 61 {
		let mut mask = 0;
		let mut h = HEXES[i];
		let mut current_index = idx(h); 
		while let Some(index) = current_index {
			mask |= BitboardAbalone::from_index(index).storage();
			h = h.add(Hex::DIRS[dir]);
			current_index = idx(h); 
		}
		table[i] = BitboardAbalone::from_storage(mask);
		i += 1;
	}

	table
}
pub const RAYS: [[BitboardAbalone; 61]; 6] = [
	generate_ray_table(0),
	generate_ray_table(1),
	generate_ray_table(2),
	generate_ray_table(3),
	generate_ray_table(4),
	generate_ray_table(5),
];
/*
#[inline]
pub fn idx(h: Hex) -> Option<usize> {
	let mut i = 0;
	while i < 61 {
		let hh = HEXES[i];
		if hh.q == h.q && hh.r == h.r {
			return Some(i);
		}
		i += 1;
	}
	None
}*/
#[inline]
pub const fn idx(h: Hex) -> Option<usize> {
	if h.q < -4 || h.q > 4 || h.r < -4 || h.r > 4 {
		return None;
	}

	let q = (h.q + 4) as usize;
	let r = (h.r + 4) as usize;

	if q >= 9 || r >= 9 {
		return None;
	}

	let v = IDX[q][r];
	if v >= 0 {
		Some(v as usize)
	} else {
		None
	}
}


#[derive(Clone, Hash, Debug)]
pub struct Abalone {
	pub black: BitboardAbalone,
	pub white: BitboardAbalone,
	pub turn: Player,
	pub black_out: u8,
	pub white_out: u8,
}
impl Default for Abalone {
	fn default() -> Self {
		Self::new_standard()
	}
}
impl Abalone {
	pub fn new_standard() -> Self {
		let mut game = Abalone { white: BitboardAbalone::from_storage(0), black:BitboardAbalone::from_storage(0), turn: Player::PLAYER1, black_out: 0, white_out: 0, };

		for q in -4..=0 {
			game.set_cell(Hex { q: q + 4, r: -4 }, Cell::Black);
			game.set_cell(Hex { q, r: 4 }, Cell::White);
		}

		for q in -4..=1 {
			game.set_cell(Hex { q: q + 3, r: -3 }, Cell::Black);
			game.set_cell(Hex { q, r: 3 }, Cell::White);
		}

		for q in -2..=0 {
			game.set_cell(Hex { q: q + 2, r: -2 }, Cell::Black);
			game.set_cell(Hex { q, r: 2 }, Cell::White);
		}

		game
	}
	#[inline]
	pub fn cell(&self, h: Hex) -> Option<Cell> {
		let i = idx(h)?;
		let mask = 1u64 << i;

		if self.black.storage() & mask != 0 {
			Some(Cell::Black)
		} else if self.white.storage() & mask != 0 {
			Some(Cell::White)
		} else {
			Some(Cell::Empty)
		}
	}

	#[inline]
	pub fn set_cell(&mut self, h: Hex, c: Cell) {
		if let Some(i) = idx(h) {
			let mask = BitboardAbalone::from_index(i);

			self.black &= !mask;
			self.white &= !mask;

			match c {
				Cell::Black => self.black |= mask,
				Cell::White => self.white |= mask,
				Cell::Empty => {}
			}
		}
	}

}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
	Simple {
		from: Hex,
		dir: u8,
	},
	Side {
		start: Hex,
		line_dir: u8,
		side_dir: u8,
		len: u8,
	},
	Push {
		start: Hex,
		dir: u8,
		len: u8,
	},
}

impl std::fmt::Display for Move {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Move::Simple { from, dir } => {
				write!(f, "{}{}", from, from.add(Hex::DIRS[*dir as usize]))
			},
			Move::Side { start, line_dir, side_dir, len } =>  {
				let mut extrimity = *start;
				for _ in 0..*len-1 {
					extrimity=extrimity.add(Hex::DIRS[*line_dir as usize]);
				}
				write!(f, "{}{}{}", start, extrimity, start.add(Hex::DIRS[*side_dir as usize]))
			},
			Move::Push { start, dir, len:_ } => {
				write!(f, "{}{}", start, start.add(Hex::DIRS[*dir as usize]))
			},
		}
	}
}
impl Abalone {
	pub fn is_legal_simple_move(&self, from: Hex, dir: u8) -> bool {
		let to = from.add(Hex::DIRS[dir as usize]);
		self.cell(from) == Some(Cell::from_player(self.turn))
			&& self.cell(to) == Some(Cell::Empty)
	}


	pub fn play_simple_move(&mut self, from: Hex, dir: u8) -> bool {
		if !self.is_legal_simple_move(from, dir) {
			return false;
		}

		let to = from.add(Hex::DIRS[dir as usize]);
		let c = Cell::from_player(self.turn);

		self.set_cell(from, Cell::Empty);
		self.set_cell(to, c);

		self.turn = self.turn.opponent();
		true
	}
	pub fn is_over(&self) -> bool {
		self.black_out >= 6 || self.white_out >= 6
	}
	pub fn winner(&self) -> Option<Player> {
		if self.black_out >= 6 {
			Some(Player::PLAYER2)
		} else if self.white_out >= 6 {
			Some(Player::PLAYER1)
		} else {
			None
		}
	}

}
impl Abalone {
	fn is_legal_push(&self, line: &[Hex], dir: u8) -> bool {
		let opponent = Cell::from_player(self.turn.opponent());

		if line.len() < 2 || line.len() > 3 {
			return false;
		}

		let mut pos = line.last().unwrap().add(Hex::DIRS[dir as usize]);

		match self.cell(pos) {
			// INLINE MOVE
			Some(Cell::Empty) => true,

			// SUMITO: check nb marbles
			Some(c) if c == opponent => {
				let mut opp_count = 1;
				pos = pos.add(Hex::DIRS[dir as usize]);

				while let Some(c2) = self.cell(pos) {
					if c2 == opponent {
						opp_count += 1;
						if opp_count >= line.len() {
							return false;
						}
						pos = pos.add(Hex::DIRS[dir as usize]);
					} else {
						break;
					}
				}

				// square after
				match self.cell(pos) {
					Some(Cell::Empty) => true,
					None => true, // marble out
					_ => false,
				}
			}

			// Allies marble
			_ => false,
		}
	}



	pub fn play_push(&mut self, line: &[Hex], dir: u8) -> bool {
		let player = Cell::from_player(self.turn);

		let head = line[line.len() - 1];
		let mut pos = head.add(Hex::DIRS[dir as usize]);

		let mut chain = line.to_vec();

		loop {
			match self.cell(pos) {
				Some(Cell::Empty) => break,
				Some(c) if c == player => return false, // blocked with owned marble
				Some(_) => chain.push(pos), // opponent marble
				None => {
					chain.push(pos); // last ball out
					break;
				}
			}
			pos = pos.add(Hex::DIRS[dir as usize]);
		}

		for i in (0..chain.len()).rev() {
			let from = chain[i];
			let to = from.add(Hex::DIRS[dir as usize]);

			match self.cell(from) {
				Some(Cell::Black) => self.set_cell(to, Cell::Black),
				Some(Cell::White) => self.set_cell(to, Cell::White),
				_ => {}
			}

			self.set_cell(from, Cell::Empty);
		}

		if let Some(last) = chain.last() {
			if self.cell(*last).is_none() {
				let prev = chain[chain.len() - 2];
				match self.cell(prev) {
					Some(Cell::Black) => self.white_out += 1,
					Some(Cell::White) => self.black_out += 1,
					_ => {}
				}
			}
		}

		self.turn = self.turn.opponent();
		true
	}
	//TODO: test and use
	pub fn play_push_bitboard(&mut self, head: usize, dir: u8) {
		let ray = RAYS[dir as usize][head];
		let blacks = self.black.pext(&ray);
		let whites = self.white.pext(&ray);
		let mut occ = blacks | whites;
		let forward = Self::dir_is_forward(dir as usize);
		let (new_blacks, new_whites, mask) = if !forward {
			//println!("{}", ray.pdep(occ));
			let leading_zeros = occ.leading_zeros();
			occ |= !(u64::MAX >> leading_zeros);
			let first_empty = 64-occ.leading_ones();
			let compact_mask = !((1<< (first_empty-1))-1);
			let mask = ray.pdep(compact_mask);
			//println!("{occ:08b}\n{compact_mask}\n{mask}");
			((blacks>>1)&compact_mask, (whites>>1)&compact_mask, mask)
		} else {
			let first_empty = occ.trailing_ones();
			let compact_mask = (1<< (first_empty))-1;
			let mask = ray.pdep(compact_mask);
			((blacks<<1)&!compact_mask, (whites<<1)&!compact_mask, mask)
		};
		self.black&=!mask;
		self.white&=!mask;
		self.black|=ray.pdep(new_blacks);
		self.white|=ray.pdep(new_whites);
	}


}

#[test]
fn test_play_push_bb() {
	let mut a=Abalone::new_standard();
	//a.play_push_bitboard(56, 4);
	a.play_push_bitboard(26, 4);
	println!("{}", a);
	a.play_push_bitboard(25, 2);
	println!("{}", a);
	
}
impl Abalone {
	pub fn legal_moves_inplace(&self, out: &mut Vec<Move>) {
		out.clear();
		let my_marbles = if self.turn == Player::PLAYER1 {
			self.black
		} else {
			self.white
		};

		for index in my_marbles.iter_bits() {
			let hex = HEXES[index as usize];
			for dir in 0..6 {
				let line = self.detect_line(hex, dir);
				//println!("hex: {:?} cell: {:?} dir: {} line:{:?}", hex, cell, dir, line);
				let len = line.len();
				if len > 3 {
					continue;
				}
				// Simple move (len = 1) TODO: remove
				if len == 1 {
					let to = hex.add(Hex::DIRS[dir as usize]);
					if self.cell(to) == Some(Cell::Empty) {
						out.push(Move::Simple { from: hex, dir });
					}
				}

				// Inline move TODO: accept size one push
				if Self::is_inline_dir(dir, dir) && self.is_legal_push(&line, dir) {
					out.push(Move::Push {
						start: line[0],
						dir,
						len: line.len() as u8,
					});
				}

				// Side moves
				if len > 1 && len <= 3 {
					for side_dir in 0..6 {
						if Self::is_side_dir(dir, side_dir)
							&& self.is_legal_side_move(&line, side_dir)
						{
							out.push(Move::Side {
								start: line[0],
								line_dir: dir,
								side_dir,
								len: len as u8,
							});
						}
					}
				}
			}
		}
	}

	fn is_legal_side_move(&self, line: &[Hex], side_dir: u8) -> bool {
		let side = Hex::DIRS[side_dir as usize];

		for &h in line {
			match self.cell(h.add(side)) {
				Some(Cell::Empty) => {}
				_ => return false,
			}
		}

		true
	}

}

impl Abalone {
		fn detect_line(&self, start: Hex, dir: u8) -> Vec<Hex> {
			let mut line = vec![start];
			let player = Cell::from_player(self.turn);

			// direction dir
			let mut pos = start.add(Hex::DIRS[dir as usize]);
			while let Some(c) = self.cell(pos) {
				if c == player {
					line.push(pos);
					pos = pos.add(Hex::DIRS[dir as usize]);
				} else {
					break;
				}
			}
			line
		}
	fn detect_line_bitboard(&self, start_idx: usize, dir_idx: usize) -> BitboardAbalone {
		let player_bits = if self.turn == Player::PLAYER1 { self.black } else { self.white };
		let ray = RAYS[dir_idx][start_idx];
		
		let ally_on_ray = player_bits & ray;
		
		let obstacles = !ally_on_ray & ray;
		
		if obstacles.is_empty() {
			return ally_on_ray; 
		}

		let forward = Self::dir_is_forward(dir_idx);
		let first_obstacle_idx = if forward {
			obstacles.lsb()
		} else {
			obstacles.msb()
		};

		let consecutive_mask = ray & Self::mask_before(first_obstacle_idx as usize, forward);
		
		ally_on_ray & consecutive_mask
	}
	fn dir_is_forward(dir_idx: usize) -> bool {
		matches!(dir_idx, 0 | 1 | 5)
	}
	fn detect_line_bitboard2(&self, start: usize, dir: usize) -> BitboardAbalone {
		let player = if self.turn == Player::PLAYER1 { self.black } else { self.white };
		let occ = self.black | self.white;

		let start_bit = BitboardAbalone::from_index(start);
		let ray = RAYS[dir][start] | start_bit;

		let occ_c = occ.pext(&ray);

		let first_gap = (!occ_c).trailing_zeros();

		let ally_c = player.pext(&ray);

		let ally_len = ally_c.trailing_ones();

		let len = ally_len.min(first_gap).min(3);

		let mask = (1u64 << len) - 1;

		ray.pdep(mask)
	}
	fn mask_before(idx: usize, forward: bool) -> BitboardAbalone {
		if forward {
			BitboardAbalone::from_storage((1u64 << idx) - 1)
		} else {
			BitboardAbalone::from_storage(!( (1u64 << (idx + 1)) - 1 ))
		}
	}
	fn is_inline_dir(line_dir: u8, dir: u8) -> bool {
		dir == line_dir || dir == (line_dir + 3) % 6
	}

	fn is_side_dir(line_dir: u8, dir: u8) -> bool {
		!Self::is_inline_dir(line_dir, dir)
	}


}
impl Abalone {
	pub fn play_side_move(&mut self, line: &[Hex], side_dir: u8) -> bool {
		let player = Cell::from_player(self.turn);
		let side = Hex::DIRS[side_dir as usize];

		for &h in line {
			match self.cell(h.add(side)) {
				Some(Cell::Empty) => {}
				_ => return false,
			}
		}

		for &h in line.iter().rev() {
			self.set_cell(h, Cell::Empty);
			self.set_cell(h.add(side), player);
		}

		self.turn = self.turn.opponent();
		true
	}


}
fn to_bitboard(hexes: &Vec<Hex>) -> BitboardAbalone {
	let mut bb = BitboardAbalone::EMPTY;
	for h in hexes {
		let index = idx(*h).unwrap();
		bb |= BitboardAbalone::from_index(index);
	}
	bb
}

impl Abalone {
	pub fn play(&mut self, mv: Move) -> bool {
		match mv {
			Move::Simple { from, dir } => {
				self.play_simple_move(from, dir)
			}

			Move::Push { start, dir, len: _ } => {
				//println!("{}", idx(start).unwrap());
				let line = self.detect_line(start, dir);
				#[cfg(debug_assertions)]
				{
					let old = self.detect_line(start, dir);
					let new = self.detect_line_bitboard(idx(start).unwrap(), dir as usize);
					//let new2 = self.detect_line_bitboard2(idx(start).unwrap(), dir as usize);
					debug_assert_eq!(to_bitboard(&old), new);
					//debug_assert_eq!(new, new2);
				}
				self.play_push(&line, dir)
			}

			Move::Side { start, line_dir, side_dir, len: _ } => {
				let line = self.detect_line(start, line_dir);
				#[cfg(debug_assertions)]
				{
					let old = self.detect_line(start, line_dir);
					let new = self.detect_line_bitboard(idx(start).unwrap(), line_dir as usize);
					//let new2 = self.detect_line_bitboard2(idx(start).unwrap(), line_dir as usize);
					debug_assert_eq!(to_bitboard(&old), new);
					//debug_assert_eq!(new, new2);
				}
				self.play_side_move(&line, side_dir)
			}
		}
	}

}
#[cfg(test)]
mod tests {

	use kudchuet::gui::BoardGame;

use crate::{bitboard::BitboardAbalone, rules::{Abalone, RAYS, to_bitboard}};

	#[test]
	fn test_legals() {
		let mut a=Abalone::new_standard();
		//println!("{}", a);
		println!("{}", BitboardAbalone::from_index(30));
		println!("{}", BitboardAbalone::from_index(1));
		println!("{}", BitboardAbalone::from_index(55));
		println!("{}", BitboardAbalone::from_index(56));
		
		println!("{}", RAYS[0][30]);
		println!("{}", RAYS[1][30]);
		println!("{}", RAYS[2][30]);
		println!("{}", RAYS[3][30]);
		println!("{}", RAYS[4][30]);
		println!("{}", RAYS[5][30]);

		//for b in RAYS[0][30].iter_bits() {
		//	println!("0: {}", b);
		//}
		//for b in RAYS[1][30].iter_bits() {
		//	println!("1: {}", b);
		//}
		//for b in RAYS[2][30].iter_bits() {
		//	println!("2: {}", b);
		//}
		//for b in RAYS[3][30].iter_bits() {
		//	println!("3: {}", b);
		//}
		//for b in RAYS[4][30].iter_bits() {
		//	println!("4: {}", b);
		//}
		//for b in RAYS[5][30].iter_bits() {
		//	println!("5: {}", b);
		//}
		println!("{}", a.detect_line_bitboard(50, 0));
		println!("{}", to_bitboard(&a.detect_line(super::HEXES[50], 0)));
		a.play_push_bitboard(56, 4);
		println!("{}", a);
		//let mut out=vec![];
		//a.legal_moves_inplace(&mut out);
		//println!("{}", out.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", "));
		////println!("{}", a.black);
		//println!("{:?}", out.len());
		//a.play(out[0]);
		//println!("{}", a);
		
	}
	#[test]
	fn test_play() {
		let mut rng = kudchuet::utils::Rng::new();
		let mut a=Abalone::new_standard();
		let mut moves = a.legal_moves();
		while !a.is_over() {
			a.play(moves[rng.range(0, moves.len())]);
			moves = a.legal_moves();
			println!("{}", a);
		}
	}
}