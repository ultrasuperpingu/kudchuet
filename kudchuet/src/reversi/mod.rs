pub mod game;
pub mod ihm;
pub mod gui;


use std::{fmt::Display, hash::{Hash, Hasher}};
use bitboard::{BitIter, Bitboard};
use crate::common::bitboards::Bitboard8x8;

pub type Coord = (u8, u8);
pub type BitPos = u8;


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Cell {
	Empty,
	White,
	Black,
}
impl Cell {
	pub fn opponent(&self) -> Self {
		match self {
			Cell::Empty => Cell::Black,
			Cell::White => Cell::Black,
			Cell::Black => Cell::White,
		}
	}
}

impl Hash for Reversi {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.last_stones.hash(state);
		self.all_stones.hash(state);
		self.turn.hash(state);
	}
}

		
impl Default for Reversi {
	fn default() -> Self {
		Self {
			last_stones: Bitboard8x8::from_coords(3,4) | Bitboard8x8::from_coords(4,3),
			all_stones: Bitboard8x8::CENTER,
			turn: Cell::Black,
		}
	}
}
impl Display for Reversi {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for row in (0..8).rev() {
			write!(f, "{:2}", row+1)?;
			for col in 0..8 {
				let char = match self.cell_from_coords(col, row) {
					Cell::Empty => ' ',
					Cell::White => 'O',
					Cell::Black => 'X',
				};
				write!(f, "|{}", char)?;
			}
			writeln!(f, "|")?;
		}
		write!(f, "   ")?;
		for c in 0..8 {
			write!(f, "{} ", (b'a' + c) as char)?;
		}
		writeln!(f)
	}
}
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Reversi
{
	last_stones: Bitboard8x8,
	all_stones: Bitboard8x8,
	turn: Cell,
}
impl Reversi
{
	pub fn turn(&self) -> Cell {
		self.turn
	}
	pub fn cell_from_coords(&self, x:u8, y:u8) -> Cell {
		if !self.all_stones.get(x, y) {
			Cell::Empty
		} else if self.turn() == Cell::White {
			if self.last_stones.get(x, y) {
				Cell::Black
			} else {
				Cell::White
			}
		} else {
			if self.last_stones.get(x, y) {
				Cell::White
			} else {
				Cell::Black
			}
		}
	}

}
const DIR_SHIFT_FUNCS: [fn(&Bitboard8x8) -> Bitboard8x8; 8] = [
	Bitboard8x8::shift_se,
	Bitboard8x8::shift_s,
	Bitboard8x8::shift_sw,
	Bitboard8x8::shift_e,
	Bitboard8x8::shift_w,
	Bitboard8x8::shift_ne,
	Bitboard8x8::shift_n,
	Bitboard8x8::shift_nw,
];

#[inline(always)]
fn flips_dir_compact(
	player: Bitboard8x8,
	opponent: Bitboard8x8,
	_pos: Bitboard8x8,
	ray_mask: Bitboard8x8,
	offset: i16, // seul le signe compte
) -> Bitboard8x8 {
	let ray = ray_mask.storage();
	if ray == 0 {
		return Bitboard8x8::EMPTY;
	}

	let p = player.pext(&ray_mask);
	let o = opponent.pext(&ray_mask);

	//println!("p: {}, o: {}, ray: {}",p,o,ray);
	let len = ray_mask.count();
	if len == 0 {
		return Bitboard8x8::EMPTY;
	}

	let mut x: u64;
	let flips_compact: u64;

	if offset > 0 {
		// voisin = bit 0
		x = 1u64 & o; // comme shift(pos) & opponent

		x |= (x << 1) & o;
		x |= (x << 1) & o;
		x |= (x << 1) & o;
		x |= (x << 1) & o;
		x |= (x << 1) & o;
		x |= (x << 1) & o;

		if ((x << 1) & p) == 0 {
			//println!("offset>0 empty");
			return Bitboard8x8::EMPTY;
		}

		flips_compact = x;
	} else {
		// voisin = bit (len - 1)
		x = (1u64 << (len - 1)) & o; // voisin s'il est adversaire

		x |= (x >> 1) & o;
		x |= (x >> 1) & o;
		x |= (x >> 1) & o;
		x |= (x >> 1) & o;
		x |= (x >> 1) & o;
		x |= (x >> 1) & o;

		if ((x >> 1) & p) == 0 {
			//println!("offset<=0 empty");
			return Bitboard8x8::EMPTY;
		}

		flips_compact = x;
	}

	//println!("flips_compact: {}", flips_compact);
	ray_mask.pdep(flips_compact)
}

