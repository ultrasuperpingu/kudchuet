//! A definition of the game Connect Four using the library, for use in tests and benchmarks.
#![allow(dead_code)]

use kudchuet::Player;
use kudchuet::ai::minimax::{Evaluation, Evaluator, Game, Winner};

use std::default::Default;
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Board {
	// Some bitboard ideas from http://blog.gamesolver.org/solving-connect-four/06-bitboard/
	/* bit order example:
	 * Leaves a blank row on top.
	 *  5 12 19 26 33 40 47
	 *  4 11 18 25 32 39 46
	 *  3 10 17 24 31 38 45
	 *  2  9 16 23 30 37 44
	 *  1  8 15 22 29 36 43
	 *  0  7 14 21 28 35 42
	 */
	all_pieces: u64,
	pub pieces_to_move: u64,
	num_moves: u8,
	hash: u64,
}

const NUM_COLS: u32 = 7;
const NUM_ROWS: u32 = 6;
const HEIGHT: u32 = NUM_ROWS + 1;
const COL_MASK: u64 = (1 << NUM_ROWS) - 1;

impl Board {
	fn reds_move(&self) -> bool {
		self.num_moves & 1 == 0
	}

	pub fn pieces_just_moved(&self) -> u64 {
		self.all_pieces ^ self.pieces_to_move
	}

	fn update_hash(&mut self, piece: u64) {
		// Lookup the hash for this position and this color.
		let position = piece.trailing_zeros() as usize;
		let color = self.num_moves as usize & 1;
		self.hash ^= HASHES[(position << 1) | color];
	}
}

