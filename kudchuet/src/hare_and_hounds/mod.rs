use bitboard_proc_macro::{BitboardDebug, bitboard};
use bitboard::{BitIter, Bitboard};

use crate::common::GameResult;
pub mod game;
pub mod gui;
// 0 3-6-9  12
//  /|\|\| \
// 1-4-7-10-13
//  \|/|\| /
// 2 5-8-11 14
#[bitboard(width=5,height=3,col_major=true)]
#[derive(Hash, BitboardDebug)]
#[repr(transparent)]
struct Board;
static NEIGHBORS_HOUDS: [Board;14] = [
	Board(0), // 0
	Board(0b00000000111000), // 1
	Board(0), // 2
	Board(0b00000011010000), // 3
	Board(0b00000010101000), // 4
	Board(0b00000110010000), // 5
	Board(0b00001010000000), // 6
	Board(0b00111101000000), // 7
	Board(0b00100010000000), // 8
	Board(0b10010000000000), // 9
	Board(0b10101000000000), // 10
	Board(0b10010000000000), // 11
	Board(0), // 12
	Board(0b00000000000000), // 13
];

static NEIGHBORS_HARE: [Board;14] = [
	Board(0), // 0
	Board(0b00000000111000), // 1
	Board(0), // 2
	Board(0b00000011010010), // 3
	Board(0b00000010101010), // 4
	Board(0b00000110010010), // 5
	Board(0b00001010001000), // 6
	Board(0b00111101111000), // 7
	Board(0b00100010100000), // 8
	Board(0b10010011000000), // 9
	Board(0b10101010000000), // 10
	Board(0b10010110000000), // 11
	Board(0), // 12
	Board(0b00111000000000), // 13
];
impl Board {
	#[inline(always)]
	const fn column(index: u8) -> u8 {
		index / 3
	}
}
// 0 3-6-9  12
//  /|\|/| \
// 1-4-7-10-13
//  \|/|\| /
// 2 5-8-11 14
impl std::fmt::Display for Board {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, r#"   {}-{}-{}"#,
			if (self.0>>3)&1 != 0 {'#'} else {'O'},
			if (self.0>>6)&1 != 0 {'#'} else {'O'},
			if (self.0>>9)&1 != 0 {'#'} else {'O'})?;
		writeln!(f, r#"  /|\|/|\"#)?;
		writeln!(f, r#" {}-{}-{}-{}-{}"#,
			if (self.0>>1)&1 != 0 {'#'} else {'O'},
			if (self.0>>4)&1 != 0 {'#'} else {'O'},
			if (self.0>>7)&1 != 0 {'#'} else {'O'},
			if (self.0>>10)&1 != 0 {'#'} else {'O'},
			if (self.0>>13)&1  != 0 {'#'} else {'O'})?;
		writeln!(f, r#"  \|/|\|/"#)?;
		writeln!(f, r#"   {}-{}-{}"#,
			if (self.0>>5)&1 != 0 {'#'} else {'O'},
			if (self.0>>8)&1 != 0 {'#'} else {'O'},
			if (self.0>>11)&1 != 0 {'#'} else {'O'})
	}
}
#[derive(Clone, Copy)]
pub enum Cell {
	Hound,
	Hare,
	Empty
}
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Move {
	from: u8,
	to: u8
}
impl Move {
	#[inline(always)]
	pub fn from(&self) -> u8 {
		self.from
	}
	#[inline(always)]
	pub fn to(&self) -> u8 {
		self.to
	} 
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct HareAndHounds {
	houds: Board,
	hare: u8,
	turn_and_count: u8
}
impl Default for HareAndHounds {
	#[inline(always)]
	fn default() -> Self {
		Self { houds: Board(42), hare: 13, turn_and_count: 0 }
	}
}
impl HareAndHounds {
	const MAX_MOVES: usize = 9;
	#[inline]
	pub fn play_unchecked(&mut self, m: Move) {
		if self.turn() {
			self.hare = m.to;
			self.turn_and_count -= 100;
		} else {
			self.houds.reset_at_index(m.from as usize);
			self.houds.set_at_index(m.to as usize);
			if Board::column(m.from) == Board::column(m.to) {
				self.turn_and_count += 1;
			} else {
				self.turn_and_count = 0;
			}
			self.turn_and_count += 100;
		}
		//if self.houds.count() != 3 {
			//println!("{}: move:{:?}", self, m);
		//}
		debug_assert!(self.houds.count() == 3, "pb after move")
	}

	#[inline]
	pub fn legal_moves(&self, moves:&mut [Move;Self::MAX_MOVES], len: &mut usize) {
		*len = 0;
		if self.turn() {
			let moves_board = NEIGHBORS_HARE[self.hare as usize] & !self.houds;
			for to in  moves_board.iter_bits() {
				moves[*len] = Move{from:self.hare, to: to as u8};
				*len += 1;
			}
		} else {
			for hound in self.houds.iter_bits() {
				let moves_board = NEIGHBORS_HOUDS[hound as usize] & !self.houds & !Board::from_index(self.hare as usize);
				for to in moves_board.iter_bits() {
					moves[*len] = Move{from:hound as u8, to: to as u8};
					*len += 1;
				}
			}
		}
	}
	#[inline]
	pub fn result(&self) -> GameResult {
		if (NEIGHBORS_HARE[self.hare as usize] & !self.houds).is_empty() {
			return GameResult::Player1;
		}
		if self.turn_and_count % 100 >= 10 {
			return GameResult::Player2; // hare wins
		}
		let hare_col = Board::column(self.hare);
		for i in self.houds.iter_bits() {
			 let hound_col = Board::column(i as u8);
			 if hound_col <= hare_col {
				return GameResult::OnGoing;
			 }
		}
		GameResult::Player2
	}
	
