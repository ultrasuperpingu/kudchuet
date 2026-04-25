use crate::{bitboard::Goban, rules::Gomoku};

impl Gomoku {
	pub const fn broken_four(mine: &Goban, _other: &Goban, empty: &Goban) -> Goban {
		let mut res;
		let mine_pattern = Goban::detect_pattern_h(mine, 0b10111);
		let empty_pattern = Goban::detect_pattern_h(empty, 0b01000);
		res = mine_pattern.and_const(&empty_pattern);

		let mine_pattern = Goban::detect_pattern_h(mine, 0b11011);
		let empty_pattern = Goban::detect_pattern_h(empty, 0b00100);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		let mine_pattern = Goban::detect_pattern_h(mine, 0b11101);
		let empty_pattern = Goban::detect_pattern_h(empty, 0b00010);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		let mine_pattern = Goban::detect_pattern_v(mine, 0b10111);
		let empty_pattern = Goban::detect_pattern_v(empty, 0b01000);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		let mine_pattern = Goban::detect_pattern_v(mine, 0b11011);
		let empty_pattern = Goban::detect_pattern_v(empty, 0b00100);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		let mine_pattern = Goban::detect_pattern_v(mine, 0b11101);
		let empty_pattern = Goban::detect_pattern_v(empty, 0b00010);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		let mine_pattern = Goban::detect_pattern_diag_inc(mine, 0b10111);
		let empty_pattern = Goban::detect_pattern_diag_inc(empty, 0b01000);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		let mine_pattern = Goban::detect_pattern_diag_inc(mine, 0b11011);
		let empty_pattern = Goban::detect_pattern_diag_inc(empty, 0b00100);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		let mine_pattern = Goban::detect_pattern_diag_inc(mine, 0b11101);
		let empty_pattern = Goban::detect_pattern_diag_inc(empty, 0b00010);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		let mine_pattern = Goban::detect_pattern_diag_dec(mine, 0b10111);
		let empty_pattern = Goban::detect_pattern_diag_dec(empty, 0b01000);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		let mine_pattern = Goban::detect_pattern_diag_dec(mine, 0b11011);
		let empty_pattern = Goban::detect_pattern_diag_dec(empty, 0b00100);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		let mine_pattern = Goban::detect_pattern_diag_dec(mine, 0b11101);
		let empty_pattern = Goban::detect_pattern_diag_dec(empty, 0b00010);
		res.or_assign_const(&mine_pattern.and_const(&empty_pattern));

		res
	}
	pub fn open_two(mine: &Goban, _other: &Goban, empty: &Goban) -> Goban {
		Self::open_two_h(mine, &empty)
			| Self::open_two_v(mine, &empty)
			| Self::open_two_diag_dec(mine, &empty)
			| Self::open_two_diag_inc(mine, &empty)
	}
	pub fn open_two_h(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_h(mine, 0b0110);
		let empty_pattern = Goban::detect_pattern_h(empty, 0b1001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_two_v(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_v(mine, 0b0110);
		let empty_pattern = Goban::detect_pattern_v(empty, 0b1001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_two_diag_inc(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_diag_inc(mine, 0b0110);
		let empty_pattern = Goban::detect_pattern_diag_inc(empty, 0b1001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_two_diag_dec(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_diag_dec(mine, 0b0110);
		let empty_pattern = Goban::detect_pattern_diag_dec(empty, 0b1001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub fn open_split_three(mine: &Goban, _other: &Goban, empty: &Goban) -> Goban {
		let mut res = Self::open_split_three_h(mine, &empty);
		res.or_assign_const(&Self::open_split_three_v(mine, &empty));
		res.or_assign_const(&Self::open_split_three_diag_dec(mine, &empty));
		res.or_assign_const(&Self::open_split_three_diag_inc(mine, &empty));
		res
	}
	pub fn open_split_three_h(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_h(mine, 0b011010);
		let empty_pattern = Goban::detect_pattern_h(empty, 0b100101);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_split_three_v(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_v(mine, 0b011010);
		let empty_pattern = Goban::detect_pattern_v(empty, 0b100101);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_split_three_diag_inc(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_diag_inc(mine, 0b011010);
		let empty_pattern = Goban::detect_pattern_diag_inc(empty, 0b100101);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_split_three_diag_dec(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_diag_dec(mine, 0b011010);
		let empty_pattern = Goban::detect_pattern_diag_dec(empty, 0b100101);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub fn open_three(mine: &Goban, _other: &Goban, empty: &Goban) -> Goban {
		let mut res = Self::open_three_h(mine, &empty);
		res.or_assign_const(&Self::open_three_v(mine, &empty));
		res.or_assign_const(&Self::open_three_diag_dec(mine, &empty));
		res.or_assign_const(&Self::open_three_diag_inc(mine, &empty));
		res
	}
	pub fn open_three_h(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_h(mine, 0b01110);
		let empty_pattern = Goban::detect_pattern_h(empty, 0b10001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_three_v(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_v(mine, 0b01110);
		let empty_pattern = Goban::detect_pattern_v(empty, 0b10001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_three_diag_inc(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_diag_inc(mine, 0b01110);
		let empty_pattern = Goban::detect_pattern_diag_inc(empty, 0b10001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_three_diag_dec(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_diag_dec(mine, 0b01110);
		let empty_pattern = Goban::detect_pattern_diag_dec(empty, 0b10001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_four(mine: &Goban, _other: &Goban, empty: &Goban) -> Goban {
		let mut res = Self::open_four_h(mine, &empty);
		res.or_assign_const(&Self::open_four_v(mine, &empty));
		res.or_assign_const(&Self::open_four_diag_inc(mine, &empty));
		res.or_assign_const(&Self::open_four_diag_dec(mine, &empty));
		res
	}

	pub const fn open_four_h(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_h(mine, 0b011110);
		let empty_pattern = Goban::detect_pattern_h(empty, 0b100001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_four_v(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_v(mine, 0b011110);
		let empty_pattern = Goban::detect_pattern_v(empty, 0b100001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn open_four_diag_inc(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_diag_inc(mine, 0b011110);
		let empty_pattern = Goban::detect_pattern_diag_inc(empty, 0b100001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}

	pub const fn open_four_diag_dec(mine: &Goban, empty: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_diag_dec(mine, 0b011110);
		let empty_pattern = Goban::detect_pattern_diag_dec(empty, 0b100001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res
	}
	pub const fn closed_four(mine: &Goban, other: &Goban, empty: &Goban) -> Goban {
		let mut res = Self::closed_four_h(mine, &empty, other);
		res.or_assign_const(&Self::closed_four_v(mine, &empty, other));
		res.or_assign_const(&Self::closed_four_diag_inc(mine, &empty, other));
		res.or_assign_const(&Self::closed_four_diag_dec(mine, &empty, other));
		res
	}

	pub const fn closed_four_h(mine: &Goban, empty: &Goban, other: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_h(mine, 0b011110);
		let other_pattern = Goban::detect_pattern_h(other, 0b100000);
		let empty_pattern = Goban::detect_pattern_h(empty, 0b000001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res.and_assign_const(&other_pattern);

		let mine_pattern = Goban::detect_pattern_h(mine, 0b011110);
		let other_pattern = Goban::detect_pattern_h(other, 0b000001);
		let empty_pattern = Goban::detect_pattern_h(empty, 0b100000);
		res.or_assign_const(
			&mine_pattern
				.and_const(&empty_pattern)
				.and_const(&other_pattern),
		);
		res
	}
	pub const fn closed_four_v(mine: &Goban, empty: &Goban, other: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_v(mine, 0b011110);
		let other_pattern = Goban::detect_pattern_v(other, 0b100000);
		let empty_pattern = Goban::detect_pattern_v(empty, 0b000001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res.and_assign_const(&other_pattern);

		let mine_pattern = Goban::detect_pattern_v(mine, 0b011110);
		let other_pattern = Goban::detect_pattern_v(other, 0b000001);
		let empty_pattern = Goban::detect_pattern_v(empty, 0b100000);
		res.or_assign_const(
			&mine_pattern
				.and_const(&empty_pattern)
				.and_const(&other_pattern),
		);
		res
	}
	pub const fn closed_four_diag_inc(mine: &Goban, empty: &Goban, other: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_diag_inc(mine, 0b011110);
		let other_pattern = Goban::detect_pattern_diag_inc(other, 0b100000);
		let empty_pattern = Goban::detect_pattern_diag_inc(empty, 0b000001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res.and_assign_const(&other_pattern);

		let mine_pattern = Goban::detect_pattern_diag_inc(mine, 0b011110);
		let other_pattern = Goban::detect_pattern_diag_inc(other, 0b000001);
		let empty_pattern = Goban::detect_pattern_diag_inc(empty, 0b100000);
		res.or_assign_const(
			&mine_pattern
				.and_const(&empty_pattern)
				.and_const(&other_pattern),
		);
		res
	}
	pub const fn closed_four_diag_dec(mine: &Goban, empty: &Goban, other: &Goban) -> Goban {
		let mine_pattern = Goban::detect_pattern_diag_dec(mine, 0b011110);
		let other_pattern = Goban::detect_pattern_diag_dec(other, 0b100000);
		let empty_pattern = Goban::detect_pattern_diag_dec(empty, 0b000001);
		let mut res = mine_pattern;
		res.and_assign_const(&empty_pattern);
		res.and_assign_const(&other_pattern);

		let mine_pattern = Goban::detect_pattern_diag_dec(mine, 0b011110);
		let other_pattern = Goban::detect_pattern_diag_dec(other, 0b000001);
		let empty_pattern = Goban::detect_pattern_diag_dec(empty, 0b100000);
		res.or_assign_const(
			&mine_pattern
				.and_const(&empty_pattern)
				.and_const(&other_pattern),
		);
		res
	}
}
#[cfg(test)]
mod tests {
	use crate::{bitboard::Goban, rules::Gomoku};

	fn goban_from_coords(coords: &[(u8, u8)]) -> Goban {
		let mut g = Goban::default();
		for &(x, y) in coords {
			g.set(x, y);
		}
		g
	}
	fn setup(mine_coords: &[(u8, u8)], other_coords: &[(u8, u8)]) -> (Goban, Goban, Goban) {
		let mine = goban_from_coords(mine_coords);
		let other = goban_from_coords(other_coords);
		let empty = mine.or_const(&other).not_const();
		(mine, other, empty)
	}
	macro_rules! assert_pattern {
		($cond:expr, $msg:expr) => {
			if !$cond {
				panic!("Pattern test failed: {}", $msg);
			}
		};
	}

	/*pub const fn pattern_shift<SHIFTFN: const Fn(&Goban) -> Goban>(mine: &Goban, mine_mask: u64, shift_fn: SHIFTFN) -> Goban {
		let mut mask = mine_mask;
		let mut res = Goban::FULL;

		let mut shifted = mine.clone_const();
		let mut current_nb_shift = 0;
		while mask != 0 {
			let lsb = mask.trailing_zeros();
			mask &= mask - 1;

			let mut i = current_nb_shift;
			while i < lsb {
				shifted = shift_fn(&shifted);
				i += 1;
				current_nb_shift+=1;
			}

			res = res.and_const(&shifted);
		}

		res
	}*/

	#[test]
	fn test_broken_four_h() {
		let (mine, _other, empty) = setup(
			&[(5, 5), (6, 5), (8, 5), (9, 5)], // XX.XX
			&[],
		);
		println!("{}", mine);
		assert_pattern!(
			Gomoku::broken_four(&mine, &_other, &empty).any(),
			"broken four horizontal should be detected"
		);
	}
	#[test]
	fn test_open_three_h() {
		let (mine, _other, empty) = setup(&[(5, 5), (6, 5), (7, 5)], &[]);
		println!("{}", mine);
		assert_pattern!(
			Gomoku::open_three_h(&mine, &empty).any(),
			//test_has_open_three_h(&mine, &empty),
			"open three horizontal should be detected"
		);
	}
	#[test]
	fn test_open_four_diag_inc() {
		let (mine, _other, empty) = setup(&[(5, 5), (6, 6), (7, 7), (8, 8)], &[]);
		println!("{}", mine);
		assert_pattern!(
			Gomoku::open_four_diag_inc(&mine, &empty).any(),
			"open four diag inc should be detected"
		);
	}
	#[test]
	fn test_closed_four_diag_inc() {
		let (mine, other, empty) = setup(&[(5, 5), (6, 6), (7, 7), (8, 8)], &[(9, 9)]);
		println!("{}\n{}", mine, other);
		assert_pattern!(
			Gomoku::closed_four_diag_inc(&mine, &empty, &other).any(),
			"closed four diag dec should be detected"
		);
	}
}
