use bitboard::{BitIter, Bitboard};
use bitboard_proc_macro::bitboard;
use kudchuet::{GameOutcome, Player, utils::splitmix64};
#[bitboard(width = 5, height = 6)]
#[derive(Debug, Hash)]
pub(crate) struct Bitboard6x5;
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Clobber {
	pub(crate) white: Bitboard6x5,
	pub(crate) black: Bitboard6x5,
	pub(crate) is_black: bool,
}
impl Default for Clobber {
	fn default() -> Self {
		Self {
			white: Bitboard6x5::EVEN_SQUARES,
			black: Bitboard6x5::ODD_SQUARES,
			is_black: false,
		}
	}
}
static NEIGHBORS: [Bitboard6x5; Bitboard6x5::NB_SQUARES] =
	Bitboard6x5::generate_neighbors_ortho_table();
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
	pub from: u8,
	pub to: u8,
}
impl Clobber {
	pub fn legal_moves(&self) -> Vec<Move> {
		let mut moves = vec![];
		let (mine, other) = &mut if self.is_black {
			(self.black, self.white)
		} else {
			(self.white, self.black)
		};
		for p in mine.iter_bits() {
			for nei in NEIGHBORS[p as usize].and_const(other).iter_bits() {
				moves.push(Move {
					from: p as u8,
					to: nei as u8,
				});
			}
		}
		moves
	}
	pub fn play_unchecked(&mut self, m: Move) {
		if self.is_black {
			self.black.reset_at_index(m.from as usize);
			self.black.set_at_index(m.to as usize);
			self.white.reset_at_index(m.to as usize);
		} else {
			self.white.reset_at_index(m.from as usize);
			self.white.set_at_index(m.to as usize);
			self.black.reset_at_index(m.to as usize);
		}
		self.is_black = !self.is_black;
	}
	pub fn result(&self) -> GameOutcome {
		let legal_moves = self.legal_moves();
		if legal_moves.is_empty() {
			GameOutcome::Player(if self.is_black {
				Player::PLAYER1
			} else {
				Player::PLAYER2
			})
		} else {
			GameOutcome::OnGoing
		}
	}
	pub fn get_hash(&self) -> u64 {
		let mut h = 0u64;
		h ^= splitmix64(self.white.storage() as u64);
		h ^= splitmix64((self.black.storage()as u64).rotate_left(17));
		if self.is_black {
			h ^= 0x9E3D70B97F4A7C14;
		}
		h
	}
}
