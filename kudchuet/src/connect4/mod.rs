
#![allow(clippy::uninlined_format_args)]

pub mod game;
pub mod ihm;
pub mod gui;
pub mod ex_connect4;
use std::{fmt, io, str::FromStr};

use ::bitboard::Bitboard;

use crate::common::bitboards::Bitboard7x7Col;
//pub use solver::{score, Solver};


/// An integer ranging from 0 to 6 representing a column of the connect four board.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Column(u8);

impl Column {
	/// Column index ranges from 0 to 6
	pub const fn from_index(index: u8) -> Column {
		assert!(index < 7);
		Column(index)
	}
	pub fn index(&self) -> u8 {
		self.0
	}
}

impl FromStr for Column {
	type Err = &'static str;
	fn from_str(source: &str) -> Result<Column, Self::Err> {
		match source.as_bytes().first() {
			Some(v @ b'1'..=b'7') => Ok(Column(v - b'1')),
			_ => Err("Only digits from 1 to 7 count as valid moves."),
		}
	}
}

impl fmt::Display for Column {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.0 + 1)
	}
}
impl Bitboard7x7Col {
	#[inline]
	fn push(&mut self, col: u8) {
		let mask = Self::col_mask(col);
		let empty = mask & !*self;
		let idx = empty.storage().trailing_zeros() as usize;
		if idx > 49 {
			println!("{} {}",self, col);
			assert!(false, "Ahhhh");
		}
		self.set_at_index(idx);
	}

	//#[inline]
	//fn pop(&mut self, col: u8) {
	//	let mask = Self::col_mask(col);
	//	let filled = mask & *self;
	//	let idx = filled.storage().leading_zeros() as usize;
	//	self.set_at_index(idx, false);
	//}

	#[inline]
	fn is_column_full(&self, col: u8) -> bool {
		let mask = Self::col_mask(col);
		//println!("{self}");
		//println!("{}", (*self & mask).count());
		(*self & mask).count() == 6
	}

	#[inline]
	fn is_column_empty(&self, col: u8) -> bool {
		let mask = Self::col_mask(col);
		(*self & mask).is_empty()
	}

	#[inline]
	fn four_aligned(&self) -> bool {
		let bb = *self;
		(bb & (bb >> 1usize) & (bb >> 2usize) & (bb >> 3usize)).any()
			|| (bb & (bb >> 7usize) & (bb >> 14usize) & (bb >> 21usize)).any()
			|| (bb & (bb >> 6usize) & (bb >> 12usize) & (bb >> 18usize)).any()
			|| (bb & (bb >> 8usize) & (bb >> 16usize) & (bb >> 24usize)).any()
	}
	pub fn pop_top(&mut self, col: u8) -> bool {
		if self.is_column_empty(col) {
			return false
		}
		self.pop_top_unchecked(col);
		true
	}
	fn top_stone_mask(&self, col: u8) -> Option<u64> {
		let mask = Self::col_mask(col);
		//let mask = Self::COLUMN_MASK[col as usize];
		let stones = *self & mask;
		stones.storage().checked_ilog2().map(|i| 1u64 << i)
	}
	pub fn pop_top_unchecked(&mut self, col: u8) {
		let bit = self.top_stone_mask(col).expect("pop_top_unchecked on empty column");
		*self.storage_mut() &= !bit;
	}
	/// Changes the bitmask to represent the stones of the other player
	pub fn flip(&mut self, mask: Self) {
		*self.storage_mut() ^= mask.storage()
	}
	/// Changes the bitmask to represent the stones of the other player and return another instance
	pub fn flipped2(&self, mask: Bitboard7x7Col) -> Self {
		Self::from_storage(self.storage() ^ mask.storage())
	}
	pub fn mask(&mut self, mask: Self) {
		*self.storage_mut() &= mask.storage();
	}
	// A unique key encoding the board. Starting from bit 49 everything is guaranteed to be zero.
	/// Two different boards are guaranteed to have two different keys.
	pub fn key(self, mask: Bitboard7x7Col) -> u64 {
		self.storage() + mask.storage()
	}

