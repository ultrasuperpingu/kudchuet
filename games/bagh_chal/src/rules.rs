

use bitboard::{BitIter, Bitboard};

use kudchuet::GameResult;

use crate::bitboard::Bitboard5x5;
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Move {
	pub from:Option<u8>,
	pub to:u8,
	pub captured:Option<u8>
}
impl Move {
	pub fn mov(from:u8, to:u8) -> Move {
		Move { from: Some(from), to, captured:None }
	}
	pub fn drop(to:u8) -> Move {
		Move { from: None, to, captured:None }
	}
	pub fn capture(from: u8, to:u8, capture: u8) -> Move {
		Move { from: Some(from), to, captured:Some(capture) }
	}
}

#[derive(Clone,Copy)]
pub struct CaptureList {
	pub moves: [(u8, u8); 8],
	pub len: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct BaghChal {
	pub(crate) goats: Bitboard5x5,
	pub(crate) tigers: Bitboard5x5,
	
	pub(crate) goats_deposited: u8,
	pub(crate) goats_captured: u8,
	pub(crate) turn_tiger: bool,
	pub(crate) history: Vec<u64>
}
impl Default for BaghChal {
	fn default() -> Self {
		Self {
			goats: Bitboard5x5::empty(),
			tigers: Bitboard5x5::CORNERS,
			goats_deposited: 0,
			goats_captured: 0,
			turn_tiger: false,
			history: Vec::new(),
		}
	}
}
impl BaghChal {
	pub fn all(&self) -> Bitboard5x5 {
		self.goats | self.tigers
	}
	pub fn free(&self) -> Bitboard5x5 {
		(!self.all()) & Bitboard5x5::FULL
	}
	pub fn tigers_turn(&self) -> bool {
		self.turn_tiger
	}
}
impl BaghChal {
	pub fn get_hash(&self) -> u64 {
		let mut key = 0u64;
		key |= self.tigers.storage() as u64;           // 25 bits
		key |= (self.goats.storage() as u64) << 25;   // 25 bits
		key |= (self.turn_tiger as u64) << 50;           // 1 bit
		key |= (self.goats_deposited as u64) << 51; // 5 bits
		key |= (self.goats_captured as u64) << 56; // 3 bits
		kudchuet::utils::fibo_hash_64(key)
	}
}
impl BaghChal {
	pub fn play_unchecked(&mut self, m: &Move) {
		if self.tigers_turn() {
			self.tigers.reset_at_index(m.from.unwrap() as usize);
			self.tigers.set_at_index(m.to as usize);
			if let Some(cap) = m.captured {
				self.goats.reset_at_index(cap as usize);
				self.goats_captured += 1;
			}
		}
		else
		{
			if let Some(from) = m.from {
				self.goats.reset_at_index(from as usize);
				self.goats.set_at_index(m.to as usize);
			} else {
				self.goats.set_at_index(m.to as usize);
				self.goats_deposited +=1;
			}
		}
		self.turn_tiger = !self.turn_tiger;
		self.history.push(self.get_hash());
	}
	pub fn undo_unchecked(&mut self, m: &Move) {
		self.history.pop();
		self.turn_tiger = !self.turn_tiger;
		if self.tigers_turn() {
			self.tigers.set_at_index(m.from.unwrap() as usize);
			self.tigers.reset_at_index(m.to as usize);
			if let Some(cap) = m.captured {
				self.goats.set_at_index(cap as usize);
				self.goats_captured -= 1;
			}
		}
		else
		{
			if let Some(from) = m.from {
				self.goats.set_at_index(from as usize);
				self.goats.reset_at_index(m.to as usize);
			} else {
				self.goats.reset_at_index(m.to as usize);
				self.goats_deposited -=1;
			}
		}
	}
	pub fn has_legal_moves(&self) -> bool {
		let free = self.free();
		if self.tigers_turn() {
			for tiger in self.tigers.iter_bits() {
				if !(Bitboard5x5::NEIGHBORS_BAGH_CHAL[tiger as usize] & free).is_empty() {
					return true;
				}
				let captures = &Bitboard5x5::TIGER_CAPTURES[tiger as usize];
				for i in 0..captures.len {
					let (mid, to) = captures.moves[i as usize];
					if self.goats.get_at_index(mid as usize) && free.get_at_index(to as usize) {
						return true;
					}
				}
			}
		} else {
			if self.goats_deposited < 20 {
				return !free.is_empty();
			} else {
				for goat in self.goats.iter_bits() {
					if !(Bitboard5x5::NEIGHBORS_BAGH_CHAL[goat as usize] & free).is_empty() {
						return true;
					}
				}
			}
		}
		false
	}
	pub fn legal_moves_inplace(&self, moves: &mut Vec<Move>) {
		moves.clear();
		let free = self.free();
		if self.tigers_turn() {
			for tiger in self.tigers.iter_bits() {
				for to in (free & Bitboard5x5::NEIGHBORS_BAGH_CHAL[tiger as usize]).iter_bits() {
					moves.push(Move::mov(tiger as u8, to as u8));
				}
				let captures = Bitboard5x5::TIGER_CAPTURES[tiger as usize];

				let mut i = 0;
				while i < captures.len {
					let (mid, to) = captures.moves[i as usize];

					if self.goats.get_at_index(mid as usize)
						&& free.get_at_index(to as usize)
					{
						moves.push(Move::capture(tiger as u8, to, mid));
					}

					i += 1;
				}
			}
			
		} else {
			if self.goats_deposited < 20 {
				for to in free.iter_bits() {
					moves.push(Move::drop(to as u8));
				}
			} else {
				for goat in self.goats.iter_bits() {
					for to in (free & Bitboard5x5::NEIGHBORS_BAGH_CHAL[goat as usize]).iter_bits() {
						moves.push(Move::mov(goat as u8, to as u8));
					}
				}
			}
		}
	}
	pub fn result(&self) -> GameResult {
		if self.goats_captured >= 5 {
			return GameResult::PLAYER2;
		}

		if self.tigers_turn() {
			let mut tiger_moves = Vec::new();
			self.legal_moves_inplace(&mut tiger_moves);
			if tiger_moves.is_empty() {
				return GameResult::PLAYER1;
			}
		}
 		let key = self.get_hash();
		if self.history.iter().filter(|h| **h == key).count() >= 3 {
			return GameResult::PLAYER2; // tigers win
		}
		GameResult::OnGoing
	}
}