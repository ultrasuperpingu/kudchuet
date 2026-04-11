use bitboard_proc_macro::{BitboardDebug, BitboardDisplay, bitboard};
use bitboard::Bitboard;


#[bitboard(width=7, height=7, col_major=true)]
#[derive(Default, BitboardDebug, BitboardDisplay, Hash)]
pub struct Bitboard7x7Col;


impl Bitboard7x7Col {
	#[inline]
	pub fn push(&mut self, col: u8) {
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
	pub fn is_column_full(&self, col: u8) -> bool {
		let mask = Self::col_mask(col);
		//println!("{self}");
		//println!("{}", (*self & mask).count());
		(*self & mask).count() == 6
	}

	#[inline]
	pub fn is_column_empty(&self, col: u8) -> bool {
		let mask = Self::col_mask(col);
		(*self & mask).is_empty()
	}

	#[inline]
	pub fn four_aligned(&self) -> bool {
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
	pub fn top_stone_mask(&self, col: u8) -> Option<u64> {
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
