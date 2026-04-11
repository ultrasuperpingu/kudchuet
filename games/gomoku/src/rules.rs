use std::fmt::{self, Display, Formatter};
use bitboard::Bitboard;

use kudchuet::{GameResult, Player};

use crate::bitboard::Goban;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
	pub to: u16,
}
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gomoku {
	pub white: Goban,
	pub black: Goban,
	pub turn: Player,
}
impl Display for Gomoku {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "Turn : {:?}", self.turn)?;

		for y in 0..19 {
			for x in 0..19 {
				
				let c = if self.white.get(x,y) {
					'W'
				} else if self.black.get(x,y) {
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
		let mut out= vec![];
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
				out.push(Move{to: i as u16});
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
			_ => unreachable!()
		}
		self.turn = self.turn.opponent();
	}

}

impl Gomoku {
	#[inline]
	pub fn result(&self) -> GameResult {
		if self.black.has_n_aligned(5) {
			return GameResult::Player1;
		}
		if self.white.has_n_aligned(5) {
			return GameResult::Player2;
		}

		GameResult::OnGoing
	}
	pub fn cell(&self, x:u8, y:u8) -> Cell {
		if self.white.get(x, y) {
			Cell::White
		} else if self.black.get(x, y) {
			Cell::Black
		} else {
			Cell::Empty
		}
	}
}

#[derive(Clone, Copy)]
pub enum Cell {
	White,
	Black,
	Empty
}
#[cfg(test)]
mod tests {
	use super::Gomoku;
	use super::Player;
	use super::GameResult;

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
		while !moves.is_empty() {
			let mv = moves[0];
			game.play_unchecked(mv);
			moves = game.legal_moves();
		}
		println!("{}", game);
	}
}