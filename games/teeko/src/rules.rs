use bitboard::{BitIter, Bitboard};
use kudchuet::{GameOutcome, Player};
use std::fmt::{self, Display, Formatter};

use crate::bitboard::Bitboard5x5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
	pub from: Option<u8>,
	pub to: u8,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Teeko {
	pub black: Bitboard5x5,
	pub red: Bitboard5x5,
	pub turn: u8,
}
impl Display for Teeko {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "Turn : {:?}", self.turn)?;

		for y in 0..5 {
			for x in 0..5 {
				let c = if self.black.get(x, y) {
					'B'
				} else if self.red.get(x, y) {
					'R'
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

impl Teeko {
	#[inline(always)]
	pub fn new() -> Self {
		Default::default()
	}
	fn empty() -> Self {
		Self {
			black: Bitboard5x5::empty(),
			red: Bitboard5x5::empty(),
			turn: 0,
		}
	}
	#[inline(always)]
	pub fn occupied(&self) -> Bitboard5x5 {
		self.black | self.red
	}
	pub fn free(&self) -> Bitboard5x5 {
		self.occupied().flipped()
	}
}
impl Teeko {
	#[inline(always)]
	pub fn legal_moves(&self) -> Vec<Move> {
		let mut out = vec![];
		self.legal_moves_inplace(&mut out);
		out
	}
	#[inline]
	pub fn legal_moves_inplace(&self, out: &mut Vec<Move>) {
		if self.turn < 8 {
			out.reserve(25);
			let empty = if self.turn == 0 {
				Bitboard5x5::CENTER.flipped()
			} else {
				self.free()
			};
			for m in empty.iter_bits() {
				out.push(Move {
					from: None,
					to: m as u8,
				});
			}
			return;
		}
		out.reserve(16);
		match self.turn % 2 {
			0 => self.moves_from_mask(self.black, self.free(), out),
			1 => self.moves_from_mask(self.red, self.free(), out),
			_ => unreachable!(),
		}
	}

	#[inline]
	fn moves_from_mask(
		&self,
		from_mask: Bitboard5x5,
		target_mask: Bitboard5x5,
		out: &mut Vec<Move>,
	) {
		for from in from_mask.iter_bits() {
			let mask = target_mask.neighbors_8(from as usize);
			for to in mask.iter_bits() {
				out.push(Move {
					from: Some(from as u8),
					to: to as u8,
				});
			}
		}
	}
}

impl Teeko {
	pub fn get_hash(&self) -> u64 {
		let mut key = 0u64;
		key |= *self.black.storage() as u64;
		key |= (*self.red.storage() as u64) << 25;
		key |= (self.turn as u64) << 50;
		kudchuet::utils::fibo_hash_64(key)
	}
}
impl Teeko {
	pub fn play(&mut self, mv: Move) -> bool {
		if !self.legal_moves().contains(&mv) {
			return false;
		}
		self.play_unchecked(mv);
		true
	}
	#[inline]
	pub fn play_unchecked(&mut self, mv: Move) {
		if self.turn < 8 {
			match self.turn % 2 {
				0 => {
					self.black.set_at_index(mv.to as usize);
				}

				1 => {
					self.red.set_at_index(mv.to as usize);
				}
				_ => unreachable!(),
			}
			self.turn += 1;
			return;
		}
		let from = Bitboard5x5::from_index(mv.from.unwrap() as usize);
		let to = Bitboard5x5::from_index(mv.to as usize);

		match self.turn % 2 {
			0 => {
				self.black = (self.black & !from) | to;
				self.turn += 1;
			}

			1 => {
				self.red = (self.red & !from) | to;
				self.turn -= 1;
			}
			_ => unreachable!(),
		}
	}
}

impl Teeko {
	#[inline]
	pub fn get_cell(&self, x: u8, y: u8) -> Option<Player> {
		if self.black.get(x, y) {
			Some(Player::PLAYER1)
		} else if self.red.get(x, y) {
			Some(Player::PLAYER2)
		} else {
			None
		}
	}
	fn set_cell(&mut self, x: u8, y: u8, player: Player) {
		match player {
			Player::PLAYER1 => {
				self.black.set(x, y);
				self.red.reset(x, y);
			}
			Player::PLAYER2 => {
				self.black.reset(x, y);
				self.red.set(x, y);
			}
			_ => unreachable!(),
		}
	}
	#[inline]
	pub fn result(&self) -> GameOutcome {
		if self.black.has_aligned::<4>() {
			GameOutcome::PLAYER1
		} else if self.red.has_aligned::<4>() {
			GameOutcome::PLAYER2
		} else {
			// Squares
			let sq = self.black
				& self.black.shifted_w()
				& self.black.shifted_s()
				& self.black.shifted_sw();

			if sq.any() {
				return GameOutcome::PLAYER1;
			}

			let sq = self.red & self.red.shifted_w() & self.red.shifted_s() & self.red.shifted_sw();

			if sq.any() {
				return GameOutcome::PLAYER2;
			}
			GameOutcome::OnGoing
		}
	}
}
impl Teeko {
	pub fn to_fen(&self) -> String {
		let mut rows = Vec::new();
		for y in 0..5 {
			let mut row = String::new();
			for x in 0..5 {
				let c = match self.get_cell(x, y) {
					Some(Player::PLAYER1) => 'b',
					Some(Player::PLAYER2) => 'r',
					Some(Player(_)) => unreachable!(),
					None => '.',
				};
				row.push(c);
			}
			rows.push(row);
		}
		let board_str = rows.join("/");
		let player_str = match self.turn % 2 {
			0 => "b",
			1 => "r",
			_ => "?",
		};
		format!("{} {}", board_str, player_str)
	}
	pub fn from_fen(pos_str: &str) -> Result<Self, String> {
		let mut game = Teeko::empty();
		let mut parts = pos_str.split_whitespace();
		let board_part = parts.next().ok_or("Missing board part")?;
		let player_part = parts.next().ok_or("Missing player part")?;

		for (y, row) in board_part.split('/').enumerate() {
			if y >= 5 {
				return Err("Too many rows".into());
			}
			for (x, ch) in row.chars().enumerate() {
				if x >= 5 {
					return Err("Too many columns".into());
				}
				let player = match ch {
					'b' => Some(Player::PLAYER1),
					'r' => Some(Player::PLAYER2),
					'.' => None,
					_ => return Err(format!("Invalid character: {}", ch)),
				};
				if let Some(p) = player {
					game.set_cell(x as u8, y as u8, p);
				}
			}
		}
		game.turn = game.black.count() as u8 + game.red.count() as u8;
		let game_turn = match player_part {
			"b" => 0,
			"r" => 1,
			_ => return Err("Invalid player indicator".into()),
		};
		if game.turn == 8 {
			game.turn += game_turn;
		} else if game.turn % 2 != game_turn {
			return Err("Player turn incoherent".into());
		}

		Ok(game)
	}
}
#[cfg(test)]
mod tests {
	use super::GameOutcome;
	use super::Teeko;

	#[test]
	fn play() {
		let mut game = Teeko::new();

		let mut moves = game.legal_moves();
		println!("{:?}", moves);
		assert_eq!(moves.len(), 24);
		let mut i = 0;
		while game.result() == GameOutcome::OnGoing && i < 10000 {
			let mv = moves[0];
			game.play_unchecked(mv);
			if i < 8 {
				assert_eq!(game.turn, i as u8 + 1);
			} else {
				assert!(game.turn == 8 || game.turn == 9);
			}
			moves.clear();
			game.legal_moves_inplace(&mut moves);
			i += 1;
		}
		println!("{}", game);
	}
	#[test]
	fn fen_roundtrip() {
		let game = Teeko::new();
		let fen = game.to_fen();
		let game2 = Teeko::from_fen(&fen).unwrap();
		assert_eq!(game.black, game2.black);
		assert_eq!(game.red, game2.red);
		assert_eq!(game.turn, game2.turn);
	}
	#[test]
	fn test_endgame() {
		let game = Teeko::from_fen(&"...../...bb/..rr./.rrb./.b... b").unwrap();
		println!("{}", game);
		println!("{:?}", game.result());
		assert_eq!(game.result(), GameOutcome::OnGoing);
		let game = Teeko::from_fen(&"...../...bb/..rr./.brr./.b... b").unwrap();
		println!("{}", game);
		println!("{:?}", game.result());
		assert_eq!(game.result(), GameOutcome::PLAYER2);
	}
}
