use bitboard::{BitIter, Bitboard};
use std::fmt::{self, Display, Formatter};

use kudchuet::{GameResult, Player};

use crate::bitboard::Goban;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
	pub to: u16,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gomoku {
	pub white: Goban,
	pub black: Goban,
	pub turn: Player,
	pub hash: u64,
}
impl Display for Gomoku {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "Turn : {:?}", self.turn)?;

		for y in 0..19 {
			for x in 0..19 {
				let c = if self.white.get(x, y) {
					'W'
				} else if self.black.get(x, y) {
					'B'
				} else {
					'.'
				};

				write!(f, "{} ", c)?;
			}
			writeln!(f)?;
		}

		Ok(())
	}
}
impl Default for Gomoku {
	fn default() -> Self {
		let mut s = Self {
			white: Default::default(),
			black: Default::default(),
			turn: Default::default(),
			hash: Default::default(),
		};
		s.hash = s.compute_zobrist();
		s
	}
}

impl Gomoku {
	#[inline(always)]
	pub fn new() -> Self {
		Default::default()
	}
	#[inline(always)]
	fn occupied(&self) -> Goban {
		self.white.or_const(&self.black)
	}
	#[inline(always)]
	fn free(&self) -> Goban {
		self.occupied().flipped()
	}
}
impl Gomoku {
	#[inline(always)]
	pub fn legal_moves(&self) -> Vec<Move> {
		let mut out = vec![];
		self.legal_moves_inplace(&mut out);
		out
	}
	#[inline]
	pub fn legal_moves_inplace(&self, out: &mut Vec<Move>) {
		let free = self.free();
		//for i in free.iter_bits() {
		//	out.push(Move{to: i as u16});
		//}
		for i in 0..Goban::NB_SQUARES {
			if free.get_at_index(i) {
				out.push(Move { to: i as u16 });
			}
		}
	}
}

impl Gomoku {
	#[inline]
	pub fn play_unchecked(&mut self, mv: Move) {
		match self.turn {
			Player::PLAYER2 => {
				self.white.set_at_index(mv.to as usize);
			}

			Player::PLAYER1 => {
				self.black.set_at_index(mv.to as usize);
			}
			_ => unreachable!(),
		}
		self.update_hash_move(&mv);
		self.turn = self.turn.opponent();
		self.update_hash_turn();
	}
	#[inline]
	pub fn undo_unchecked(&mut self, mv: Move) {
		self.update_hash_undo(&mv);
		match self.turn {
			Player::PLAYER2 => {
				self.black.reset_at_index(mv.to as usize);
			}

			Player::PLAYER1 => {
				self.white.reset_at_index(mv.to as usize);
			}
			_ => unreachable!(),
		}
		self.turn = self.turn.opponent();
		self.update_hash_turn();
	}
}

impl Gomoku {
	#[inline]
	pub fn result(&self) -> GameResult {
		if self.black.has_aligned::<5>() {
			return GameResult::PLAYER1;
		}
		if self.white.has_aligned::<5>() {
			return GameResult::PLAYER2;
		}

		GameResult::OnGoing
	}
	pub fn cell(&self, x: u8, y: u8) -> Cell {
		if self.white.get(x, y) {
			Cell::White
		} else if self.black.get(x, y) {
			Cell::Black
		} else {
			Cell::Empty
		}
	}
}

pub struct Zobrist {
	pub pieces: [[u64; 2]; Goban::NB_SQUARES],
	pub turn: u64,
}

impl Zobrist {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);
		let mut pieces = [[0u64; 2]; Goban::NB_SQUARES];

		let mut i = 0;
		while i < Goban::NB_SQUARES {
			pieces[i][0] = rng.u64();
			pieces[i][1] = rng.u64();
			i += 1;
		}

		Self {
			pieces,
			turn: rng.u64(),
		}
	}
}
impl Gomoku {
	pub const ZOBRIST_KEYS: Zobrist = Zobrist::new(0x15A4CDE);
	#[inline(always)]
	pub fn get_hash(&self) -> u64 {
		self.hash
	}
	fn compute_zobrist(&self) -> u64 {
		let mut h = 0u64;
		for i in self.black.clone().iter_bits() {
			h ^= Self::ZOBRIST_KEYS.pieces[i as usize][0];
		}
		for i in self.white.clone().iter_bits() {
			h ^= Self::ZOBRIST_KEYS.pieces[i as usize][1];
		}
		if self.turn == Player::PLAYER2 {
			h ^= Self::ZOBRIST_KEYS.turn;
		}
		h
	}

	fn update_hash_move(&mut self, m: &Move) {
		let p_idx = if self.turn == Player::PLAYER1 { 0 } else { 1 };
		self.hash ^= Self::ZOBRIST_KEYS.pieces[m.to as usize][p_idx];
	}
	fn update_hash_undo(&mut self, m: &Move) {
		let p_idx = if self.turn == Player::PLAYER1 { 1 } else { 0 };
		self.hash ^= Self::ZOBRIST_KEYS.pieces[m.to as usize][p_idx];
	}
	fn update_hash_turn(&mut self) {
		self.hash ^= Self::ZOBRIST_KEYS.turn
	}
}

#[derive(Clone, Copy)]
pub enum Cell {
	White,
	Black,
	Empty,
}
#[cfg(test)]
mod tests {
	use super::GameResult;
	use super::Gomoku;
	use super::Player;

	#[test]
	fn play_one_move() {
		let mut game = Gomoku::new();
		assert_eq!(game.turn, Player::PLAYER1);
		assert_eq!(game.result(), GameResult::OnGoing);

		let moves = game.legal_moves();
		assert!(!moves.is_empty());

		let mv = moves[0];
		game.play_unchecked(mv);

		assert_eq!(game.turn, Player::PLAYER2);
	}
	#[test]
	fn play() {
		let mut game = Gomoku::new();

		let mut moves = game.legal_moves();
		assert!(!moves.is_empty());
		assert_eq!(game.get_hash(), game.compute_zobrist());
		while !moves.is_empty() {
			let mv = moves[0];
			game.play_unchecked(mv);
			assert_eq!(game.get_hash(), game.compute_zobrist());
			moves = game.legal_moves();
		}
		println!("{}", game);
	}
}