#[inline(always)]
#[allow(dead_code)]
fn shift_compact(bits: u64, offset: i32, lane_mask: u64) -> u64 {
	if offset > 0 {
		(bits << 1) & lane_mask
	} else {
		(bits >> 1) & lane_mask
	}
}
/*
#[inline(always)]
fn shift2(bits: Bitboard8x8, offset: i32, mask: Bitboard8x8) -> Bitboard8x8 {
	if offset > 0 {
		bits << (offset as usize) & mask
	} else {
		bits >> (-offset) as usize & mask
	}
}*/

#[inline(always)]
pub(crate) fn flips_dir(player: Bitboard8x8, opponent: Bitboard8x8, pos: Bitboard8x8, dir: usize) -> Bitboard8x8 {
	let shift = DIR_SHIFT_FUNCS[dir];
	//let mask = Bitboard8x8::from_storage(DIR_MASK[dir]);

	let mut x = shift(&pos) & opponent;

	x |= shift(&x) & opponent;
	x |= shift(&x) & opponent;
	x |= shift(&x) & opponent;
	x |= shift(&x) & opponent;
	x |= shift(&x) & opponent;

	if !(shift(&x) & player).is_empty() {
		x
	} else {
		Bitboard8x8::EMPTY
	}
}
/*
#[inline(always)]
pub(crate) fn flips_dir(
	player: Bitboard8x8,
	opponent: Bitboard8x8,
	pos: Bitboard8x8,
	dir: usize,
) -> Bitboard8x8 {
	let off  = DIR_OFFSET[dir];
	//let mask = Bitboard8x8::from_storage(DIR_MASK[dir]);

	// première case dans la direction
	let mut x = pos.shift_by_index(off) & opponent;
	if x.is_empty() {
		return Bitboard8x8::EMPTY;
	}

	let mut flips = x;

	loop {
		// avance encore
		x = x.shift_by_index(off);

		// si on tombe sur un pion du joueur → flips valides
		if !(x & player).is_empty() {
			return flips;
		}

		// si plus d’adversaires → chaîne non encadrée → rien
		x &= opponent;
		if x.is_empty() {
			return Bitboard8x8::EMPTY;
		}

		flips |= x;
	}
}*/

