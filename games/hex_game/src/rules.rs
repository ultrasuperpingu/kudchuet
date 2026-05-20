use bitboard::BitIter;
use kudchuet::{gui::BoardMove, GameOutcome, Player};

use crate::bitboard::HexBoard;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hex {
	pub blue: HexBoard,
	pub red: HexBoard,
	pub current_player: Player,
	pub hash: u64,
}

pub struct Zobrist {
	pub pieces: [[u64; 2]; HexBoard::NB_SQUARES],
	pub turn: u64,
}

impl Zobrist {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);
		let mut pieces = [[0; 2]; HexBoard::NB_SQUARES];
		let mut i = 0;
		while i < HexBoard::NB_SQUARES {
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
impl Default for Hex {
	fn default() -> Self {
		let mut s = Self {
			blue: Default::default(),
			red: Default::default(),
			current_player: Default::default(),
			hash: Default::default(),
		};
		s.hash = s.compute_hash();
		s
	}
}
impl Hex {
	const ZOBRIST: Zobrist = Zobrist::new(1254);
	fn compute_hash(&self) -> u64 {
		let mut h = 0u64;
		for i in self.blue.iter_bits() {
			h ^= Self::ZOBRIST.pieces[i as usize][0];
		}
		for i in self.red.iter_bits() {
			h ^= Self::ZOBRIST.pieces[i as usize][1];
		}
		if self.current_player == Player::PLAYER2 {
			h ^= Self::ZOBRIST.turn;
		}
		h
	}
	pub fn update_turn_hash(&mut self) {
		self.hash ^= Self::ZOBRIST.turn;
	}
	pub fn update_hash_single_move(&mut self, m: &Move, piece_sign: usize) {
		//let opp_sign = 1 - piece_sign;
		self.hash ^= Self::ZOBRIST.pieces[m.to() as usize][piece_sign];
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Move {
	Place(u8),
	Swap,
}
impl Hex {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn current_player(&self) -> Player {
		self.current_player
	}

	pub fn legal_moves_inplace(&self, results: &mut Vec<Move>) {
		if self.current_player == Player::PLAYER2 && self.red.is_empty() {
			results.push(Move::Swap);
		}
		let empty = (self.red | self.blue).flipped();
		for e in empty.iter_bits() {
			results.push(Move::Place(e as u8));
		}
	}

	pub fn play_unchecked(&mut self, mv: Move) {
		match mv {
			Move::Place(to) => match self.current_player {
				Player::PLAYER1 => {
					self.blue.set_at_index(to as usize);
					self.update_hash_single_move(&mv, 0);
				}
				Player::PLAYER2 => {
					self.red.set_at_index(to as usize);
					self.update_hash_single_move(&mv, 1);
				}
				_ => unreachable!(),
			},
			Move::Swap => {
				self.red = self.blue.mirrored_diag();
				self.blue = HexBoard::EMPTY;

				self.hash = self.compute_hash();
			}
		}
		self.update_turn_hash();
		self.switch_player();
	}

	pub fn switch_player(&mut self) {
		self.current_player = self.current_player.opponent();
	}

	pub fn result(&self) -> GameOutcome {
		if self.has_connection(Player::PLAYER1) {
			return GameOutcome::Player(Player::PLAYER1);
		}

		if self.has_connection(Player::PLAYER2) {
			return GameOutcome::Player(Player::PLAYER2);
		}

		GameOutcome::OnGoing
	}

	fn has_connection(&self, player: Player) -> bool {
		let stones = match player {
			Player::PLAYER1 => self.blue,
			Player::PLAYER2 => self.red,
			_ => unreachable!(),
		};

		let mut visited = HexBoard::empty();
		let mut stack = Vec::<usize>::new();

		match player {
			// Blue : Left -> Right
			Player::PLAYER1 => {
				let mut y = 0;

				while y < HexBoard::HEIGHT {
					let idx = HexBoard::index_from_coords(0, y);

					if stones.get_at_index(idx) {
						stack.push(idx);
						visited.set_at_index(idx);
					}

					y += 1;
				}

				while let Some(sq) = stack.pop() {
					let (x, _) = HexBoard::coords_from_index(sq);

					if x == HexBoard::WIDTH - 1 {
						return true;
					}

					let neighbors = crate::bitboard::NEIGHBORS[sq] & stones;

					for n in neighbors.iter_bits() {
						let n = n as usize;

						if !visited.get_at_index(n) {
							visited.set_at_index(n);
							stack.push(n);
						}
					}
				}
			}

			// Red : Bottom -> Top
			Player::PLAYER2 => {
				let mut x = 0;

				while x < HexBoard::WIDTH {
					let idx = HexBoard::index_from_coords(x, 0);

					if stones.get_at_index(idx) {
						stack.push(idx);
						visited.set_at_index(idx);
					}

					x += 1;
				}

				while let Some(sq) = stack.pop() {
					let (_, y) = HexBoard::coords_from_index(sq);

					if y == HexBoard::HEIGHT - 1 {
						return true;
					}

					let neighbors = crate::bitboard::NEIGHBORS[sq] & stones;

					for n in neighbors.iter_bits() {
						let n = n as usize;

						if !visited.get_at_index(n) {
							visited.set_at_index(n);
							stack.push(n);
						}
					}
				}
			}

			_ => unreachable!(),
		}

		false
	}
}
impl core::fmt::Display for Hex {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		writeln!(f, "Current player: {:?}", self.current_player)?;

		write!(f, "   ")?;
		for x in 0..HexBoard::WIDTH {
			write!(f, "{} ", (b'A' + x) as char)?;
		}
		writeln!(f)?;

		for y in 0..HexBoard::HEIGHT {
			write!(f, "{:2} ", y + 1)?;

			for _ in 0..y {
				write!(f, " ")?;
			}

			for x in 0..HexBoard::WIDTH {
				let idx = HexBoard::index_from_coords(x, y);

				let c = if self.blue.get_at_index(idx) {
					'●'
				} else if self.red.get_at_index(idx) {
					'○'
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
#[cfg(test)]
mod tests {
	use kudchuet::{gui::BoardGame, utils::Rng, GameOutcome};

	use crate::rules::Hex;

	#[test]
	fn test_play() {
		let mut hex = Hex::default();
		let mut rnd = Rng::new();
		while hex.result() == GameOutcome::OnGoing {
			let moves = hex.legal_moves();
			let index = rnd.range(0, moves.len());
			hex.play_unchecked(moves[index]);
			println!("{}", hex);
		}
		println!("Winner: {:?}", hex.result())
	}
}
