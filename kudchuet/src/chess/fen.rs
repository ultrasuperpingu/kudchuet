
use crate::chess::*;
use crate::chess::mychess::ChessBoard;
use crate::common::bitboards::Bitboard8x8;


impl ChessBoard {
	
	pub fn from_fen(fen: &str) -> Result<Self, String> {
		let mut board = ChessBoard {
			whites: Bitboard8x8::empty(),
			blacks: Bitboard8x8::empty(),
			pawns: Bitboard8x8::empty(),
			rooks: Bitboard8x8::empty(),
			knights: Bitboard8x8::empty(),
			bishops: Bitboard8x8::empty(),
			queens: Bitboard8x8::empty(),
			kings: Bitboard8x8::empty(),
			turn: Color::White,
			castling_rights: CastlingRights::default(),
			ep_square: None,
			hash: 0,
		};

		let parts: Vec<&str> = fen.split_whitespace().collect();
		if parts.len() < 4 {
			return Err("Invalid FEN: incomplete FEN".into());
		}

		let mut rank = 7;
		let mut file = 0;

		for c in parts[0].chars() {
			match c {
				'/' => {
					if file != 8 {
						return Err("Invalid FEN: rank does not contain 8 squares".into());
					}
					rank -= 1;
					file = 0;
				}
				'1'..='8' => {
					file += c.to_digit(10).unwrap() as i8;
				}
				_ => {
					if file > 7 {
						return Err("Invalid FEN (too many columns)".into());
					}
					let idx = (rank * 8 + file) as usize;
					let bb = Bitboard8x8::from_index(idx);

					match c {
						'P' => { board.pawns |= bb; board.whites |= bb; }
						'N' => { board.knights |= bb; board.whites |= bb; }
						'B' => { board.bishops |= bb; board.whites |= bb; }
						'R' => { board.rooks |= bb; board.whites |= bb; }
						'Q' => { board.queens |= bb; board.whites |= bb; }
						'K' => { board.kings |= bb; board.whites |= bb; }

						'p' => { board.pawns |= bb; board.blacks |= bb; }
						'n' => { board.knights |= bb; board.blacks |= bb; }
						'b' => { board.bishops |= bb; board.blacks |= bb; }
						'r' => { board.rooks |= bb; board.blacks |= bb; }
						'q' => { board.queens |= bb; board.blacks |= bb; }
						'k' => { board.kings |= bb; board.blacks |= bb; }

						_ => return Err(format!("Invalid FEN (unknown character {})", c)),
					}

					file += 1;
				}
			}
		}
		if file != 8 {
			return Err("Invalid FEN: rank does not contain 8 squares".into());
		}

		if rank != 0 {
			return Err("Invalid FEN: board does not contains 8 ranks".into());
		}

		// --- 2) Trait ---
		board.turn = match parts[1] {
			"w" => Color::White,
			"b" => Color::Black,
			_ => return Err("Invalid FEN: trait invalid".into()),
		};

		// --- 3) Roques ---
		let mut rights = CastlingRights::default(); // = CastlingRights(0)

		// Si 'K' n'est pas présent → retirer roque blanc côté roi
		if !parts[2].contains('K') {
			rights.remove_white_kingside();
		}

		// Si 'Q' n'est pas présent → retirer roque blanc côté dame
		if !parts[2].contains('Q') {
			rights.remove_white_queenside();
		}

		// Si 'k' n'est pas présent → retirer roque noir côté roi
		if !parts[2].contains('k') {
			rights.remove_black_kingside();
		}

		// Si 'q' n'est pas présent → retirer roque noir côté dame
		if !parts[2].contains('q') {
			rights.remove_black_queenside();
		}
		board.castling_rights = rights;


		// --- 4) En passant ---
		board.ep_square = if parts[3] != "-" {
			let sq = parts[3];
			if sq.len() != 2 {
				return Err("Invalid FEN: En passant square is invalid".into());
			}
			let file = sq.as_bytes()[0] - b'a';
			let rank = sq.as_bytes()[1] - b'1';
			if file > 7 || rank > 7 {
				return Err("Invalid FEN: En passant square is out of bound".into());
			}
			Some(Square(rank * 8 + file))
		} else {
			None
		};

		// --- 5) Halfmove clock ---
		if parts.len() > 4 {
			let _half_counter = parts[4]
				.parse::<u8>()
				.map_err(|_| "Invalid FEN: incorrect halfmove clock".to_string())?;
		}

		// --- 6) Fullmove number ---
		if parts.len() > 5 {
			let _fullmoves = parts[5]
				.parse::<u16>()
				.map_err(|_| "Invalid FEN: incorrect fullmoves number".to_string())?;
		}
		board.hash = board.compute_zobrist();
		Ok(board)
	}
	pub fn to_fen(&self) -> String {
		let mut fen = String::new();

		for rank in (0..8).rev() {
			let mut empty = 0;

			for file in 0..8 {
				let idx = rank * 8 + file;
				let bb = Bitboard8x8::from_index(idx);

				let piece = if (self.pawns & bb).any() {
					Some('p')
				} else if (self.knights & bb).any() {
					Some('n')
				} else if (self.bishops & bb).any() {
					Some('b')
				} else if (self.rooks & bb).any() {
					Some('r')
				} else if (self.queens & bb).any() {
					Some('q')
				} else if (self.kings & bb).any() {
					Some('k')
				} else {
					None
				};

				match piece {
					Some(mut c) => {
						if (self.whites & bb).any() {
							c = c.to_ascii_uppercase();
						}
						if empty > 0 {
							fen.push_str(&empty.to_string());
							empty = 0;
						}
						fen.push(c);
					}
					None => {
						empty += 1;
					}
				}
			}

			if empty > 0 {
				fen.push_str(&empty.to_string());
			}

			if rank > 0 {
				fen.push('/');
			}
		}

		// Trait
		fen.push(' ');
		fen.push(match self.turn {
			Color::White => 'w',
			Color::Black => 'b',
		});

		// Roques
		fen.push(' ');
		let mut rights = String::new();
		if self.castling_rights.white_kingside() { rights.push('K'); }
		if self.castling_rights.white_queenside() { rights.push('Q'); }
		if self.castling_rights.black_kingside() { rights.push('k'); }
		if self.castling_rights.black_queenside() { rights.push('q'); }
		if self.castling_rights.none() { rights.push('-'); }
		fen.push_str(&rights);

		// En passant
		fen.push(' ');
		if let Some(Square(sq)) = self.ep_square {
			let file = sq % 8;
			let rank = sq / 8;
			fen.push((b'a' + file) as char);
			fen.push((b'1' + rank) as char);
		} else {
			fen.push('-');
		}

		// Halfmove clock + fullmove number
		fen.push_str(" 0 1");

		fen
	}

}

