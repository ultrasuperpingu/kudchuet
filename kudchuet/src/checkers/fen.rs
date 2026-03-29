use crate::{checkers::rules::Checkers10, common::Player};

impl Checkers10 {
	/// Parse the FEN representation and return the corresponding board
	pub fn from_fen(fen: &str) -> Result<Self, String> {
		let mut board = Self::empty();
		let parts: Vec<&str> = fen.split(':').collect();
		
		if parts.len() < 3 { 
			return Err("Invalid format. Expected Trait:Whites:Blacks".to_string()); 
		}

		board.current_player = match parts[0].to_uppercase().as_str() {
			"W" => Player::Player1,
			"B" => Player::Player2,
			_ => return Err(format!("Invalid trait : {}", parts[0])),
		};

		for part in &parts[1..] {
			let mut chars = part.chars();
			let color_char = chars.next().ok_or("Empty piece section")?;
			let pieces_str = chars.as_str();
			
			if pieces_str.is_empty() { continue; }

			for piece_code in pieces_str.split(',') {
				if piece_code.is_empty() { continue; }
				
				let is_queen = piece_code.starts_with('K');
				let num_str = if is_queen { &piece_code[1..] } else { piece_code };
				
				match num_str.parse::<u8>() {
					Ok(num) if num >= 1 && num <= 50 => {
						let idx = (num - 1) as usize;
						match color_char {
							'W' | 'w' => {
								if is_queen { board.white_queens.set_at_index(idx); }
								else { board.white_pawns.set_at_index(idx); }
							},
							'B' | 'b' => {
								if is_queen { board.black_queens.set_at_index(idx); }
								else { board.black_pawns.set_at_index(idx); }
							},
							_ => return Err(format!("Unknown color: {}", color_char)),
						}
					},
					_ => return Err(format!("Invalid square number: {}", num_str)),
				}
			}
		}
		Ok(board)
	}

	/// Get the FEN reprensentation of the board
	pub fn to_fen(&self) -> String {
		let mut fen = String::new();

		fen.push_str(if self.current_player == Player::Player1 { "W:" } else { "B:" });

		fen.push('W');
		let mut w_pieces = Vec::new();
		for i in 0..50 {
			if self.white_queens.get_at_index(i) { w_pieces.push(format!("K{}", i + 1)); }
			else if self.white_pawns.get_at_index(i) { w_pieces.push(format!("{}", i + 1)); }
		}
		fen.push_str(&w_pieces.join(","));

		fen.push_str(":B");
		let mut b_pieces = Vec::new();
		for i in 0..50 {
			if self.black_queens.get_at_index(i) { b_pieces.push(format!("K{}", i + 1)); }
			else if self.black_pawns.get_at_index(i) { b_pieces.push(format!("{}", i + 1)); }
		}
		fen.push_str(&b_pieces.join(","));

		fen
	}
}
#[cfg(test)]
mod tests {
	use crate::checkers::rules::Checkers10;

	#[test]
	fn test_roundtrip() {
		let b = Checkers10::from_fen("W:W31,32,33,34,35:B12,13,14,15,20");
		assert_eq!(b.unwrap().to_fen(), "W:W31,32,33,34,35:B12,13,14,15,20");
	}
}