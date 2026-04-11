use crate::{bitboard::BitboardAbalone, rules::{Abalone, Hex, idx}};
use kudchuet::Player;

impl std::fmt::Display for Abalone {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Turn: {:?}", self.turn)?;
		writeln!(f, "Black out: {}, White out: {}", self.black_out, self.white_out)?;
		writeln!(f)?;

		// Le plateau Abalone standard : q,r ∈ [-4..4] avec q+r+s=0
		// On affiche par rangées de r, de haut en bas (r = -4 → +4)
		for r in -4..=4 {
			// indentation pour l'effet hexagonal
			let indent = if r > 0 { (r + 4i8).unsigned_abs() as usize} else {(r - 4i8).unsigned_abs() as usize};
			write!(f, "{:indent$}", "", indent = indent)?;

			for q in -4..=4 {
				let s = -q - r;
				if !(-4..=4).contains(&s) {
					continue; // hors plateau
				}

				let hex = Hex { q, r };
				let bit = 1u64 << idx(hex).unwrap();

				let c = if (self.black.storage() & bit) != 0 {
					'●'
				} else if (self.white.storage() & bit) != 0 {
					'○'
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


impl Abalone {
	pub fn to_fen(&self) -> String {
		let mut fen = String::with_capacity(80);

		for r in -4..=4 {
			for q in -4..=4 {
				let s = -q - r;
				if !(-4..=4).contains(&s) {
					continue;
				}

				let hex = Hex { q, r };
				let idx = idx(hex).unwrap();

				let c = if self.black.get_at_index(idx) {
					'X'
				} else if self.white.get_at_index(idx) {
					'O'
				} else {
					'.'
				};

				fen.push(c);
			}
			fen.push('/');
		}

		fen.pop(); // last '/'

		fen.push(' ');
		fen.push(if self.turn == Player::PLAYER1 { 'b' } else { 'w' });

		fen.push(' ');
		fen.push_str(&self.black_out.to_string());

		fen.push(' ');
		fen.push_str(&self.white_out.to_string());

		fen
	}
	pub fn from_fen(fen: &str) -> Result<Self, String> {
		let mut parts = fen.split_whitespace();

		let board_part = parts.next().ok_or("Missing board in FEN")?;
		let turn_part = parts.next().ok_or("Missing turn in FEN")?;
		let black_out_part = parts.next().ok_or("Missing black_out in FEN")?;
		let white_out_part = parts.next().ok_or("Missing white_out in FEN")?;

		let turn = match turn_part {
			"b" => Player::PLAYER1,
			"w" => Player::PLAYER2,
			_ => return Err(format!("Invalid turn '{}'", turn_part)),
		};

		let black_out: u8 = black_out_part
			.parse()
			.map_err(|_| format!("Invalid black_out '{}'", black_out_part))?;

		let white_out: u8 = white_out_part
			.parse()
			.map_err(|_| format!("Invalid white_out '{}'", white_out_part))?;

		let mut black = BitboardAbalone::empty();
		let mut white = BitboardAbalone::empty();

		let rows: Vec<&str> = board_part.split('/').collect();
		if rows.len() != 9 {
			return Err(format!("FEN must have 9 rows, got {}", rows.len()));
		}

		for (ri, row) in rows.iter().enumerate() {
			let r = ri as i8 - 4;
			let expected_len = 9 - r.unsigned_abs() as usize;

			if row.len() != expected_len {
				return Err(format!(
					"Row {} has length {}, expected {}",
					ri, row.len(), expected_len
				));
			}

			let q_start = (-4).max(-r - 4);

			for (i, c) in row.chars().enumerate() {
				let q = q_start + i as i8;
				let hex = Hex { q, r };

				let idx = idx(hex).ok_or(format!("Invalid hex {:?}", hex))?;

				match c.to_ascii_lowercase() {
					'x' | 'b' => black.set_at_index(idx),
					'o' | 'w' => white.set_at_index(idx),
					'.' => {}
					_ => return Err(format!("Invalid character '{}' in FEN", c)),
				}
			}
		}

		Ok(Abalone {
			black,
			white,
			turn,
			black_out,
			white_out,
		})
	}

}

#[test]
fn test(){
	let a = Abalone::new_standard();
	let fen = a.to_fen();
	println!("{}", fen);
	let a2 = Abalone::from_fen(&fen);
	println!("{}", a2.unwrap());
}