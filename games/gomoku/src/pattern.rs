use crate::{bitboard::Goban, rules::Gomoku};

impl Gomoku {
	pub const fn detect_pattern_h(mine: &Goban, mine_mask: u64) -> Goban {
		let mut mask = mine_mask;
		let mut res = Goban::FULL;

		let mut shifted = mine.clone_const();
		let mut current_nb_shift = 0;
		while mask != 0 {
			let lsb = mask.trailing_zeros();
			mask &= mask - 1;

			let mut i = current_nb_shift;
			while i < lsb {
				shifted.shift_e();
				i += 1;
				current_nb_shift+=1;
			}

			res.and_assign_const(&shifted);
		}

		res
	}
	pub const fn detect_pattern_v(mine: &Goban, mine_mask: u64) -> Goban {
		let mut mask = mine_mask;
		let mut res = Goban::FULL;

		let mut shifted = mine.clone_const();
		let mut current_nb_shift = 0;
		while mask != 0 {
			let lsb = mask.trailing_zeros();
			mask &= mask - 1;

			let mut i = current_nb_shift;
			while i < lsb {
				shifted.shift_n();
				i += 1;
				current_nb_shift+=1;
			}

			res.and_assign_const(&shifted);
		}

		res
	}
	pub const fn detect_pattern_diag_inc(mine: &Goban, mine_mask: u64) -> Goban {
		let mut mask = mine_mask;
		let mut res = Goban::FULL;

		let mut shifted = mine.clone_const();
		let mut current_nb_shift = 0;
		while mask != 0 {
			let lsb = mask.trailing_zeros();
			mask &= mask - 1;

			let mut i = current_nb_shift;
			while i < lsb {
				shifted.shift_ne();
				i += 1;
				current_nb_shift+=1;
			}

			res.and_assign_const(&shifted);
		}

		res
	}
	pub const fn detect_pattern_diag_dec(mine: &Goban, mine_mask: u64) -> Goban {
		let mut mask = mine_mask;
		let mut res = Goban::FULL;

		let mut shifted = mine.clone_const();
		let mut current_nb_shift = 0;
		while mask != 0 {
			let lsb = mask.trailing_zeros();
			mask &= mask - 1;

			let mut i = current_nb_shift;
			while i < lsb {
				shifted.shift_se();
				i += 1;
				current_nb_shift+=1;
			}

			res.and_assign_const(&shifted);
		}

		res
	}
	pub const fn has_broken_four(mine: &Goban, other: &Goban) -> bool {
		let all = mine.or_const(other);
		let empty = all.not_const();
		Self::has_broken_four_h(mine, &empty)
			|| Self::has_broken_four_v(mine, &empty)
			|| Self::has_broken_four_diag_inc(mine, &empty)
			|| Self::has_broken_four_diag_dec(mine, &empty)
	}
	pub const fn has_broken_four_h(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_h(mine, 0b11011);
		let empty_pattern = Self::detect_pattern_h(empty, 0b00100);
		mine_pattern.and_const(&empty_pattern).any()
	}

	pub const fn has_broken_four_v(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_v(mine, 0b11011);
		let empty_pattern = Self::detect_pattern_v(empty, 0b00100);
		mine_pattern.and_const(&empty_pattern).any()
	}
	pub const fn has_broken_four_diag_inc(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_diag_inc(mine, 0b11011);
		let empty_pattern = Self::detect_pattern_diag_inc(empty, 0b00100);
		mine_pattern.and_const(&empty_pattern).any()
	}
	pub const fn has_broken_four_diag_dec(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_diag_dec(mine, 0b11011);
		let empty_pattern = Self::detect_pattern_diag_dec(empty, 0b00100);
		mine_pattern.and_const(&empty_pattern).any()
	}

	pub fn has_open_three(mine: &Goban, other: &Goban) -> bool {
		let all = mine.or_const(other);
		let empty = all.not_const();
		Self::has_open_three_h(mine, &empty)
			|| Self::has_open_three_v(mine, &empty)
			|| Self::has_open_three_diag_dec(mine, &empty)
			|| Self::has_open_three_diag_inc(mine, &empty)
	}
	pub fn has_open_three_h(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_h(mine, 0b01110);
		let empty_pattern = Self::detect_pattern_h(empty, 0b10001);
		mine_pattern.and_const(&empty_pattern).any()
	}
	pub const fn has_open_three_v(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_v(mine, 0b01110);
		let empty_pattern = Self::detect_pattern_v(empty, 0b10001);
		mine_pattern.and_const(&empty_pattern).any()
	}
	pub const fn has_open_three_diag_inc(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_diag_inc(mine, 0b01110);
		let empty_pattern = Self::detect_pattern_diag_inc(empty, 0b10001);
		mine_pattern.and_const(&empty_pattern).any()
	}
	pub const fn has_open_three_diag_dec(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_diag_dec(mine, 0b01110);
		let empty_pattern = Self::detect_pattern_diag_dec(empty, 0b10001);
		mine_pattern.and_const(&empty_pattern).any()
	}
	pub const fn has_open_four(mine: &Goban, other: &Goban) -> bool {
		let empty = mine.or_const(other).not_const();
		Self::has_open_four_h(mine, &empty)
			|| Self::has_open_four_v(mine, &empty)
			|| Self::has_open_four_diag_inc(mine, &empty)
			|| Self::has_open_four_diag_dec(mine, &empty)
	}