	/// Bitmask with `1`s in all positions in which would imply victory for the current player if he
	/// can place a stone in them.
	pub fn winning_positions(self) -> u64 {
		// Vertical (These can only be won by adding one stone on top)
		let mut winning = (self.storage() << 1) & (self.storage() << 2) & (self.storage() << 3);

		let add_left_right_gaps = |shift| {
			let mut w = 0u64;
			// All but the vertical one can be one by adding one to the "left" of three stones, one
			// two the "right", or filling gaps in the middle. We generalize our definition of left
			// and right with the shift variable
			let two_to_the_left = self.storage() << shift & self.storage() << (2 * shift);
			// Two to the left, and also a third one
			w |= two_to_the_left & self.storage() << (3 * shift);
			// Two to the left, and also one to the right
			w |= two_to_the_left & self.storage() >> shift;
			let two_to_the_right = self.storage() >> shift & self.storage() >> (2 * shift);
			// Two to the right and one to the left
			w |= two_to_the_right & self.storage() << shift;
			// Two to the right and also a third one
			w |= two_to_the_right & self.storage() >> (3 * shift);
			w
		};

		// Horizontal; Can be won by adding a stone left, right, but also by filling a gap.
		winning |= add_left_right_gaps(6 + 1);

		// Diagonal; Bottom left to top right
		winning |= add_left_right_gaps(6 + 1 + 1);

		// Diagonal; Top left to bottom right
		winning |= add_left_right_gaps(6 + 1 - 1);

		winning & Self::FULL.storage()
	}
	/// Bitmask with possible positions for the next stone to land in
	pub fn possible(self) -> u64 {
		(self.storage() + Self::SOUTH_BORDER.storage()) & Self::FULL.storage()
	}

}

/// State of a field in a four in a row board
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
	Empty,
	PlayerOne,
	PlayerTwo,
}

/// Implementation of the Connect Four game. The board is implemented as two 64 bit masks. It allows
/// for fast checking of winning conditions and legal moves. Apart from being able to play connect
/// four, this type also features some utility functions which can help with implementations of
/// heuristics and solvers.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct ConnectFour {
	/// Bitboard encoding the stones of the player who did insert the last stone. Starts with Player
	/// one.
	last: Bitboard7x7Col,
	/// Bitboard encoding all cells containing stones, no matter the player.
	both: Bitboard7x7Col,
}

impl ConnectFour {
	/// Create an empty board
	pub fn new() -> ConnectFour {
		ConnectFour {
			last: Bitboard7x7Col::default(),
			both: Bitboard7x7Col::default(),
		}
	}

	/// Inserts a stone for the current player if move is legal. `true` if move has been legal
	pub fn play(&mut self, column: Column) -> bool {
		// Let's check if the move is legal, otherwise return false.
		if self.both.is_column_full(column.0) {
			return false;
		}
		self.play_unchecked(column);
		true
	}
	/// Inserts a stone for the current player.
	pub fn play_unchecked(&mut self, column: Column) {
		let copy=*self;
		// Now we add a stone to the bitmask for both player.
		self.both.push(column.0);
		// Flip players after adding the stone, so the stone is accounted for the last player
		self.last.flip(self.both);
		if !(self.both&Bitboard7x7Col::NORTH_BORDER).is_empty() {
			println!("{column:?}\n{copy}\n{self}");
			println!("{:?}\n{}\n{}",copy.both, self.both, self.both&Bitboard7x7Col::NORTH_BORDER);
			println!("{:?}\n{}\n{}",copy.last, self.both, self.both&Bitboard7x7Col::NORTH_BORDER);
			debug_assert!((self.both&Bitboard7x7Col::NORTH_BORDER).is_empty())
		}
		
	}
	pub fn undo(&mut self, column: Column) -> bool {
		if self.both.is_column_empty(column.0) {
			return false;
		}
		self.undo_unchecked(column);
		true
	}
	pub fn undo_unchecked(&mut self, column: Column) {
		self.both.pop_top_unchecked(column.0);
		self.last.flip(self.both);
		self.last.mask(self.both);
	}
	/// `true` if the column is not full.
	#[inline]
	pub fn is_legal_move(&self, column: Column) -> bool {
		!self.both.is_column_full(column.0)
	}

	/// Create a game state from a sequence of moves. Each move represented as a number from 1 to 7
	/// standing for the column the player put in their stones.
	pub fn from_move_list(move_list: &str) -> ConnectFour {
		let mut game = ConnectFour::new();
		for c in move_list
			.as_bytes()
			.iter()
			.map(|c| c - b'1')
			.map(Column::from_index)
		{
			if !game.play(c) {
				panic!("Illegal move in String describing Connect Four Game")
			}
		}
		game
	}