impl Reversi {
	fn flips_from(&self, x: u8, y: u8) -> Bitboard8x8 {
		let index = Bitboard8x8::index_from_coords(x, y);
		let pos = Bitboard8x8::from_index(index);

		// Joueur courant = all ^ last
		let player   = self.all_stones ^ self.last_stones;
		let opponent = self.last_stones;
		
		let mut flips = Bitboard8x8::empty();
		for dir in 0..8 {
			flips |= flips_dir(player, opponent, pos, dir);
		}
		
		// même mapping que DIR_OFFSET
		// 0:+1(E), 1:-1(W), 2:+8(N), 3:-8(S), 4:+9(NE), 5:-9(SW), 6:+7(NW), 7:-7(SE)
		//flips |= flips_dir_compact(player, opponent, pos, Bitboard8x8::ray_e_mask(index),  1);
		//flips |= flips_dir_compact(player, opponent, pos, Bitboard8x8::ray_w_mask(index), -1);
		//flips |= flips_dir_compact(player, opponent, pos, Bitboard8x8::ray_s_mask(index),  8);
		//flips |= flips_dir_compact(player, opponent, pos, Bitboard8x8::ray_n_mask(index), -8);
		//flips |= flips_dir_compact(player, opponent, pos, Bitboard8x8::ray_se_mask(index),  9);
		//flips |= flips_dir_compact(player, opponent, pos, Bitboard8x8::ray_nw_mask(index), -9);
		//flips |= flips_dir_compact(player, opponent, pos, Bitboard8x8::ray_sw_mask(index),  7);
		//flips |= flips_dir_compact(player, opponent, pos, Bitboard8x8::ray_ne_mask(index), -7);

		flips
	}
	pub fn play(&mut self, x: u8, y: u8) -> bool {
		if self.is_over() || !self.is_legal_move(x, y) {
			return false;
		}
		if x > 7 || y > 7 && !self.legal_moves_mask().is_empty() {
			return false;
		}
		self.play_unchecked(x, y);
		true
	}
	#[inline]
	pub fn play_unchecked(&mut self, x: u8, y: u8) {
		if x > 7 || y > 7 {
			self.pass();
			return;
		}
		let idx = (y as u64) * 8 + (x as u64);
		let pos = Bitboard8x8::from_index(idx as usize);

		// compute turned stones
		let flips = self.flips_from(x, y);

		// add the stone
		self.all_stones |= pos;

		// set current player stones in last stones
		self.last_stones^=self.all_stones;
		
		// add flipped stones to current player
		self.last_stones |= flips;
		self.turn = self.turn.opponent();
	}

	#[inline]
	pub fn pass(&mut self) {
		self.last_stones^=self.all_stones;
		self.turn = self.turn.opponent();
	}
}
impl Reversi {
	#[inline]
	pub fn legal_moves(&self, moves: &mut Vec<(u8, u8)>) {
		moves.clear();
		for idx in self.legal_moves_mask().iter_bits() {
			let (x,y) = Bitboard8x8::coords_from_index(idx as usize);
			moves.push((x, y))
		};
		if !self.is_over() && moves.is_empty() {
			moves.push((8, 8))
		}
	}
	#[inline]
	pub fn legal_moves_mask(&self) -> Bitboard8x8 {
		let player   = self.all_stones ^ self.last_stones;
		let opponent = self.last_stones;
		let empty    = !(player | opponent);

		let mut moves = Bitboard8x8::empty();
		

		for dir in 0..8 {
			let shift = DIR_SHIFT_FUNCS[dir];
			//let mask = Bitboard8x8::from_storage(DIR_MASK[dir]);

			// pions adverses adjacents dans la direction
			let mut x = shift(&player) & opponent;

			// propage à travers les chaînes adverses
			x |= shift(&x) & opponent;
			x |= shift(&x) & opponent;
			x |= shift(&x) & opponent;
			x |= shift(&x) & opponent;
			x |= shift(&x) & opponent;
			/*let mut x = player.shift_by_index(off) & opponent;
			let mut y = x;

			loop {
				y = y.shift_by_index(off) & opponent;
				if y.is_empty() {
					break;
				}
				x |= y;
			}*/


			// cases vides derrière une chaîne adverse terminée par un pion du joueur
			moves |= shift(&x) & empty;
		}

		moves
	}
	#[inline]
	pub fn is_legal_move(&self, x: u8, y: u8) -> bool {
		let pos = Bitboard8x8::from_coords(x, y);

		// Already occupied
		if !(self.all_stones & pos).is_empty() {
			return false;
		}
		// flipping at least on stone
		!self.flips_from(x, y).is_empty()
	}
	#[inline]
	pub fn is_over(&self) -> bool {
		// Si le joueur courant a un coup, ce n’est pas fini
		if !self.legal_moves_mask().is_empty() {
			return false;
		}

		// Simuler un "pass" pour regarder les coups de l’adversaire
		let mut tmp = *self;
		tmp.pass(); // change de joueur

		tmp.legal_moves_mask().is_empty()
	}
	#[inline]
	pub fn winner(&self) -> Option<Cell> {
		if !self.is_over() {
			return None;
		}
		// player don't change is is_over, so turn is the last player
		let black = if self.turn == Cell::Black { (self.all_stones ^ self.last_stones).count()} else {self.last_stones.count()};
		let white = if self.turn == Cell::White { (self.all_stones ^ self.last_stones).count()} else {self.last_stones.count()};

		if black > white {
			Some(Cell::Black)
		} else if white > black {
			Some(Cell::White)
		} else {
			None // égalité
		}
	}
	#[inline]
	pub fn is_draw(&self) -> bool {
		if !self.is_over() {
			return false;
		}

		let p1 = (self.all_stones ^ self.last_stones).count();
		let p2 = self.last_stones.count();

		p1 == p2
	}

}