	pub const fn has_open_four_h(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_h(mine, 0b011110);
		let empty_pattern = Self::detect_pattern_h(empty, 0b100001);
		mine_pattern.and_const(&empty_pattern).any()
	}
	pub const fn has_open_four_v(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_v(mine, 0b011110);
		let empty_pattern = Self::detect_pattern_v(empty, 0b100001);
		mine_pattern.and_const(&empty_pattern).any()
	}
	pub const fn has_open_four_diag_inc(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_diag_inc(mine, 0b011110);
		let empty_pattern = Self::detect_pattern_diag_inc(empty, 0b100001);
		mine_pattern.and_const(&empty_pattern).any()
	}

	pub const fn has_open_four_diag_dec(mine: &Goban, empty: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_diag_dec(mine, 0b011110);
		let empty_pattern = Self::detect_pattern_diag_dec(empty, 0b100001);
		mine_pattern.and_const(&empty_pattern).any()
	}
	pub const fn has_closed_four(mine: &Goban, other: &Goban) -> bool {
		let empty = mine.or_const(other).not_const();

		Self::has_closed_four_h(mine, &empty, other)
			|| Self::has_closed_four_v(mine, &empty, other)
			|| Self::has_closed_four_diag_inc(mine, &empty, other)
			|| Self::has_closed_four_diag_dec(mine, &empty, other)
	}

	pub const fn has_closed_four_h(mine: &Goban, empty: &Goban, other: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_h(mine, 0b011110);
		let other_pattern = Self::detect_pattern_h(other, 0b100000);
		let empty_pattern = Self::detect_pattern_h(empty, 0b000001);
		let mut res = mine_pattern.and_const(&empty_pattern).and_const(&other_pattern).any();
		if !res {
			let mine_pattern = Self::detect_pattern_h(mine, 0b011110);
			let other_pattern = Self::detect_pattern_h(other, 0b000001);
			let empty_pattern = Self::detect_pattern_h(empty, 0b100000);
			res = mine_pattern.and_const(&empty_pattern).and_const(&other_pattern).any();
		}
		res
	}
	pub const fn has_closed_four_v(mine: &Goban, empty: &Goban, other: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_v(mine, 0b011110);
		let other_pattern = Self::detect_pattern_v(other, 0b100000);
		let empty_pattern = Self::detect_pattern_v(empty, 0b000001);
		let mut res = mine_pattern.and_const(&empty_pattern).and_const(&other_pattern).any();
		if !res {
			let mine_pattern = Self::detect_pattern_v(mine, 0b011110);
			let other_pattern = Self::detect_pattern_v(other, 0b000001);
			let empty_pattern = Self::detect_pattern_v(empty, 0b100000);
			res = mine_pattern.and_const(&empty_pattern).and_const(&other_pattern).any();
		}
		res
	}
	pub const fn has_closed_four_diag_inc(mine: &Goban, empty: &Goban, other: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_diag_inc(mine, 0b011110);
		let other_pattern = Self::detect_pattern_diag_inc(other, 0b100000);
		let empty_pattern = Self::detect_pattern_diag_inc(empty, 0b000001);
		let mut res = mine_pattern.and_const(&empty_pattern).and_const(&other_pattern).any();
		if !res {
			let mine_pattern = Self::detect_pattern_diag_inc(mine, 0b011110);
			let other_pattern = Self::detect_pattern_diag_inc(other, 0b000001);
			let empty_pattern = Self::detect_pattern_diag_inc(empty, 0b100000);
			res = mine_pattern.and_const(&empty_pattern).and_const(&other_pattern).any();
		}
		res
	}
	pub const fn has_closed_four_diag_dec(mine: &Goban, empty: &Goban, other: &Goban) -> bool {
		let mine_pattern = Self::detect_pattern_diag_dec(mine, 0b011110);
		let other_pattern = Self::detect_pattern_diag_dec(other, 0b100000);
		let empty_pattern = Self::detect_pattern_diag_dec(empty, 0b000001);
		let mut res = mine_pattern.and_const(&empty_pattern).and_const(&other_pattern).any();
		if !res {
			let mine_pattern = Self::detect_pattern_diag_dec(mine, 0b011110);
			let other_pattern = Self::detect_pattern_diag_dec(other, 0b000001);
			let empty_pattern = Self::detect_pattern_diag_dec(empty, 0b100000);
			res = mine_pattern.and_const(&empty_pattern).and_const(&other_pattern).any();
		}
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
			Gomoku::has_broken_four_h(&mine, &empty),
			"broken four horizontal should be detected"
		);
	}
	#[test]
	fn test_broken_four_v() {
		let (mine, _other, empty) = setup(&[(5, 5), (5, 6), (5, 8), (5, 9)], &[]);
		println!("{}", mine);
		assert_pattern!(
			Gomoku::has_broken_four_v(&mine, &empty),
			"broken four vertical should be detected"
		);
	}
	#[test]
	fn test_open_three_h() {
		let (mine, _other, empty) = setup(&[(5, 5), (6, 5), (7, 5)], &[]);
		println!("{}", mine);
		assert_pattern!(
			Gomoku::has_open_three_h(&mine, &empty),
			//test_has_open_three_h(&mine, &empty),
			"open three horizontal should be detected"
		);
	}
	#[test]
	fn test_open_four_diag_inc() {
		let (mine, _other, empty) = setup(&[(5, 5), (6, 6), (7, 7), (8, 8)], &[]);
		println!("{}", mine);
		assert_pattern!(
			Gomoku::has_open_four_diag_inc(&mine, &empty),
			"open four diag inc should be detected"
		);
	}
	#[test]
	fn test_closed_four_diag_inc() {
		let (mine, other, empty) = setup(
			&[(5, 5), (6, 6), (7, 7), (8, 8)],
			&[(9, 9)],
		);
		println!("{}\n{}", mine, other);
		assert_pattern!(
			Gomoku::has_closed_four_diag_inc(&mine, &empty, &other),
			"closed four diag dec should be detected"
		);
	}
}
