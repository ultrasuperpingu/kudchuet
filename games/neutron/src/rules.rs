use bitboard::{BitIter, Bitboard};


use std::fmt::{self, Display, Formatter};

use kudchuet::{GameResult, Player};

use crate::pext_tables;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
	pub(crate) pawn: (u8, u8),
	pub(crate) neutron: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Neutron {
	pub white: Bitboard5x5,
	pub black: Bitboard5x5,
	pub neutron: Bitboard5x5,
	pub turn: Player,
	pub move_count: u8,
	pub hash: u64,
}

impl Display for Neutron {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "Tour : {:?}", self.turn)?;

		for y in 0..5 {
			for x in 0..5 {
				
				let c = if self.black.get(x,y) {
					'B'
				} else if self.white.get(x,y) {
					'W'
				} else if self.neutron.get(x, y) {
					'N'
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

impl Neutron {
	pub fn new() -> Self {
		let mut s = Self {
			white:Bitboard5x5::SOUTH_BORDER,
			black:Bitboard5x5::NORTH_BORDER,
			neutron:Bitboard5x5::CENTER,
			turn: Player::PLAYER1,
			move_count: 0,
			hash: 0,
		};
		//s.hash = s.compute_hash();
		s.hash = s.compute_zobrist();
		s
	}
}
impl Default for Neutron {
	fn default() -> Self {
		Self::new()
	}
}

impl Neutron {

	#[inline]
	pub fn occupied(&self) -> Bitboard5x5 {
		self.white | self.black | self.neutron
	}
	pub fn free(&self) -> Bitboard5x5 {
		(!self.occupied()) & Bitboard5x5::FULL
	}
}

#[inline(always)]
fn get_slide_moves(square: u8, blockers: Bitboard5x5) -> Bitboard5x5 {
	let sq = square as usize; 
	let mask = pext_tables::SLIDE_MASKS[sq];
	let moves = pext_tables::SLIDE_MOVES[sq];

	//let index = Bitboard8x8::from_storage(blockers).pext(&Bitboard8x8::from_storage(mask));
	let index = blockers.pext(&mask);

	//Bitboard8x8::from_storage(moves[index as usize])
	moves[index as usize]
}
impl Neutron {
	#[inline]
	pub fn legal_moves(&self) -> Vec<Move> {
		let mut out= vec![];
		self.legal_moves_inplace(&mut out);
		out
	}

	pub fn legal_moves_inplace(&self, out: &mut Vec<Move>) {
		out.clear();

		let my_pawns = match self.turn {
			Player::PLAYER1 => self.white,
			Player::PLAYER2 => self.black,
			_ => unreachable!(),
		};

		let blockers = self.occupied();

		// first move: only pawn
		if self.move_count == 0 {
			for pawn_from in my_pawns.iter_bits() {
				let moves = get_slide_moves(pawn_from as u8, blockers);
				for pawn_to in moves.iter_bits() {
					out.push(Move {
						pawn: (pawn_from as u8, pawn_to as u8),
						neutron: None,
					});
				}
			}
			return;
		}
		let neutron_idx = self.get_neutron_index();
		let neutron_moves = get_slide_moves(self.neutron.lsb() as u8, blockers);
		for neutron_to in neutron_moves.iter_bits() {
			for pawn_from in my_pawns.iter_bits() {
				let mut occ_after = blockers;
				occ_after.reset_at_index(neutron_idx);
				occ_after.set_at_index(neutron_to as usize);
				let moves = get_slide_moves(pawn_from as u8, occ_after);
				for pawn_to in moves.iter_bits() {
					out.push(Move {
						pawn: (pawn_from as u8, pawn_to as u8),
						neutron: Some(neutron_to as u8),
					});
				}
			}
		}

	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Piece {
	Black,
	White,
	Neutron
} 
impl Neutron {
	pub const ZOBRIST_KEYS: Zobrist = Zobrist::new(0x15A4CDE);
	#[inline(always)]
	pub fn get_hash(&self) -> u64 {
		self.hash
	}
	fn compute_zobrist(&self) -> u64 {
		let mut h = 0u64;
		for i in self.white.iter_bits() { h ^= Self::ZOBRIST_KEYS.pieces[i as usize][0]; }
		for i in self.black.iter_bits() { h ^= Self::ZOBRIST_KEYS.pieces[i as usize][1]; }
		h ^= Self::ZOBRIST_KEYS.neutron[self.neutron.lsb() as usize];
		if self.turn == Player::PLAYER2 { h ^= Self::ZOBRIST_KEYS.turn; }
		h
	}

	fn update_hash_pawn_move(&mut self, m: &Move) {
		let p_idx = if self.turn == Player::PLAYER1 { 0 } else { 1 };
		self.hash ^= Self::ZOBRIST_KEYS.pieces[m.pawn.0 as usize][p_idx];
		self.hash ^= Self::ZOBRIST_KEYS.pieces[m.pawn.1 as usize][p_idx];
	}
	fn update_hash_neutron_move(&mut self, old_pos: usize, new_pos: usize) {
		self.hash ^= Self::ZOBRIST_KEYS.neutron[old_pos];
		self.hash ^= Self::ZOBRIST_KEYS.neutron[new_pos];
	}
	fn update_hash_turn(&mut self) {
		self.hash ^= Self::ZOBRIST_KEYS.turn
	}
}
impl Neutron {
	#[inline]
	pub fn piece_at(&self, x: u8, y: u8) -> Option<Piece> {
		if self.neutron.get(x, y) {
			Some(Piece::Neutron)
		} else if self.white.get(x, y) {
			Some(Piece::White)
		} else if self.black.get(x, y) {
			Some(Piece::Black)
		} else {
			None
		}
	}
	#[inline]
	pub fn piece_at_index(&self, index: u8) -> Option<Piece> {
		if self.neutron.get_at_index(index as usize) {
			Some(Piece::Neutron)
		} else if self.white.get_at_index(index as usize) {
			Some(Piece::White)
		} else if self.black.get_at_index(index as usize) {
			Some(Piece::Black)
		} else {
			None
		}
	}
	#[inline(always)]
	pub fn turn(&self) -> Player {
		self.turn
	}
	#[inline(always)]
	pub fn get_neutron(&self) -> (u8, u8) {
		Bitboard5x5::coords_from_index(self.get_neutron_index())
	}
	#[inline(always)]
	pub fn get_neutron_index(&self) -> usize {
		self.neutron.lsb() as usize
	}
	#[inline]
	pub fn play(&mut self, mv: &Move) {
		let bb = match self.turn {
			Player::PLAYER1 => &mut self.white,
			Player::PLAYER2 => &mut self.black,
			_ => unreachable!()
		};
		bb.reset_at_index(mv.pawn.0 as usize);
		bb.set_at_index(mv.pawn.1 as usize);
		
		if let Some(neutron_to) = mv.neutron {
			let old_neutron = self.neutron.storage().trailing_zeros() as usize;
			self.neutron.reset_at_index(old_neutron);
			self.neutron.set_at_index(neutron_to as usize);
			self.update_hash_neutron_move(old_neutron, neutron_to as usize);
			
		}
		self.update_hash_pawn_move(mv);
		self.turn = self.turn.opponent();
		self.move_count+=1;

		self.update_hash_turn();
	}

	#[inline(always)]
	pub fn is_first_move(&self) -> bool {
		self.move_count == 0
	}
	pub fn possible_moves_for(&self, index:usize) -> Vec<u8> {
		let mut pos=vec![];
		let legals = self.legal_moves();
		if index == self.get_neutron_index() {
			for m in legals {
				if let Some(neutron) = m.neutron.as_ref() {
					if !pos.contains(neutron) {
						pos.push(*neutron);
					}
				}
			}
		} else {
			for m in legals {
				if m.pawn.0 as usize == index {
					pos.push(m.pawn.1);
				}
			}
		}
		pos
	} 
}

pub struct Zobrist {
	pub pieces: [[u64; 2]; Bitboard5x5::NB_SQUARES],
	pub neutron: [u64; Bitboard5x5::NB_SQUARES],
	pub turn: u64,
}

impl Zobrist {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);
		let mut pieces = [[0u64; 2]; Bitboard5x5::NB_SQUARES];
		let mut neutron = [0u64; Bitboard5x5::NB_SQUARES];
		
		let mut i = 0;
		while i < Bitboard5x5::NB_SQUARES {
			pieces[i][0] = rng.u64();
			pieces[i][1] = rng.u64();
			neutron[i] = rng.u64();
			i += 1;
		}
		
		Self {
			pieces,
			neutron,
			turn: rng.u64(),
		}
	}
}
impl Neutron {
	
	pub fn result(&self) -> GameResult {
		if !(self.neutron & Bitboard5x5::SOUTH_BORDER).is_empty() {
			return GameResult::Player1;
		}

		if !(self.neutron & Bitboard5x5::NORTH_BORDER).is_empty() {
			return GameResult::Player2;
		}

		let mut moves = Vec::new();
		self.legal_moves_inplace(&mut moves);

		if moves.is_empty() {
			match self.turn {
				Player::PLAYER1 => GameResult::Player1,
				Player::PLAYER2 => GameResult::Player2,
				_ => unreachable!()
			}
		} else {
			GameResult::OnGoing
		}
	}

}
impl Neutron {
	pub fn to_fen(&self) -> String {
		let mut fen = String::new();
		for y in 0..5 {
			for x in 0..5 {
				let c = match self.piece_at(x, y) {
					Some(Piece::White) => 'W',
					Some(Piece::Black) => 'B',
					Some(Piece::Neutron) => 'N',
					None => '.',
				};
				fen.push(c);
			}
			if y != 4 {
				fen.push('/');
			}
		}
		fen.push(' ');
		fen.push(match self.turn {
			Player::PLAYER1 => '1',
			Player::PLAYER2 => '2',
			_ => unreachable!(),
		});
		fen
	}

	pub fn from_fen(fen: &str) -> Result<Self, String> {
		let mut parts = fen.split(' ');
		let board_part = parts.next().ok_or("Empty FEN or invalid format")?;
		let turn_part = parts.next().ok_or("FEN missing player part")?;

		let mut white = Bitboard5x5::empty();
		let mut black = Bitboard5x5::empty();
		let mut neutron = Bitboard5x5::empty();

		let rows: Vec<&str> = board_part.split('/').collect();
		if rows.len() != 5 {
			return Err(format!("FEN must have 5 rows, found {}", rows.len()));
		}

		for (y, row) in rows.iter().enumerate() {
			if row.chars().count() != 5 {
				return Err(format!("Row {} must have 5 columns, found {}", y, row.chars().count()));
			}
			for (x, c) in row.chars().enumerate() {
				let idx = y * 5 + x;
				match c {
					'W' => white.set_at_index(idx),
					'B' => black.set_at_index(idx),
					'N' => {
						if !neutron.is_empty() {
							return Err("Multiple neutrons found".into());
						}
						neutron.set_at_index(idx);
					}
					'.' => {}
					_ => return Err(format!("Invalid character '{}' in FEN", c)),
				}
			}
		}

		if neutron.is_empty() {
			return Err("No neutron found in FEN".into());
		}

		let turn = match turn_part {
			"1" => Player::PLAYER1,
			"2" => Player::PLAYER2,
			_ => return Err(format!("Invalid player '{}' in FEN", turn_part)),
		};

		let mut state = Self {
			white,
			black,
			neutron,
			turn,
			move_count: 0,
			hash: 0,
		};
		state.hash = state.compute_zobrist();
		Ok(state)
	}
}

#[cfg(test)]
mod tests {
	use super::Neutron;

	#[test]
	fn test_play() {
		let mut neutron = Neutron::new();
		println!("{}", neutron);
		let mut legal_moves = neutron.legal_moves();
		let mut i=0;
		while !legal_moves.is_empty() && i<100 {
			println!("{:?}", legal_moves);
			neutron.play(&legal_moves[0]);
			println!("{}", neutron);
			assert_eq!(neutron.compute_zobrist(), neutron.get_hash());
			legal_moves = neutron.legal_moves();
			i+=1;
		}
	}
	#[test]
	fn test_fen_round_trip() {
		// Création du plateau initial
		let neutron = Neutron::new();

		// Sérialisation en FEN
		let fen = neutron.to_fen();
		println!("FEN initial: {}", fen);

		// Désérialisation depuis le FEN
		let loaded = Neutron::from_fen(&fen).expect("Impossible de parser la FEN");

		// Vérification que tout correspond
		assert_eq!(neutron.white, loaded.white, "Blancs différents");
		assert_eq!(neutron.black, loaded.black, "Noirs différents");
		assert_eq!(neutron.neutron, loaded.neutron, "Neutron différent");
		assert_eq!(neutron.turn, loaded.turn, "Tour différent");
		assert_eq!(neutron.hash, loaded.hash, "Hash différent");

		// Vérification que la FEN round-trip est identique
		let fen2 = loaded.to_fen();
		assert_eq!(fen, fen2, "FEN round-trip différente");
	}
}