impl Display for Board {
	fn fmt(&self, f: &mut Formatter) -> Result {
		let red_pieces =
			if self.reds_move() { self.pieces_to_move } else { self.pieces_just_moved() };
		let yellow_pieces =
			if self.reds_move() { self.pieces_just_moved() } else { self.pieces_to_move };
		for row in (0..6).rev() {
			for col in 0..7 {
				write!(
					f,
					"{}",
					if red_pieces >> (row + col * HEIGHT) & 1 != 0 {
						'\u{1F534}'
					} else if yellow_pieces >> (row + col * HEIGHT) & 1 != 0 {
						'\u{1F7E1}'
					} else {
						'\u{25ef}'
					}
				)?;
			}
			writeln!(f)?;
		}
		Ok(())
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Place {
	pub col: u8,
}

impl Place {
	fn col_shift(&self) -> u32 {
		self.col as u32 * HEIGHT
	}
}


#[derive(Debug)]
pub struct C4Game;

impl Game for C4Game {
	type S = Board;
	type M = Place;

	fn generate_moves(b: &Board, moves: &mut Vec<Place>) -> Option<Winner> {
		if let Some(w) = Self::get_winner(b) {
			return Some(w);
		}
		let mut cols = b.all_pieces;
		for i in 0..NUM_COLS {
			if cols & COL_MASK < COL_MASK {
				moves.push(Place { col: i as u8 });
			}
			cols >>= HEIGHT;
		}
		None
	}

	fn get_winner(b: &Board) -> Option<Winner> {
		// Position of pieces for the player that just moved.
		let pieces = b.pieces_just_moved();

		// Detect pairs of two pieces in a row, then pairs of two pairs in a
		// row.
		let matches = |shift| -> bool {
			let pairs = pieces & (pieces >> shift);
			pairs & (pairs >> (2 * shift)) != 0
		};

		if matches(1) || matches(HEIGHT) || matches(HEIGHT + 1) || matches(HEIGHT - 1) {
			if b.reds_move() {
				return Some(Winner::Player(0));
			} else {
				return Some(Winner::Player(1));
			}
		}

		// Full board with no winner.
		if b.num_moves as u32 == NUM_ROWS * NUM_COLS {
			Some(Winner::Draw)
		} else {
			None
		}
	}

	fn apply(b: &mut Board, place: Place) -> Option<Board> {
		//let mut b = b.clone();
		let col = (b.all_pieces >> place.col_shift()) & COL_MASK;
		let new_piece = (col + 1) << place.col_shift();
		// Swap colors
		b.pieces_to_move ^= b.all_pieces;
		b.all_pieces |= new_piece;
		b.num_moves += 1;
		b.update_hash(new_piece);
		//Some(b)
		//println!("play({place:?}):\n{}", b);
		None
	}
	fn undo(b: &mut Board, place: Place) {
		//let col = (b.all_pieces >> place.col_shift()) & COL_MASK;
		//let new_piece = col << place.col_shift();

		let mask = COL_MASK << place.col_shift();
		let stones = b.all_pieces & mask;
		let new_piece = stones.checked_ilog2().map(|i| 1u64 << i).unwrap();

		//println!("dans undo: {new_piece:?}");
		b.update_hash(new_piece);

		b.all_pieces &= !new_piece;
		// Swap colors
		b.pieces_to_move ^= b.all_pieces;
		//b.num_moves -= 1;
		b.num_moves = b.num_moves.wrapping_sub(1);
		//println!("undo({place:?}):\n{}", b);
		
		
	}

	fn zobrist_hash(b: &Board) -> u64 {
		b.hash
	}
	
	fn current_player(state: &Self::S) -> Player {
		if state.reds_move() {
			Player::PLAYER2
		} else {
			Player::PLAYER1
		}
	}
}

pub struct DumbEvaluator;

impl Evaluator for DumbEvaluator {
	type G = C4Game;
	fn evaluate_for(&self, _: &Board, _p: Player) -> Evaluation {
		0
	}
}

impl Board {
	// Return bitmap of all open locations that would complete a four in a row for the given player.
	fn find_fourth_moves(&self, pieces: u64) -> u64 {
		let mut all = self.all_pieces;
		// Mark the fake row on top as full to prevent wrapping around.
		let mut top_row = COL_MASK + 1;
		for _ in 0..NUM_COLS {
			all |= top_row;
			top_row <<= HEIGHT;
		}

		let matches = |shift| -> u64 {
			let pairs = pieces & (pieces >> shift); // Pairs of this color.
			let singles = (pieces >> shift) & !all | (pieces << shift) & !all; // One of this color and one empty.
			(pairs >> (shift * 2)) & singles | (pairs << (shift * 2)) & singles
		};

		// Vertical
		matches(1) |
		// Horizontal
		matches(HEIGHT) |
		// Diagonal
		matches(HEIGHT+1) |
		// Other diagonal
		matches(HEIGHT-1)
	}
}

#[derive(Clone, Default)]
pub struct BasicEvaluator;

impl Evaluator for BasicEvaluator {
	type G = C4Game;
	fn evaluate_for(&self, b: &Board, p: Player) -> Evaluation {
		let player_pieces = b.pieces_to_move;
		let opponent_pieces = b.pieces_just_moved();
		let mut player_wins = b.find_fourth_moves(player_pieces);
		let mut opponent_wins = b.find_fourth_moves(opponent_pieces);

		let mut score = 0;
		// Bonus points for moves in the middle columns.
		for col in 2..5 {
			score +=
				((player_pieces >> (HEIGHT * col)) & COL_MASK).count_ones() as Evaluation;
			score -= ((opponent_pieces >> (HEIGHT * col)) & COL_MASK).count_ones()
				as Evaluation;
		}

		// Count columns that cause immediate win.
		// Count columns that then allow immediate win.
		let mut all = b.all_pieces;
		for _ in 0..NUM_COLS {
			let next_move = (all & COL_MASK) + 1;
			if next_move > COL_MASK {
				continue;
			}
			if next_move & player_wins != 0 {
				score += 10;
			}
			if next_move & opponent_wins != 0 {
				score -= 10;
			}
			let afterwards_move = next_move << 1;
			if afterwards_move & player_wins != 0 {
				score += 5;
			}
			if afterwards_move & opponent_wins != 0 {
				score -= 5;
			}

			all >>= HEIGHT;
			player_wins >>= HEIGHT;
			opponent_wins >>= HEIGHT;
		}
		//TODO
		score
	}
}

// There aren't that many positions per color, so just encode the zobrist hash statically.
const HASHES: [u64; 100] = [
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
	0x2a0f08694d7ea84d,
	0x09cad4dbfc990fa2,
	0xfe5f27499de6b4f8,
	0x3d8ed8ab1d44997f,
	0x2af64deca431f644,
	0xf2712b5274180c36,
	0x30eeae3a821bf86c,
	0x31c921831f06ad2f,
	0x40683ff11655cd2f,
	0xb78183a74cd6cb03,
	0xde9e15a6f99bda2f,
	0xa5293988641edb9b,
];
#[cfg(test)]
mod tests {
	use kudchuet::ai::minimax::util::perft;

use super::Board;

	//depth           count        time        kn/s
	//    0               1       2.4µs       416.7
	//    1               7       1.3µs      5384.6
	//    2              49       7.7µs      6363.6
	//    3             343       1.8µs    190555.6
	//    4            2401     356.2µs      6740.6
	//    5           16807      70.7µs    237722.8
	//    6          117649     152.1µs    773497.7
	//    7          823536     439.9µs   1872098.2
	//    8         5673234       2.6ms   2189507.9
	//    9        39394572      18.7ms   2105907.1
	//   10       268031646     139.9ms   1916202.1
	//   11      1844590828     865.4ms   2131479.2
	//   12     12418296244        6.2s   2011439.2
	//   13     84496181330       64.7s   1306705.9
	//cargo test --release connect4::ex_connect4::tests::perft_test -- --nocapture
	#[test]
	fn perft_test() {
		let mut board = Board::default();

		let nodes = perft::<super::C4Game>(&mut board, 13, true);
		const NB_NODES: [u64;14] = [
			1,
			7,
			49,
			343,
			2401,
			16807,
			117649,
			823536,
			5673234,
			39394572,
			268031646,
			1844590828,
			12418296244,
			84496181330,
		];
		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}

}