#[cfg(test)]
mod tests {
	use crate::chess::mychess::ChessBoard;

	#[test]
	fn test_fen() {
		pub const TEST_FENS: &[&str] = &[
			// 1. Initial
			"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",

			// 2. 1.e4
			"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",

			// 3. En passant
			"rnbqkbnr/pp1ppppp/8/3P4/8/8/PPP2PPP/RNBQKBNR w KQkq d6 0 3",

			// 4. Partial castling
			"rnbq1rk1/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQK2R w K - 5 5",

			// 5. Partial castling
			"r3k2r/pppq1ppp/2np4/4p3/4P3/2NP1N2/PPPQ1PPP/2KR1B1R b kq - 7 7",

			// 6. Kiwi
			"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",

			// 7. Promotion imminente
			"8/6P1/8/8/8/8/8/7K w - - 0 1",

			// 8. Mat du berger
			"rnbqkb1r/pppp1Qpp/5n2/4p3/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 0 3",

			// 9. Only kings
			"4k3/8/8/8/8/8/8/4K3 w - - 0 1",

			// 10. Stress test (illegal)
			"rnbqkbnr/pppppppp/pppppppp/pppppppp/PPPPPPPP/PPPPPPPP/pppppppp/RNBQKBNR w KQkq - 0 1",
		];
		for fen in TEST_FENS {
			let board = ChessBoard::from_fen(fen);
			
			let board = board.unwrap_or_else(|e| panic!("{} {:?}", fen, e));

			let fen_out = board.to_fen();
			assert_eq!(fen_out[..fen_out.len() - 4], fen[..fen.len() - 4]);
		}

		assert!(ChessBoard::from_fen("").is_err());
		// Missing square
		assert!(ChessBoard::from_fen("rnbqkbn/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err());
		// Missing line
		assert!(ChessBoard::from_fen("rnbqkbnr/pppppppp/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err());
		// Invalid trait
		assert!(ChessBoard::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR A KQkq - 0 1").is_err());
		// No optional fields
		ChessBoard::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -").unwrap_or_else(|e| panic!("{} {:?}", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -", e));
	}
}
