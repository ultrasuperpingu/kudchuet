use bitboard::{BitIter, Bitboard};

pub mod ihm;
pub mod gui;
pub mod game;
use std::fmt::{self, Display, Formatter};

use crate::common::{GameResult, Player, bitboards::Bitboard6x5};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
	Add{index:u8},
	Move{
		from:u8,
		to:u8
	},
	Take{
		from:u8,
		to:u8,
		supplement_pawn:Option<u8>
	}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct YoteRules(u8);
impl YoteRules {
	pub fn allow_reserve_additional_take(&self) -> bool {
		self.0 &1 != 0
	}
	pub fn mandatory_takes(&self) -> bool {
		self.0 &2 != 0
	}
	//pub allow_multiple_takes(&self) -> bool {self.0 &4 != 0}
	pub fn with_allow_reserve_additional_take(mut self) -> Self {
		self.0 |= 1;
		self
	}
	pub fn with_mandatory_takes(mut self) -> Self {
		self.0 |= 2;
		self
	}
	//pub with_allow_multiple_takes(&self) -> Self {self.0 |= 2;self}
	pub fn set_allow_reserve_additional_take(&mut self, v:bool) {
		if !v {
			self.0 &= !1;
		} else {
			self.0 |= 1;
		}
	}
	pub fn set_mandatory_takes(&mut self, v:bool) {
		if !v {
			self.0 &= !2;
		} else {
			self.0 |= 2;
		}
	}
	//pub allow_multiple_takes(&self) -> bool {self.0 &4 != 0}
	
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Yote {
	pub white: Bitboard6x5,
	pub black: Bitboard6x5,
	pub reserve_white: u8,
	pub reserve_black: u8,
	pub turn: Player,
	pub hash: u64,
	pub rules: YoteRules,
}

impl Display for Yote {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		writeln!(f, "Turn: {:?}", self.turn)?;

