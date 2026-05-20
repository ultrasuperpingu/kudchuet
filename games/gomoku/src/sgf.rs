use crate::rules::Gomoku;
use bitboard::common_bitboards::Goban;
use bitboard::BitIterRef;
use kudchuet::Player;

pub fn serialize_sgf(game: &Gomoku) -> String {
	fn coord_to_sgf(x: u8, y: u8) -> String {
		let cx = (b'a' + x) as char;
		let cy = (b'a' + y) as char;
		format!("{}{}", cx, cy)
	}

	let mut sgf = String::from("(;");

	// Size
	sgf.push_str("SZ[19]");

	// Black
	if !game.black.is_empty() {
		sgf.push_str("AB");
		for index in game.black.iter_bits_ref() {
			let (x, y) = Goban::coords_from_index(index as usize);
			sgf.push_str(&format!("[{}]", coord_to_sgf(x, y)));
		}
	}

	// White
	if !game.white.is_empty() {
		sgf.push_str("AW");
		for index in game.white.iter_bits_ref() {
			let (x, y) = Goban::coords_from_index(index as usize);
			sgf.push_str(&format!("[{}]", coord_to_sgf(x, y)));
		}
	}

	// Player on turn
	sgf.push_str(&format!(
		"PL[{}]",
		match game.turn {
			Player::PLAYER1 => "B",
			Player::PLAYER2 => "W",
			_ => unreachable!(),
		}
	));

	sgf.push(')');
	sgf
}
pub fn parse_sgf(input: &str) -> Result<Gomoku, String> {
	use regex::Regex;

	// Property SGF : AB[aa][bb], AW[cc], PL[B], PL[W], etc.
	let re_prop = Regex::new(r"([A-Z]+)((\[[A-Za-z]+\])+)").unwrap();

	// SGF Coord : [xy]
	let re_val = Regex::new(r"\[([a-z])([a-z])\]").unwrap();

	let mut black = Goban::new();
	let mut white = Goban::new();
	let mut turn = Player::PLAYER1;

	for cap in re_prop.captures_iter(input) {
		let prop = &cap[1];
		let values = &cap[2];

		match prop {
			"AB" => {
				for v in re_val.captures_iter(values) {
					let x = v[1].as_bytes()[0] - b'a';
					let y = v[2].as_bytes()[0] - b'a';
					black.set(x, y);
				}
			}
			"AW" => {
				for v in re_val.captures_iter(values) {
					let x = v[1].as_bytes()[0] - b'a';
					let y = v[2].as_bytes()[0] - b'a';
					white.set(x, y);
				}
			}
			"PL" => {
				if values.contains("[B]") {
					turn = Player::PLAYER1;
				} else if values.contains("[W]") {
					turn = Player::PLAYER2;
				}
			}
			_ => {}
		}
	}

	let mut res = Gomoku {
		white,
		black,
		turn,
		hash: 0,
	};

	res.hash = res.compute_zobrist();
	Ok(res)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::rules::Gomoku;
	use kudchuet::Player;

	#[test]
	fn test_sgf_roundtrip() {
		let mut game = Gomoku {
			white: Goban::new(),
			black: Goban::new(),
			turn: Player::PLAYER2,
			hash: 0,
		};

		game.black.set(3, 3); // d4
		game.black.set(4, 4); // e5
		game.white.set(2, 5); // c6

		game.hash = game.compute_zobrist();

		let sgf = serialize_sgf(&game);

		assert!(sgf.contains("AB[dd][ee]"));
		assert!(sgf.contains("AW[cf]"));
		assert!(sgf.contains("PL[W]"));

		let parsed = parse_sgf(&sgf).expect("SGF parsing failed");

		assert_eq!(parsed.turn, game.turn);
		assert_eq!(parsed.black, game.black);
		assert_eq!(parsed.white, game.white);

		assert_eq!(parsed.hash, parsed.compute_zobrist());

		let sgf2 = serialize_sgf(&parsed);
		assert_eq!(sgf, sgf2);
	}
}
