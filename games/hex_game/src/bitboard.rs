use bitboard_proc_macro::bitboard;
use bitboard_proc_macro::BitboardDebug;

#[bitboard(width = 11, height = 11)]
#[derive(BitboardDebug, Default)]
pub struct HexBoard;

const fn neighbors(index: usize) -> HexBoard {
	let (x, y) = HexBoard::coords_from_index(index);
	let mut board = HexBoard::empty();
	if x < HexBoard::WIDTH - 1 {
		board.set_at_index(HexBoard::index_from_coords(x + 1, y));
	}
	if x > 0 {
		board.set_at_index(HexBoard::index_from_coords(x - 1, y));
	}
	if y < HexBoard::HEIGHT - 1 {
		if x > 0 {
			board.set_at_index(HexBoard::index_from_coords(x - 1, y + 1));
		}
		board.set_at_index(HexBoard::index_from_coords(x, y + 1));
	}
	if y > 0 {
		if x < HexBoard::WIDTH - 1 {
			board.set_at_index(HexBoard::index_from_coords(x + 1, y - 1));
		}
		board.set_at_index(HexBoard::index_from_coords(x, y - 1));
	}
	board
}
const fn generate_neighbors_table() -> [HexBoard; HexBoard::NB_SQUARES] {
	let mut table = [HexBoard::empty(); HexBoard::NB_SQUARES];
	let mut i = 0;
	while i < HexBoard::NB_SQUARES {
		table[i] = neighbors(i);
		i += 1;
	}
	table
}
pub static NEIGHBORS: [HexBoard; HexBoard::NB_SQUARES] = generate_neighbors_table();

impl core::fmt::Display for HexBoard {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "   ")?;
		for x in 0..HexBoard::WIDTH {
			write!(f, "{} ", (b'A' + x) as char)?;
		}
		writeln!(f)?;

		for y in 0..HexBoard::HEIGHT {
			write!(f, "{:2} ", y + 1)?;

			for _ in 0..y {
				write!(f, " ")?;
			}

			for x in 0..HexBoard::WIDTH {
				let idx = HexBoard::index_from_coords(x, y);

				let c = if self.get_at_index(idx) { '●' } else { '.' };

				write!(f, "{} ", c)?;
			}

			writeln!(f)?;
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::bitboard::NEIGHBORS;

	#[test]
	fn test_neighbors() {
		for (i, n) in NEIGHBORS.iter().enumerate() {
			println!("{}:\n{}", i, n);
		}
	}
}