		for y in 0..5 {
			for x in 0..6 {
				
				let c = if self.black.get(x,y) {
					'B'
				} else if self.white.get(x,y) {
					'W'
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

impl Yote {
	pub fn new() -> Self {
		let mut s = Self {
			white:Bitboard6x5::empty(),
			black:Bitboard6x5::empty(),
			reserve_black:12,
			reserve_white:12,
			turn: Player::Player1,
			hash: 0,
			rules: YoteRules::default()
		};
		s.hash = s.compute_hash();
		s
	}
}
impl Default for Yote {
	fn default() -> Self {
		Self::new()
	}
}
impl Yote {

	#[inline]
	pub fn occupied(&self) -> Bitboard6x5 {
		self.white | self.black
	}
	#[inline]
	pub fn free(&self) -> Bitboard6x5 {
		(!(self.white | self.black)) & Bitboard6x5::FULL
	}
}
impl Yote {
	#[inline]
	pub fn legal_moves(&self) -> Vec<Move> {
		let mut out= vec![];
		self.legal_moves_inplace(&mut out);
		out
	}
	pub fn legal_moves_inplace(&self, out: &mut Vec<Move>) {
		out.clear();
		
		let (myself, other, other_reserve) = match self.turn {
			Player::Player1 => {
				(self.white, self.black, self.reserve_black)
			},
			Player::Player2 => {
				(self.black, self.white, self.reserve_white)
			},
			_ => unreachable!()
		};
		for pawn in myself.iter_bits() {
			let pawn = pawn as u8;
			let possible_takes=other.neighbors_ortho(pawn as usize);
			//println!("{}", possible_takes);
			for m in possible_takes.iter_bits() {
				let dest=next_cell(pawn, m as u8);
				if let Some(dest) = dest {
					if self.free().get_at_index(dest as usize) {
						for additional in other.iter_bits() {
							if additional != m {
								out.push(Move::Take { from: pawn, to: dest, supplement_pawn:Some(additional as u8) });
							}
						}
						if self.rules.allow_reserve_additional_take() {
							if other_reserve > 0 {
								out.push(Move::Take { from: pawn, to: dest, supplement_pawn:Some(30) });
							}
						}
						out.push(Move::Take { from: pawn, to: dest, supplement_pawn:None });
					}
				}
			}
			if self.rules.mandatory_takes() && !out.is_empty() {
				continue;
			}
			let moves=(self.free()).neighbors_ortho(pawn as usize);
			for m in moves.iter_bits() {
				out.push(Move::Move { from: pawn, to: m as u8 });
			}
		}
		if self.rules.mandatory_takes() {
			let has_take = out.iter().any(|mv| matches!(mv, Move::Take { .. }));
			if has_take {
				out.retain(|mv| matches!(mv, Move::Take { .. }));
			}
		}

		if !self.rules.mandatory_takes() || out.is_empty() {
			if self.turn == Player::Player1 && self.reserve_white > 0 || self.turn == Player::Player2 && self.reserve_black > 0 {
				for index in (self.free()).iter_bits() {
					out.push(Move::Add { index: index as u8 });
				}
			}
		}
	}

}

impl Yote {
	pub fn compute_hash(&self) -> u64 {
		let mut hash = 0;
		for i in 0..30 {      // loop over the board positions
			if self.white.get_at_index(i) {
				hash ^= HASHES[i+1];
			}
			else if self.black.get_at_index(i) {
				hash ^= HASHES[i+31];
			}
		}
		hash ^= HASHES[61 + self.reserve_white as usize];
		hash ^= HASHES[74 + self.reserve_black as usize];
		if self.turn == Player::Player1 {
			hash ^= HASHES[0];
		}
		hash
	}
	#[inline]
	pub fn get_hash(&self) -> u64 {
		self.hash
	}
}
fn middle_cell(from: usize, to: usize) -> usize {
	let diff = from.abs_diff(to);

	if diff == 2 && from / 6 == to / 6 {
		(from + to) / 2
	} else if diff == 12 {
		from.min(to) + 6
	} else {
		panic!("invalid cells")
	}
}
fn next_cell(from: u8, to: u8) -> Option<u8> {
	let diff = to as isize - from as isize;

	// même rangée pour les mouvements horizontaux
	let same_row = from / 6 == to / 6;

	if diff == 1 && same_row {
		(from + 2 < 30 && from / 6 == (from + 2) / 6).then(|| from + 2)
	} else if diff == -1 && same_row {
		(from >= 2 && from / 6 == (from - 2) / 6).then(|| from - 2)
	} else if diff == 6 {
		(from + 12 < 30).then(|| from + 12)
	} else if diff == -6 {
		(from >= 12).then(|| from - 12)
	} else {
		None
	}
}

impl Yote {
	pub fn play(&mut self, mv: Move) {
		match mv {
			Move::Add { index } => 
			{
				let index = index as usize;
				if self.turn == Player::Player1 {
					self.white.set_at_index(index);
					self.reserve_white-=1;
					// remove turn mask
					self.hash ^= HASHES[0];
					// add white
					self.hash ^= HASHES[index+1];

					// enlever ancien hash
					self.hash ^= HASHES[61 + (self.reserve_white + 1) as usize];
					self.hash ^= HASHES[61 + self.reserve_white as usize];

				} else {
					self.black.set_at_index(index);
					self.reserve_black-=1;
					// add black
					self.hash ^= HASHES[index+31];
					// reserve
					self.hash ^= HASHES[74 + (self.reserve_black + 1) as usize];
					self.hash ^= HASHES[74 + self.reserve_black as usize];

					// add turn mask
					self.hash ^= HASHES[0];
				}
			},
			Move::Move { from, to } => 
			{
				let from = from as usize;
				let to = to as usize;
				if self.turn == Player::Player1 {
					self.white.reset_at_index(from);
					self.white.set_at_index(to);
					// remove turn mask
					self.hash ^= HASHES[0];
					// remove white from
					self.hash ^= HASHES[from+1];
					// add white to
					self.hash ^= HASHES[to+1];
				} else {
					self.black.reset_at_index(from);
					self.black.set_at_index(to);
					// remove black from
					self.hash ^= HASHES[from+31];
					// add black to
					self.hash ^= HASHES[to+31];
					// add turn mask
					self.hash ^= HASHES[0];
				}
			},
			Move::Take { from, to, supplement_pawn } => {
				let from = from as usize;
				let to = to as usize;
				let supplement_pawn = if let Some(val) = supplement_pawn {
					val as isize
				} else {
					-1
				};
				if self.turn == Player::Player1 {
					self.white.reset_at_index(from);
					self.white.set_at_index(to);
					self.black.reset_at_index(middle_cell(from,to));
					// remove turn mask
					self.hash ^= HASHES[0];
					// remove white from
					self.hash ^= HASHES[from+1];
					// add white to
					self.hash ^= HASHES[to+1];
					// remove black from
					self.hash ^= HASHES[middle_cell(from,to)+31];
					if supplement_pawn >= 30 {
						self.reserve_black-=1;
						// reserve
						self.hash ^= HASHES[74 + (self.reserve_black + 1) as usize];
						self.hash ^= HASHES[74 + self.reserve_black as usize];

					} else if supplement_pawn >= 0 {
						self.black.reset_at_index(supplement_pawn as usize);
						// remove black from
						self.hash ^= HASHES[supplement_pawn as usize+31];
					}
				} else {
					self.black.reset_at_index(from);
					self.black.set_at_index(to);
					self.white.reset_at_index(middle_cell(from,to));
					// remove black from
					self.hash ^= HASHES[from+31];
					// add black to
					self.hash ^= HASHES[to+31];
					// remove white from
					self.hash ^= HASHES[middle_cell(from,to)+1];
					if supplement_pawn >= 30 {
						self.reserve_white-=1;
						// reserve
						self.hash ^= HASHES[61 + (self.reserve_white + 1) as usize];
						self.hash ^= HASHES[61 + self.reserve_white as usize];

					} else if supplement_pawn >= 0 {
						self.white.reset_at_index(supplement_pawn as usize);
						// remove white from
						self.hash ^= HASHES[supplement_pawn as usize+1];
					}
					// add turn mask
					self.hash ^= HASHES[0];
				}
			},
		}
		self.turn = self.turn.opponent();
	}
	

}


impl Yote {
	#[inline]
	pub fn white_pawns_count(&self) -> usize {
		self.reserve_white as usize + self.white.count() as usize
	}
	#[inline]
	pub fn black_pawns_count(&self) -> usize {
		self.reserve_black as usize + self.black.count() as usize
	}
	pub fn result(&self) -> GameResult {
		if self.white_pawns_count() <= 3 && self.black_pawns_count() <= 3 {
			return GameResult::Draw;
		}

		if self.white_pawns_count() == 0 {
			return GameResult::Player2;
		}
		if self.black_pawns_count() == 0 {
			return GameResult::Player1;
		}

		GameResult::OnGoing
	}
}
#[test]
fn test_add_moves() {
	let mut g = Yote::new();

	// Blanc commence
	let legal = g.legal_moves();
	assert!(legal.iter().any(|m| matches!(m, Move::Add { .. })));
	assert_eq!(legal.len(), 30); // plateau vide

	// Joue un coup Add
	g.play(Move::Add { index: 0 });
	assert!(g.white.get_at_index(0));
	assert_eq!(g.reserve_white, 11);
	assert_eq!(g.turn, Player::Player2);
}
#[test]
fn test_simple_move() {
	let mut g = Yote::new();

	g.play(Move::Add { index: 0 }); // blanc
	g.play(Move::Add { index: 6 }); // noir

	// Blanc peut bouger 0 → 1 ou 0 → 6 (si libre)
	let legal = g.legal_moves();
	assert!(legal.iter().any(|m| matches!(m, Move::Move { from: 0, to: 1 })));
}
#[test]
fn test_simple_take() {
	let mut g = Yote::new();

	// Blanc en 0, Noir en 1
	g.play(Move::Add { index: 0 });
	g.play(Move::Add { index: 1 });

	// Blanc peut capturer 1 → atterrir en 2
	let legal = g.legal_moves();
	assert!(legal.iter().any(|m| matches!(m, Move::Take { from: 0, to: 2, .. })));

	// Joue la capture
	let mv = legal.into_iter().find(|m| matches!(m, Move::Take { from: 0, to: 2, .. })).unwrap();
	g.play(mv);

	assert!(g.white.get_at_index(2));
	assert!(!g.black.get_at_index(1));
}
#[test]
fn test_take_with_supplement() {
	let mut g = Yote::new();

	// Blanc en 0, Noir en 1 et 5
	g.play(Move::Add { index: 0 });
	g.play(Move::Add { index: 1 });
	g.play(Move::Add { index: 5 }); // noir

	// Blanc capture 1 → 2 et peut retirer 5
	let legal = g.legal_moves();
	let mv = legal.into_iter().find(|m| matches!(
		m,
		Move::Take { from: 0, to: 2, supplement_pawn: Some(5) }
	)).unwrap();

	g.play(mv);

	assert!(g.white.get_at_index(2));
	assert!(!g.black.get_at_index(1));
	assert!(!g.black.get_at_index(5));
}
#[test]
fn test_draw_condition() {
	let mut g = Yote::new();

	g.reserve_white = 0;
	g.reserve_black = 0;

	// Place 3 pions blancs et 3 noirs
	g.white = Bitboard6x5::from_storage(0b111);
	g.black = Bitboard6x5::from_storage(0b111 << 3);

	assert_eq!(g.result(), GameResult::Draw);
}
#[test]
fn test_white_win() {
	let mut g = Yote::new();

	g.white = Bitboard6x5::from_storage(1);
	g.black = Bitboard6x5::empty();
	g.reserve_black = 0;

	assert_eq!(g.result(), GameResult::Player1);
}
#[test]
fn test_hash_consistency() {
	let mut g = Yote::new();
	let h0 = g.hash;

	g.play(Move::Add { index: 0 });
	let h1 = g.hash;

	assert_ne!(h0, h1);

	// Rejouer la même position doit donner le même hash
	let mut g2 = Yote::new();
	g2.play(Move::Add { index: 0 });

	assert_eq!(h1, g2.hash);
	assert_eq!(g2.compute_hash(), g2.hash);
}
#[test]
fn test_hash_perft() {
	fn recurse(g: &mut Yote, depth: u32, initial_hash: u64) {
		if depth == 0 {
			// Le hash doit être identique à celui de départ
			assert_eq!(g.hash, initial_hash);
			return;
		}

		let moves = g.legal_moves();

		for mv in moves {
			let h_before = g.hash;
			let saved = g.clone();
			g.play(mv);

			// Vérifier que le hash incrémental == hash recomputé
			assert_eq!(g.hash, g.compute_hash());

			recurse(g, depth - 1, g.hash);

			*g = saved;

			// Après undo, le hash doit revenir exactement à h_before
			assert_eq!(g.hash, h_before);
		}
	}

	let mut g = Yote::new();
	let h0 = g.hash;

	recurse(&mut g, 4, h0); // depth 3 suffit pour détecter 99% des bugs
}

#[test]
fn test_add_generation() {
	let g = Yote::new();
	let legal = g.legal_moves();

	assert_eq!(legal.len(), 30);
	assert!(legal.iter().all(|m| matches!(m, Move::Add { .. })));
}
#[test]
fn test_move_blocked() {
	let mut g = Yote::new();

	g.play(Move::Add { index: 0 });
	g.play(Move::Add { index: 1 });

	let legal = g.legal_moves();
	assert!(!legal.iter().any(|m| matches!(m, Move::Move { from: 0, to: 1 })));
}
#[test]
fn test_play() {
	let mut g = Yote::new();
	println!("{}", g);
	while g.result() == GameResult::OnGoing {
		let mvs=g.legal_moves();
		g.play(mvs[0]);
		println!("{}", g);
	}
}

const HASHES: [u64; 88] = [
	0x73399349585d196e,
	0xe512dc15f0da3dd1,
	0x4fbc1b81c6197db2,
	0x16b5034810111a66,
	0xa9a9d0183e33c311,
	0xbb9d7bdea0dad2d6,
	0x089d9205c11ca5c7,
	0x18d9db91aa689617,
	0x1336123120681e34,
	0xc902e6c0bd6ef6bf,
	0x16985ba0916238c1,
	0x6144c3f2ab9f6dc4,
	0xf24b4842de919a02,
	0xdd6dd35ba0c150a1,
	0x369a9de8ec3676e3,
	0x2c366fb99be782d8,
	0x24d3231335c0dbd6,
	0x14048390c56e38f1,
	0x55dfbc820f635186,
	0x0dc98cb87372d5fa,
	0xe3098781582027b4,
	0x088158ec8202adca,
	0x231df62376ad9514,
	0xd3747fad069caeae,
	0x4e4f26cb41d0c620,
	0x06d0e37cd11b8f1c,
	0xed33865175fbbdd2,
	0xf1f52569481f0d8f,
	0xfb6fd5c922e2127c,
	0x6778bb0eba4a6649,
	0xe35b853bdac1210b,
	0x465a67712ec749a2,
	0x83b1fd78e576fe72,
	0xe84827644a5ccbe6,
	0x89095321ce8e4d03,
	0x298c529eecb0ec36,
	0xe9dcc93d77cb49ad,
	0xa7446daa1834c04a,
	0x93f15442b434d550,
	0x7f2a36dbf1cbce3f,
	0x03365a42023b02b3,
	0x101d87e850689cda,
	0x113b31e2760d2050,
	0x9cdb7b7394e1b0ae,
	0xd04530b3b7daf3a3,
	0x717e67aed6b4ffc9,
	0x4ae564a3f3ca8b03,
	0x07c50a4d89351437,
	0x7f3b32175e5f37e0,
	0x6e3599203bb50cd7,
	0xcfe2319d4a6cfa73,
	0xdbc6a398b10f5c3b,
	0x9c1ba28ae655bbd1,
	0x9dc87a426451941a,
	0x691e618354a55cb5,
	0x61b8cabbc575f4ba,
	0x7e6f31f1818593d4,
	0x9fa69e1ef4df8a9b,
	0x5a9dc96c3cb18d8f,
	0x65c4e9c0f40114f5,
	0x4e66504db2d937cf,
	0x4ebd6d097fe1e256,
	0xfb10983e639af6b1,
	0xcfbed7bd4032a59a,
	0x1f47f6a95049fe4f,
	0xbd461d202b879890,
	0xfc050073b0c74cbe,
	0x2923526a1f7092e9,
	0x0b1d30bb6b960bc7,
	0x632d12e4a9d0229d,
	0x8d4ffd6ab37c6bfd,
	0x561e36b8609b94ec,
	0x32e8482c9e7ed80c,
	0xaf62a119227b1029,
	0x62cb2a585410c311,
	0x7df3aeef90e1a0cb,
	0xe6d5a176f8a1b180,
	0x156e5162d8f2bef8,
	0xee84c58f5ebbe811,
	0xd32a1b4e24038bac,
	0xeaa1dbdbdd7731f7,
	0xedb554afd3d07cc6,
	0xbc789444317d4d05,
	0x0e23ce8f3d581fcd,
	0xacb498d4569249a8,
	0x843fb2519edc9f5a,
	0xe222f0eb79436809,
	0x7a88365f089ae80b,
	//0x2a0f08694d7ea84d,
	//0x09cad4dbfc990fa2,
	//0xfe5f27499de6b4f8,
	//0x3d8ed8ab1d44997f,
	//0x2af64deca431f644,
	//0xf2712b5274180c36,
	//0x30eeae3a821bf86c,
	//0x31c921831f06ad2f,
	//0x40683ff11655cd2f,
	//0xb78183a74cd6cb03,
	//0xde9e15a6f99bda2f,
	//0xa5293988641edb9b,
];