#[cfg(test)]
mod tests {
	use bitboard::Bitboard;

	use crate::common::bitboards::Bitboard8x8;
	use super::Cell;

	use super::{flips_dir_compact, flips_dir};

	use super::Reversi;
	#[test]
	fn test_shift() {
		println!("{}", Bitboard8x8::BORDER.shift_n());
		println!("{}", Bitboard8x8::BORDER.shift_s());
		println!("{}", Bitboard8x8::BORDER.shift_e());
		println!("{}", Bitboard8x8::BORDER.shift_w());
		println!("{}", Bitboard8x8::BORDER.shift_ne());
		println!("{}", Bitboard8x8::BORDER.shift_nw());
		println!("{}", Bitboard8x8::BORDER.shift_se());
		println!("{}", Bitboard8x8::BORDER.shift_sw());

		println!("{}", Bitboard8x8::BORDER.shift(5,5));
		println!("{}", Bitboard8x8::BORDER.shift(-5,5));
		
	}
	
	#[test]
	fn test_play() {
		let mut board = Reversi::default();
		println!("{}\n{}", board.all_stones, board.last_stones);
		println!("{:?}\n{:?}", board.all_stones, board.last_stones);
		println!("{}", board);
		let mut out=vec![];
		board.legal_moves(&mut out);
		assert!(out.contains(&(4, 2)));
		assert!(out.contains(&(5, 3)));
		assert!(out.contains(&(2, 4)));
		assert!(out.contains(&(3, 5)));
		board.play_unchecked(2,4);
		println!("{}", board);

		board.all_stones = Bitboard8x8::from_storage(0xFFFFFFFFFFFFFFFF);
		board.legal_moves(&mut out);
		println!("{}", board);
		println!("{:?}",out);
		assert!(out == []);
		assert!(board.is_over());
		assert!(board.winner() == Some(Cell::White));
		assert!(!board.is_draw());

		board.last_stones = Bitboard8x8::from_storage(0x00000000FFFFFFFF);
		println!("{}", board);
		board.legal_moves(&mut out);
		println!("{:?}", out);
		assert!(out == []);
		assert!(board.is_over());
		assert!(board.winner() == None);
		assert!(board.is_draw());
	}
	fn some_bitboards() -> Vec<Bitboard8x8> {
		let mut v = Vec::new();

		// bitboards avec un seul bit
		for i in 0..64 {
			v.push(Bitboard8x8::from_storage(1u64 << i));
		}

		// bitboards avec deux bits
		for i in 0..64 {
			for j in 0..64 {
				if i != j {
					v.push(Bitboard8x8::from_storage((1u64 << i) | (1u64 << j)));
				}
			}
		}

		// quelques patterns classiques
		v.push(Bitboard8x8::from_storage(0x00FF_0000_0000_FF00));
		v.push(Bitboard8x8::from_storage(0x8100_0000_0000_0081));
		v.push(Bitboard8x8::from_storage(0xFFFF_FFFF_FFFF_FFFF));

		v
	}
	#[test]
	fn simple_pos() {
		//let player = Bitboard8x8::from_storage(1);
		//let opponent = Bitboard8x8::from_storage(0b01111110);
		//let index = 7;
		//let pos = Bitboard8x8::from_index(index);
		//
		//let ray_mask = Bitboard8x8::compute_ray_w_mask(index);
		//println!("ray_mask:\n{ray_mask}\nplayer:{player}\nopponent:{opponent}");
//
		//let a = flips_dir(player, opponent, pos, 1);
		//let b = flips_dir_compact(player, opponent, pos, ray_mask, crate::reversi::rules::DIR_OFFSET[1]);
		//println!("{a}");
		//if a != b {
		//	println!("MISMATCH: pos={index}, a=\n{a}\n b=\n{b}");
		//	return;
		//}

		//let player = Bitboard8x8::from_storage(1);
		//let opponent = Bitboard8x8::WEST_BORDER&!Bitboard8x8::CORNERS;
		//let index = 56;
		//let pos = Bitboard8x8::from_index(index);
		//
		//let ray_mask = Bitboard8x8::compute_ray_s_mask(index);
		//println!("ray_mask:\n{ray_mask}\nplayer:{player}\nopponent:{opponent}");
//
		//let a = flips_dir(player, opponent, pos, 3);
		//let b = flips_dir_compact(player, opponent, pos, ray_mask, crate::reversi::rules::DIR_OFFSET[3]);
		//println!("{a}");
		//if a != b {
		//	println!("MISMATCH: pos={index}, a=\n{a}\n b=\n{b}");
		//	return;
		//}

		let player = Bitboard8x8::from_storage(1);
		let opponent = Bitboard8x8::from_storage(0b_00001000_00000100_00000010_00000000);
		let index = 36;
		let pos = Bitboard8x8::from_index(index);
		
		let ray_mask = Bitboard8x8::compute_ray_sw_mask(index);
		println!("ray_mask:\n{ray_mask}\nplayer:{player}\nopponent:{opponent}");

		let a = flips_dir(player, opponent, pos, 5);
		let b = flips_dir_compact(player, opponent, pos, ray_mask, 1);
		println!("{a}");
		if a != b {
			println!("MISMATCH: pos={index}, a=\n{a}\n b=\n{b}");
			return;
		}
	}
	#[test]
	fn check_equiv() {
		//println!("ray_w_mask(2) = {}", Bitboard8x8::compute_ray_n_mask(2));

		for player in some_bitboards() {
			for opponent in some_bitboards() {
				if !(player & opponent).is_empty() { continue; }
				//println!("player:\n{}\nopponent:\n{}",player , opponent);
				for pos_idx in 0..64 {
					let pos = Bitboard8x8::from_storage(1u64 << pos_idx);

					// On récupère l’index pour les rayons
					let index = pos_idx;

					// Pour chaque direction, on compare l’ancienne version et la nouvelle
					for dir in 0..8 {
						let a = flips_dir(player, opponent, pos, dir);

						let ray_mask = match dir {
							0 => Bitboard8x8::compute_ray_e_mask(index),
							1 => Bitboard8x8::compute_ray_w_mask(index),
							2 => Bitboard8x8::compute_ray_n_mask(index),
							3 => Bitboard8x8::compute_ray_s_mask(index),
							4 => Bitboard8x8::compute_ray_ne_mask(index),
							5 => Bitboard8x8::compute_ray_sw_mask(index),
							6 => Bitboard8x8::compute_ray_nw_mask(index),
							7 => Bitboard8x8::compute_ray_se_mask(index),
							_ => unreachable!(),
						};

						let b = flips_dir_compact(player, opponent, pos, ray_mask, -1);

						if a != b {
							println!("MISMATCH: dir={dir}, pos={pos_idx}, a={a:?}, b={b:?}");
							return;
						}
					}
				}
			}
		}

		println!("OK: flips_dir == flips_dir_compact pour tous les tests");
	}


}