	/// Prints out a text representation of a board to `out`
	#[inline]
	pub fn print_to(&self, mut out: impl io::Write) -> io::Result<()> {
		write!(out, "{self}")
	}

	#[inline]
	pub fn legal_moves(&self) -> Vec<Column> {
		let mut moves = [Column(0); 7];
		let n = self.legal_moves_array(&mut moves);
		moves[0..n].iter().copied().collect()
	}

	#[inline]
	pub fn legal_moves_array(&self, out: &mut [Column; 7]) -> usize {
		if self.last.four_aligned() {
			return 0;
		}
		let mut n = 0;
		for col in [3,4,2,5,1,6,0].map(Column::from_index) {
			if self.is_legal_move(col) {
				out[n] = col;
				n += 1;
			}
		}
		n
	}
	const PLAYERS : [Cell; 2] = [Cell::PlayerOne, Cell::PlayerTwo];
		
	/// Access any cell of the board and find out whether it is empty, or holding a stone of Player
	/// One or Two.
	#[inline]
	pub fn cell(&self, x: u8, y: u8) -> Cell {
		if !self.both.get(x, y) {
			Cell::Empty
		} else if !self.last.get(x, y) {
			Self::PLAYERS[self.both.count() as usize % 2]
		} else {
			Self::PLAYERS[(self.both.count() as usize + 1) % 2]
		}
	}
	#[inline]
	pub fn player_turn(&self) -> Cell {
		Self::PLAYERS[(self.both.count() as usize) % 2]
	}
	
	pub fn heuristic_(player: Bitboard7x7Col, both: Bitboard7x7Col) -> u32 {
		let openings = player.winning_positions();
		// only count openings, which are not blocked by enemy stones already
		let true_openings = openings & !(both.storage());
		true_openings.count_ones()
	}

	/// Heurisitc used to decide which moves to explore first, in order to allow for better pruning
	/// of the search tree. Higher means better for the player which put in the last stone.
	pub fn heuristic(&self) -> u32 {
		Self::heuristic_(self.last, self.both)
	}
	pub fn opponent_heuristic(&self) -> u32 {
		Self::heuristic_(self.last.flipped2(self.both), self.both)
	}
	/// Number of stones in the board
	pub fn stones(&self) -> u8 {
		self.both.count() as u8
	}

	/// `true` if the player which did insert the last stone has won the game.
	pub fn is_victory(&self) -> bool {
		self.last.four_aligned()
	}
	pub fn is_loose(&self) -> bool {
		self.last.flipped2(self.both).four_aligned()
	}
	/// Uses the first 49 Bits to uniquely encode the board.
	pub fn encode(&self) -> u64 {
		self.last.storage() + self.both.storage()
	}

	/// `true` if the current player has winning moves available
	pub fn can_win_in_next_move(&self) -> bool {
		let mut current = self.last;
		current.flip(self.both);
		self.both.possible() & current.winning_positions() != 0
	}

	/// `true` if game has a winner or is a draw.
	pub fn is_over(&self) -> bool {
		self.stones() == 42 || self.is_victory()
	}

	/// List all moves, which prevent the opponent from winning immediately. Only gives valid results
	/// if [`Self::can_win_in_next_move`] is `false`.
	pub fn non_loosing_moves(&self) -> impl Iterator<Item = Column> {
		debug_assert!(!self.can_win_in_next_move());
		let nlm = self.non_loosing_moves_impl();
		(0..7).filter(move |&i| nlm.contains(i)).map(Column::from_index)
	}

	// Only valid to call if `can_win_in_next_move` is `false`.
	fn non_loosing_moves_impl(&self) -> NonLoosingMoves {
		debug_assert!(!self.can_win_in_next_move());
		NonLoosingMoves::new(self.last, self.both)
	}
}
#[derive(Clone, Copy)]
pub struct NonLoosingMoves(u64);

