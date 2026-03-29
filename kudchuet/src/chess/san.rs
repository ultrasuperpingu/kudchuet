use crate::chess::*;
use crate::chess::mychess::ChessBoard;

impl ChessBoard {
	pub fn find_castle(&self, kingside: bool, legal: &Vec<Move>) -> Result<Move, String> {

		let king_dest = match (self.turn, kingside) {
			(Color::White, true)  => Square::from_coords(6, 0), // g1
			(Color::White, false) => Square::from_coords(2, 0), // c1
			(Color::Black, true)  => Square::from_coords(6, 7), // g8
			(Color::Black, false) => Square::from_coords(2, 7), // c8
		};

		for mv in legal {
			if mv.to == king_dest {
				if self.piece_at(mv.from) == Some(Piece::King) {
					return Ok(*mv);
				}
			}
		}

		Err("No legal castling move found".into())
	}

	pub fn san_to_move(&self, san: &str) -> Result<Move, String> {
		let san = san.trim();

		let legal_moves = self.legal_moves();

		if san == "O-O" || san == "0-0" {
			return self.find_castle(true, &legal_moves);
		}
		if san == "O-O-O" || san == "0-0-0" {
			return self.find_castle(false, &legal_moves);
		}

		// 2. Promotion
		let (san, promotion) = if let Some(eq) = san.find('=') {
			let promo = match san[eq+1..].chars().next().unwrap() {
				'Q' => Piece::Queen,
				'R' => Piece::Rook,
				'B' => Piece::Bishop,
				'N' => Piece::Knight,
				_ => return Err("Invalid promotion".into()),
			};
			(&san[..eq], Some(promo))
		} else {
			(san, None)
		};

		// 3. Capture ?
		let is_capture = san.contains('x');

		// 4. Pièce
		let mut chars = san.chars();
		let first = chars.next().unwrap();

		let (piece, rest) = match first {
			'N' => (Piece::Knight, &san[1..]),
			'B' => (Piece::Bishop, &san[1..]),
			'R' => (Piece::Rook,   &san[1..]),
			'Q' => (Piece::Queen,  &san[1..]),
			'K' => (Piece::King,   &san[1..]),
			'a'..='h' => (Piece::Pawn, san),
			_ => return Err("Invalid SAN: invalid first char".into()),
		};

		// 5. Case d’arrivée = les deux derniers chars
		let to = Square::from_uci(&rest[rest.len()-2..])
			.ok_or("Invalid destination square")?;

		// 6. Désambiguïsation éventuelle
		let mut dis_file = None;
		let mut dis_rank = None;

		let middle = &rest[..rest.len()-2];
		for c in middle.chars() {
			if ('a'..='h').contains(&c) { dis_file = Some(c); }
			if ('1'..='8').contains(&c) { dis_rank = Some(c); }
		}

		// 7. Trouver le coup légal correspondant
		let maybe_moves: Vec<&Move> = legal_moves.iter().filter(|m| {
			let b= m.to == to && m.promotion == promotion && self.piece_at(m.from).unwrap() == piece && self.color_at(m.from).unwrap() == self.turn;
			if !b { return false; }
			if let Some(f) = dis_file {
				if m.from.file() != (f as u8 - b'a') as usize { return false; }
			}
			if let Some(r) = dis_rank {
				if m.from.rank() != (r as u8 - b'1') as usize { return false; }
			}
			true
		}).collect();
		if maybe_moves.len() == 1 {
			let is_real_capture = self.color_at(to) == Some(self.turn.opponent()) || Some(to) == self.ep_square;
			if is_capture && !is_real_capture {
				return Err("Move specify a capture but the only move matching the SAN is not a capture".into());
			} // If we want to refuse not specified capture, it should be done in else. But we tolerate it
			
			return Ok(*maybe_moves[0]);
		}
		if maybe_moves.len() > 1 {
			return Err("Ambiguous SAN: multiple legal moves match the notation".into());
		}
		Err("No matching legal move".into())
	}