	pub fn cell(&self, x:u8, y:u8) -> Cell {
		if self.hare == Board::index_from_coords(x, y) as u8 {
			Cell::Hare
		} else if self.houds.get(x, y) {
			Cell::Hound
		} else {
			Cell::Empty
		}
	}
	#[inline(always)]
	pub fn turn(&self) -> bool {
		self.turn_and_count >= 100
	}
	#[inline(always)]
	pub fn compute_hash(&self) -> u64 {
		let hounds = (self.houds.0 as u64) << 32;                 // 16 bits
		let hare   = (self.hare as u64) << 48;            // 8 bits
		let turn   = (self.turn_and_count as u64) << 56; // 8 bits

		crate::utils::splitmix64(hounds | hare | turn)

	}
}

impl std::fmt::Display for HareAndHounds {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Turn: {}", self.turn())?;
		writeln!(f, r#"   {}-{}-{}"#,
			if (self.houds.0>>3)&1 != 0 {'F'} else if self.hare == 3 {'L'} else {'.'},
			if (self.houds.0>>6)&1 != 0 {'F'} else if self.hare == 6 {'L'} else {'.'},
			if (self.houds.0>>9)&1 != 0 {'F'} else if self.hare == 9 {'L'} else {'.'})?;
		writeln!(f, r#"  /|\|/|\"#)?;
		writeln!(f, r#" {}-{}-{}-{}-{}"#,
			if (self.houds.0>>1)&1 != 0 {'F'} else if self.hare == 1 {'L'} else {'.'},
			if (self.houds.0>>4)&1 != 0 {'F'} else if self.hare == 4 {'L'} else {'.'},
			if (self.houds.0>>7)&1 != 0 {'F'} else if self.hare == 7 {'L'} else {'.'},
			if (self.houds.0>>10)&1 != 0 {'F'} else if self.hare == 10 {'L'} else {'.'},
			if (self.houds.0>>13)&1  != 0 {'F'} else if self.hare == 13 {'L'} else {'.'})?;
		writeln!(f, r#"  \|/|\|/"#)?;
		writeln!(f, r#"   {}-{}-{}"#,
			if (self.houds.0>>5)&1 != 0 {'F'} else if self.hare == 5 {'L'} else {'.'},
			if (self.houds.0>>8)&1 != 0 {'F'} else if self.hare == 8 {'L'} else {'.'},
			if (self.houds.0>>11)&1 != 0 {'F'} else if self.hare == 11 {'L'} else {'.'})
	}
}
#[test]
fn test_game() {
	let mut hah = HareAndHounds::default();
	let mut i=0;
	let mut rng = crate::utils::Rng::new();
	while hah.result() == GameResult::OnGoing && i< 10000 {
		println!("{}", hah);
		let mut len = 0;
		let mut moves=[Move::default();HareAndHounds::MAX_MOVES];
		hah.legal_moves(&mut moves, &mut len);
		println!("{} {:?}", len, moves);
		hah.play_unchecked(moves[rng.range(0, len)]);
		println!("{:?}", hah.result());
		i+=1;
		assert!(hah.houds.count() == 3);
	}
	println!("{}", hah);
}
#[test]
fn test_game_bug() {
	let mut hah = HareAndHounds::default();
	hah.houds=Board::from_storage(0b0110010000000);
	hah.hare=6;
	hah.turn_and_count = 0;
	println!("{}", hah.houds);
	println!("{}", hah);
	let mut len = 0;
	let mut moves=[Move::default();HareAndHounds::MAX_MOVES];
	hah.legal_moves(&mut moves, &mut len);
	println!("{} {:?}", len, moves);
	hah.play_unchecked(moves[0]);
	println!("{}", hah);
	
}

#[test]
fn test_game_bug2() {
	for _ in 0..1000 {
		test_game_bug();
	} 
}
#[test]
fn test_display() {
	//println!("0\n{}", NEIGHBORS_HOUDS[0]);
	//println!("1\n{}", NEIGHBORS_HOUDS[1]);
	//println!("2\n{}", NEIGHBORS_HOUDS[2]);
	//println!("3\n{}", NEIGHBORS_HOUDS[3]);
	//println!("4\n{}", NEIGHBORS_HOUDS[4]);
	//println!("5\n{}", NEIGHBORS_HOUDS[5]);
	//println!("6\n{}", NEIGHBORS_HOUDS[6]);
	//println!("7\n{}", NEIGHBORS_HOUDS[7]);
	//println!("8\n{}", NEIGHBORS_HOUDS[8]);
	//println!("9\n{}", NEIGHBORS_HOUDS[9]);
	println!("10\n{}", NEIGHBORS_HOUDS[10]);
	println!("11\n{}", NEIGHBORS_HOUDS[11]);
	println!("12\n{}", NEIGHBORS_HOUDS[12]);
	println!("13\n{}", NEIGHBORS_HOUDS[13]);

	println!("0\n{}", NEIGHBORS_HARE[0]);
	println!("1\n{}", NEIGHBORS_HARE[1]);
	println!("2\n{}", NEIGHBORS_HARE[2]);
	println!("3\n{}", NEIGHBORS_HARE[3]);
	println!("4\n{}", NEIGHBORS_HARE[4]);
	println!("5\n{}", NEIGHBORS_HARE[5]);
	println!("6\n{}", NEIGHBORS_HARE[6]);
	println!("7\n{}", NEIGHBORS_HARE[7]);
	println!("8\n{}", NEIGHBORS_HARE[8]);
	println!("9\n{}", NEIGHBORS_HARE[9]);
	println!("10\n{}", NEIGHBORS_HARE[10]);
}