impl NonLoosingMoves {
	pub (crate) fn new(opponent: Bitboard7x7Col, both: Bitboard7x7Col) -> Self {
		// Check if we need to block a stone, to prevent the opponent from winning
		let openings = opponent.winning_positions();
		let mut possible = both.possible();
		let forced_moves = openings & possible;
		if forced_moves != 0 {
			// If there are more than two, we can not prevent the opponent from winning.
			if forced_moves & (forced_moves - 1) != 0 {
				return Self(0);
			}
			possible = forced_moves;
		};
		// Do not play below an opening to prevent giving opponent a winning move
		possible &= !(openings >> 1);
		Self(possible)
	}

	/// `true` if there are no moves left, which do not prevent the opponent from winning with in
	/// his/her next turn.
	pub fn is_empty(self) -> bool {
		self.0 == 0
	}

	/// `true` if throwing a stone in the indexed column is not loosing immediatly in the next
	/// opponents turn.
	pub fn contains(self, index: u8) -> bool {
		self.0 & Bitboard7x7Col::col_mask(index).storage() != 0
	}
	pub fn to_moves(self) -> Vec<u8> {
		(0..7u8).filter(move |&c| self.contains(c)).collect()
	}
}
impl fmt::Display for ConnectFour {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for y in (0..6).rev() {
			for field in (0..7).map(|x| self.cell(x, y)) {
				let c = match field {
					Cell::PlayerOne => 'X',
					Cell::PlayerTwo => 'O',
					Cell::Empty => ' ',
				};
				write!(f, "|{}", c)?;
			}
			writeln!(f, "|")?;
		}
		writeln!(f, "---------------\n 1 2 3 4 5 6 7")
	}
}
#[cfg(test)]
mod tests {
	use crate::{common::{bitboards::Bitboard7x7Col, gui::BoardGame}, connect4::{Column, ConnectFour}};

	#[test]
	fn test_legal_moves_initial() {
		let mut b = ConnectFour::new();
		let mut moves = [Column::from_index(0); 7];
		let n = b.legal_moves_array(&mut moves);
		assert_eq!(n, 7);
		println!("{}", b);
		
		for _ in 0..6 {
			b.play(Column(0));
			println!("{}", b);
			
		}

		let mut moves = [Column::from_index(0); 7];
		let n = b.legal_moves_array(&mut moves);
		assert_eq!(n, 6);
	}
	#[test]
	fn test_legal_moves_with_full() {
		let mut b = ConnectFour::new();
		b.both = Bitboard7x7Col::from_storage((1<<6)-1);
		b.last.set_at_index(1);
		b.last.set_at_index(3);
		b.last.set_at_index(5);
		println!("{b}");
		println!("{:?}", b.legal_moves());
		assert!(!b.is_legal_move(Column(0)));
	}
	#[test]
	fn test_play() {
		let mut b = ConnectFour::new();
		let mut moves = [Column::from_index(0); 7];
		let mut n = b.legal_moves_array(&mut moves);
		while !b.is_over() {
			b.play_unchecked(moves[rand::random_range(0..n)]);
			n = b.legal_moves_array(&mut moves);
		}
		println!("{}", b);
		println!("{:?}", b.result());
	}
	#[test]
	fn test_play2() {
		let mut b = ConnectFour::new();
		b.both = Bitboard7x7Col::from_storage(0b0111111_0000000_0000001_0000001_0000111_0000001_0000000);
		b.last = Bitboard7x7Col::from_storage(0b0001110_0000000_0000001_0000000_0000100_0000001_0000000);
		println!("{b}");
		println!("{}", b.is_legal_move(Column(6)));
		println!("{:?}", b.legal_moves());
		let mut moves = [Column::from_index(0); 7];
		let n = b.legal_moves_array(&mut moves);
		println!("{n}{:?}", moves);
	}
	#[test]
	fn test_hash_collide() {
		for _ in 0..100000 {
			let mut hashes = std::collections::HashMap::new();
			let mut b = ConnectFour::new();
			let mut moves = [Column::from_index(0); 7];
			let mut n = b.legal_moves_array(&mut moves);
			while !b.is_over() {
				b.play_unchecked(moves[rand::random_range(0..n)]);
				let hash = b.encode();
				if hashes.contains_key(&hash) {
					println!("collision on {} :\n {}\n{}", hash, hashes[&hash], b);
				}
				assert!(!hashes.contains_key(&hash));
				hashes.insert(hash, b.clone());
				n = b.legal_moves_array(&mut moves);
			}
		}
	}
}