	pub fn move_to_san(&self, mv: &Move) -> Result<String, String> {
		// Castling
		let piece = self.piece_at(mv.from);
		if piece.is_none() {
			return Err("Invalid move for this state".into());
		}
		let piece = piece.unwrap();
		if piece == Piece::King {
			let diff = mv.to.file() as i8 - mv.from.file() as i8;
			if diff == 2 { return Ok("O-O".to_string()); }
			if diff == -2 { return Ok("O-O-O".to_string()); }
		}

		let mut san = String::new();

		// Piece type
		if piece != Piece::Pawn {
			san.push(match piece {
				Piece::Knight => 'N',
				Piece::Bishop => 'B',
				Piece::Rook   => 'R',
				Piece::Queen  => 'Q',
				Piece::King   => 'K',
				_ => unreachable!(),
			});

			// 3. Dissambigation
			let legal_moves = self.legal_moves();
			let ambiguous_moves: Vec<&Move> = legal_moves.iter()
				.filter(|m| m.to == mv.to && m.from != mv.from && self.piece_at(m.from) == Some(piece))
				.collect();

			if !ambiguous_moves.is_empty() {
				let same_file = ambiguous_moves.iter().any(|m| m.from.file() == mv.from.file());
				let same_rank = ambiguous_moves.iter().any(|m| m.from.rank() == mv.from.rank());

				if !same_file {
					san.push((b'a' + mv.from.file() as u8) as char);
				} else if !same_rank {
					san.push((b'1' + mv.from.rank() as u8) as char);
				} else {
					san.push((b'a' + mv.from.file() as u8) as char);
					san.push((b'1' + mv.from.rank() as u8) as char);
				}
			}
		} else {
			// pawn capure
			if self.is_capture(mv) {
				san.push((b'a' + mv.from.file() as u8) as char);
			}
		}

		// capture
		if self.is_capture(mv) {
			san.push('x');
		}

		// destination square
		san.push_str(&mv.to.to_string());

		// promotion
		if let Some(promo) = mv.promotion {
			san.push('=');
			san.push(match promo {
				Piece::Queen  => 'Q',
				Piece::Rook   => 'R',
				Piece::Bishop => 'B',
				Piece::Knight => 'N',
				_ => unreachable!(),
			});
		}

		// check or checkmate
		let mut next_board = self.clone();
		next_board.play(mv);
		let status = next_board.status();
		if status.is_player1() || status.is_player2() {
			san.push('#');
		} else if next_board.is_in_check(next_board.turn) {
			san.push('+');
		}

		Ok(san)
	}

	// Helper pour détecter les captures (incluant En Passant)
	fn is_capture(&self, mv: &Move) -> bool {
		self.color_at(mv.to).is_some() || 
		(self.piece_at(mv.from) == Some(Piece::Pawn) && Some(mv.to) == self.ep_square)
	}

}

#[cfg(test)]
mod tests {
	use crate::chess::*;
	use crate::chess::mychess::ChessBoard;

	#[test]
	fn test_san_to_move() {
		// Position initiale
		let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
		let board = ChessBoard::from_fen(fen).unwrap();

		// 1. Coup simple : e4
		let mv = board.san_to_move("e4").expect("SAN e4 should parse");
		assert_eq!(mv.from, Square::from_coords(4, 1)); // e2
		assert_eq!(mv.to,   Square::from_coords(4, 3)); // e4

		// 2. Coup de cavalier : Nf3
		let mv = board.san_to_move("Nf3").expect("SAN Nf3 should parse");
		assert_eq!(mv.from, Square::from_coords(6, 0)); // g1
		assert_eq!(mv.to,   Square::from_coords(5, 2)); // f3

		// 3. En passant possible après e4 d5 exd5
		let fen2 = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
		let board2 = ChessBoard::from_fen(fen2).unwrap();

		let mv = board2.san_to_move("exd5").expect("SAN exd5 should parse");
		assert_eq!(mv.from, Square::from_coords(4, 3)); // e4
		assert_eq!(mv.to,   Square::from_coords(3, 4)); // d5

		// 4. Promotion
		let fen3 = "8/6P1/8/8/8/8/8/7K w - - 0 1";
		let board3 = ChessBoard::from_fen(fen3).unwrap();

		let mv = board3.san_to_move("g8=Q").expect("SAN g8=Q should parse");
		assert_eq!(mv.from, Square::from_coords(6, 6)); // g7
		assert_eq!(mv.to,   Square::from_coords(6, 7)); // g8
		assert_eq!(mv.promotion, Some(Piece::Queen));

		// 5. Roque roi
		let fen4 = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
		let board4 = ChessBoard::from_fen(fen4).unwrap();

		let mv = board4.san_to_move("O-O").expect("SAN O-O should parse");
		assert_eq!(mv.to, Square::from_coords(6, 0)); // g1

		// 6. Roque dame
		let mv = board4.san_to_move("O-O-O").expect("SAN O-O-O should parse");
		assert_eq!(mv.to, Square::from_coords(2, 0)); // c1

		// 7. Désambiguïsation : Nbd2
		let fen5 = "rnbqkbnr/pppppppp/8/8/8/1N6/PPP1PPPP/RNBQKBNR w KQkq - 0 1";
		let board5 = ChessBoard::from_fen(fen5).unwrap();

		let mv = board5.san_to_move("Nbd2").expect("SAN Nbd2 should parse");
		assert_eq!(mv.from, Square::from_coords(1, 0)); // b1
		assert_eq!(mv.to,   Square::from_coords(3, 1)); // d2

		// 8. Désambiguïsation par colonne : Rad1
		let fen6 = "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1";
		let board6 = ChessBoard::from_fen(fen6).unwrap();

		let mv = board6.san_to_move("Rad1").expect("SAN Rad1 should parse");
		assert_eq!(mv.from, Square::from_coords(0, 0)); // a1
		assert_eq!(mv.to,   Square::from_coords(3, 0)); // e1

		let mv = board6.san_to_move("a1a7").expect("UCI a1a7 should parse");
		assert_eq!(mv.from, Square::from_coords(0, 0)); // a1
		assert_eq!(mv.to,   Square::from_coords(0, 6)); // a7
	}

}
