use std::fmt::{self, Debug};

use bitboard::{BitIter, Bitboard};
use kudchuet::{GameOutcome, Player, gui::GUIGame};

use crate::bitboard::SolitaireBoard;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Solitaire {
	pub(crate) board: SolitaireBoard,
	hash: u64,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Move {
	pub from: u8,
	pub to: u8,
}
impl Solitaire {
	pub fn new() -> Self {
		let mut s = Self {
			board: SolitaireBoard::initial_state(),
			hash: 0,
		};
		s.hash = s.compute_hash();
		s
	}
	pub fn is_valid_cell(x: u8, y: u8) -> bool {
		SolitaireBoard::MASK.get(x, y)
	}
}
impl Solitaire {
	pub fn legal_moves_inplace(&self, moves: &mut Vec<Move>) {
		for from in self.board.iter_bits() {
			for to in SolitaireBoard::JUMPS[from as usize].iter_bits() {
				if !self.board.get_at_index(to as usize)
					&& self.board.get_at_index((from as usize + to as usize) / 2)
				{
					moves.push(Move {
						from: from as u8,
						to: to as u8,
					});
				}
			}
		}
	}
	pub fn play_unchecked(&mut self, m: Move) {
		self.board.reset_at_index(m.from as usize);
		self.board
			.reset_at_index((m.from as usize + m.to as usize) / 2);
		self.board.set_at_index(m.to as usize);
		self.update_hash_move(&m);
	}
	pub fn result(&self) -> GameOutcome {
		let legal_moves = self.legal_moves();
		if legal_moves.is_empty() {
			GameOutcome::Player(if self.board.count() == 1 {
				Player::PLAYER1
			} else {
				Player::PLAYER2
			})
		} else {
			GameOutcome::OnGoing
		}
	}
}
impl Default for Solitaire {
	fn default() -> Self {
		Self::new()
	}
}

impl fmt::Display for Solitaire {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.board)
	}
}

impl Solitaire {
	pub const ZOBRIST_KEYS: Zobrist = Zobrist::new(0x15A4CDE);
	#[inline(always)]
	pub fn get_hash(&self) -> u64 {
		self.hash
	}
	pub const fn compute_hash(&self) -> u64 {
		let mut h = 0u64;

		let mut i = 0;
		while i < SolitaireBoard::NB_SQUARES {
			if self.board.get_at_index(i) {
				h ^= Self::ZOBRIST_KEYS.content[i];
			}
			i += 1;
		}

		h
	}

	fn update_hash_move(&mut self, m: &Move) {
		let mid = (m.from as usize + m.to as usize) / 2;

		self.hash ^= Self::ZOBRIST_KEYS.content[m.from as usize];
		self.hash ^= Self::ZOBRIST_KEYS.content[mid];
		self.hash ^= Self::ZOBRIST_KEYS.content[m.to as usize];
	}
}
pub struct Zobrist {
	pub content: [u64; SolitaireBoard::NB_SQUARES],
}
impl Zobrist {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);
		let mut content = [0; SolitaireBoard::NB_SQUARES];

		let mut i = 0;
		while i < SolitaireBoard::NB_SQUARES {
			content[i] = rng.u64();
			i += 1;
		}

		Self { content }
	}
}
#[cfg(test)]
mod tests {
	use crate::rules::Solitaire;
	use kudchuet::ai::move_search::Strategy;
	use kudchuet::ai::move_search::astar::AStar;
	use kudchuet::gui::GUIGame as _;

	#[test]
	fn display() {
		let solitaire = Solitaire::new();
		println!("{}\n\n", solitaire);
		let moves = solitaire.legal_moves();
		println!("{:?}", moves);
	}

	#[test]
	fn test_legal_moves() {
		let mut state = Solitaire::new();
		println!("{}", state);
		let mut moves = state.legal_moves();
		println!("{:?}", moves);
		let mut i = 0;
		while i < 500000 && !moves.is_empty() {
			let mv = fastrand::choice(&moves).unwrap();

			state.play_unchecked(*mv);
			assert_eq!(state.compute_hash(), state.hash);
			moves = state.legal_moves();
			i += 1;
		}

		println!("{}", i);
		println!("{:?}", state.result());
		println!("{}", state);
		println!("{:?}", moves);
	}
	#[test]
	fn test_play() {
		let mut state = Solitaire::new();
		println!("{}", state);
		let mut i = 0;
		let mut strategy = AStar::<Solitaire>::default();
		//strategy.set_depth_or_timeout(12, Duration::from_secs(1));
		while i < 500000 {
			let mv = strategy.choose_move(&mut state);
			assert_eq!(state.compute_hash(), state.get_hash());
			if let Some(mv) = mv {
				state.play(mv);
				println!("{}", state);
			} else {
				break;
			}
			i += 1;
		}

		println!("{}", i);
		println!("{:?}", state.result());
	}
}
