use bitboard::{BitIter, Bitboard};
use std::fmt::{self, Display, Formatter};
use crate::common::{GameResult, Player, bitboards::Bitboard5x5};

pub mod gui;
pub mod game;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
	pub from: u8,
	pub to: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreeMusketeers {
	pub musketeers: Bitboard5x5,
	pub guards: Bitboard5x5,
	pub turn: u8,
}
impl Display for ThreeMusketeers {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "Turn : {:?}", self.turn)?;

		for y in 0..5 {
			for x in 0..5 {
				
				let c = if self.musketeers.get(x,y) {
					'M'
				} else if self.guards.get(x,y) {
					'G'
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

impl Default for ThreeMusketeers {
	#[inline(always)]
	fn default() -> Self {
		let corners = Bitboard5x5::CORNERS | Bitboard5x5::CENTER;
		let musketeers = Bitboard5x5::compute_diag_inc_mask(0) & corners;

		let guards = Bitboard5x5::FULL & !musketeers;
		Self {
			musketeers,
			guards,
			turn: 0,
		}
	}
}

impl ThreeMusketeers {
	#[inline(always)]
	pub fn new() -> Self {
		Default::default()
	}
	fn empty() -> Self {
		Self {
			musketeers: Bitboard5x5::empty(),
			guards: Bitboard5x5::empty(),
			turn: 0,
		}
	}
	#[inline(always)]
	fn occupied(&self) -> Bitboard5x5 {
		self.musketeers | self.guards
	}
}
impl ThreeMusketeers {
	#[inline(always)]
	pub fn legal_moves(&self) -> Vec<Move> {
		let mut out= vec![];
		self.legal_moves_inplace(&mut out);
		out
	}
	#[inline]
	pub fn legal_moves_inplace(&self, out: &mut Vec<Move>) {
		out.clear();
		match self.turn {
			0 => {
				out.reserve(12);
				self.legal_moves_musketeers_inplace(out)
			},
			1 => {
				out.reserve(24);
				self.legal_moves_guards_inplace(out)
			},
			_ => unreachable!()
		}
	}
	#[inline(always)]
	pub fn legal_moves_musketeers(&self) -> Vec<Move> {
		let mut out= vec![];
		self.legal_moves_musketeers_inplace(&mut out);
		out
	}
	#[inline(always)]
	fn legal_moves_musketeers_inplace(&self, out: &mut Vec<Move>) {
		self.moves_from_mask(self.musketeers, self.guards, out)
	}
	#[inline(always)]
	fn legal_moves_guards_inplace(&self, out: &mut Vec<Move>) {
		self.moves_from_mask(self.guards, !self.occupied(), out)
	}
	#[inline]
	fn moves_from_mask(&self, from_mask: Bitboard5x5, target_mask: Bitboard5x5, out: &mut Vec<Move>) {
		for from in from_mask.iter_bits() {
			let mask = target_mask.neighbors_ortho(from as usize);
			for to in mask.iter_bits() {
				out.push(Move { from: from as u8, to: to as u8 });
			}
		}
	}
	#[inline]
	fn has_legal_move_musketeers(&self) -> bool {
		for from in self.musketeers.iter_bits() {
			let mask = self.guards.neighbors_ortho(from as usize);
			if mask.any() {
				return true;
			}
		}
		false
	}

}

impl ThreeMusketeers {
	fn get_hash(&self) -> u64 {
		let mut key = 0u64;
		key |= self.musketeers.storage() as u64;
		key |= (self.guards.storage() as u64) << 25;
		key |= (self.turn as u64) << 50;
		crate::utils::fibo_hash_64(key)
	}
}
impl ThreeMusketeers {
	pub fn play(&mut self, mv: Move) -> bool {
		if !self.legal_moves().contains(&mv) {
			return false;
		}
		self.play_unchecked(mv);
		true
	}
	#[inline]
	pub fn play_unchecked(&mut self, mv: Move) {
		let from = Bitboard5x5::from_index(mv.from as usize);
		let to = Bitboard5x5::from_index(mv.to as usize);
		
		match self.turn {
			0 => {
				self.musketeers = (self.musketeers & !from) | to;
				self.guards = self.guards & !to;
				self.turn = 1;
			}

			1 => {
				self.guards = (self.guards & !from) | to;
				self.turn = 0;
			}
			_ => unreachable!()
		}
	}

}

impl ThreeMusketeers {
	#[inline]
	pub fn musketeers_aligned(&self) -> bool {
		let m = self.musketeers;
		let (x, y) = Bitboard5x5::coords_from_index(m.lsb() as usize);
		if (m & Bitboard5x5::row_mask(y)).count() == 3 {
			return true;
		}
		if (m & Bitboard5x5::col_mask(x)).count() == 3 {
			return true;
		}

		false
	}
	#[inline]
	pub fn get_cell(&self, x: u8, y: u8) -> Option<Player> {
		if self.musketeers.get(x, y) {
			Some(Player::PLAYER1)
		} else if self.guards.get(x, y) {
			Some(Player::PLAYER2)
		} else {
			None
		}
	}
	fn set_cell(&mut self, x: u8, y: u8, player:Player) {
		match player {
			Player::PLAYER1 => {
				self.musketeers.set(x, y);
				self.guards.reset(x, y);
			},
			Player::PLAYER2 => {
				self.musketeers.reset(x, y);
				self.guards.set(x, y);
			},
			_ => unreachable!(),
		}
	}
	#[inline]
	pub fn result(&self) -> GameResult {
		if self.musketeers_aligned() {
			return GameResult::Player2;
		}
		if self.guards.is_empty() {
			return GameResult::Player1;
		}
		if self.turn == 0 {
			if !self.has_legal_move_musketeers() {
				return GameResult::Player1;
			}
		}

		GameResult::OnGoing
	}
}
impl ThreeMusketeers {
	pub fn to_fen(&self) -> String {
		let mut rows = Vec::new();
		for y in 0..5 {
			let mut row = String::new();
			for x in 0..5 {
				let c = match self.get_cell(x, y) {
					Some(Player::PLAYER1) => 'm',
					Some(Player::PLAYER2) => 'g',
					Some(Player::RandomMove) => unreachable!(),
					Some(Player::Player(_)) => unreachable!(),
					None => '.',
				};
				row.push(c);
			}
			rows.push(row);
		}
		let board_str = rows.join("/");
		let player_str = match self.turn {
			0 => "m",
			1 => "g",
			_ => "?",
		};
		format!("{} {}", board_str, player_str)
	}
	fn from_fen(pos_str: &String) -> Result<Self, String> {
		let mut game = ThreeMusketeers::empty();
		let mut parts = pos_str.split_whitespace();
		let board_part = parts.next().ok_or("Missing board part")?;
		let player_part = parts.next().ok_or("Missing player part")?;

	
		for (y, row) in board_part.split('/').enumerate() {
			if y >= 5 as usize {
				return Err("Too many rows".into());
			}
			for (x, ch) in row.chars().enumerate() {
				if x >= 5 as usize {
					return Err("Too many columns".into());
				}
				let player = match ch {
					'm' => Some(Player::PLAYER1),
					'g' => Some(Player::PLAYER2),
					'.' => None,
					_ => return Err(format!("Invalid character: {}", ch)),
				};
				if let Some(p) = player {
					game.set_cell(x as u8, y as u8, p);
				}
			}
		}

		game.turn = match player_part {
			"m" => 0,
			"g" => 1,
			_ => return Err("Invalid player indicator".into()),
		};

		Ok(game)
	}
}
#[cfg(test)]
mod tests {
	use super::ThreeMusketeers;
	use super::GameResult;
		
	#[test]
	fn play_one_move() {
		let mut game = ThreeMusketeers::new();
		assert_eq!(game.turn, 0);
		assert_eq!(game.result(), GameResult::OnGoing);

		let moves = game.legal_moves();
		assert!(!moves.is_empty());

		let mv = moves[0];
		game.play_unchecked(mv);

		assert_eq!(game.turn, 1);
	}
	#[test]
	fn play() {
		let mut game = ThreeMusketeers::new();
		
		let mut moves = game.legal_moves();
		assert!(!moves.is_empty());
		while !moves.is_empty() {
			let mv = moves[0];
			game.play_unchecked(mv);
			moves = game.legal_moves();
		}
		println!("{}", game);
	}
	#[test]
	fn fen_roundtrip() {
		let game = ThreeMusketeers::new();
		let fen = game.to_fen();
		let game2 = ThreeMusketeers::from_fen(&fen).unwrap();
		assert_eq!(game.musketeers, game2.musketeers);
		assert_eq!(game.guards, game2.guards);
		assert_eq!(game.turn, game2.turn);
	}
}