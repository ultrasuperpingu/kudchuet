use crate::chinese_checkers::{ChineseCheckers, ChineseCheckersPlayer, bitboard::ChineseCheckerBoard};
impl ChineseCheckers {
	pub fn to_fen(&self) -> String {
		let mut fen = String::new();

		for y in 0..17 {
			if y > 0 { fen.push('\n'); }

			for x in 0..13 {
				if !ChineseCheckerBoard::is_playable(x, y) {
					fen.push(' '); 
					continue;
				}

				let c = if self.red.get(x, y) { 'R' }
					else if self.blue.get(x, y) { 'B' }
					else if self.green.get(x, y) { 'G' }
					else if self.yellow.get(x, y) { 'Y' }
					else if self.black.get(x, y) { 'b' }
					else if self.white.get(x, y) { 'w' }
					else { '.' };

				fen.push(c);
			}
		}

		// encode le joueur actif + nb_players
		fen.push(' ');
		fen.push(match self.turn {
			ChineseCheckersPlayer::Red => 'R',
			ChineseCheckersPlayer::Blue => 'B',
			ChineseCheckersPlayer::Green => 'G',
			ChineseCheckersPlayer::Yellow => 'Y',
			ChineseCheckersPlayer::Black => 'b',
			ChineseCheckersPlayer::White => 'w',
		});
		fen.push_str(&self.nb_players.to_string());

		fen
	}

	pub fn from_fen(fen: &str) -> Result<Self, String> {
		//let fen = fen.trim();

		let mut parts = fen.rsplitn(2, ' ');
		let turn_nb = parts.next().ok_or("Missing turn/nb_players")?;
		let board_part = parts.next().ok_or("Missing board")?;

		if turn_nb.len() < 2 {
			return Err("Invalid turn/nb_players format".to_string());
		}

		let turn_char = turn_nb.chars().next().unwrap();
		let nb_players: u8 = turn_nb[1..].parse().map_err(|_| "Invalid nb_players")?;

		let mut game = ChineseCheckers::empty();
		game.nb_players=nb_players;

		for (y, line) in board_part.lines().enumerate() {
			if line.len() != 13 {
				return Err(format!("Invalid line length at y={}: {}", y, line.len()));
			}

			for (x, ch) in line.chars().enumerate() {
				match ch {
					' '|'_' => continue,
					'R' => game.red.set(x as u8, y as u8),
					'B' => game.blue.set(x as u8, y as u8),
					'G' => game.green.set(x as u8, y as u8),
					'Y' => game.yellow.set(x as u8, y as u8),
					'b' => game.black.set(x as u8, y as u8),
					'w' => game.white.set(x as u8, y as u8),
					'.' => {}
					_ => return Err(format!("Invalid FEN character: {}", ch)),
				}
			}
		}

		game.turn = match turn_char {
			'R' => ChineseCheckersPlayer::Red,
			'B' => ChineseCheckersPlayer::Blue,
			'G' => ChineseCheckersPlayer::Green,
			'Y' => ChineseCheckersPlayer::Yellow,
			'b' => ChineseCheckersPlayer::Black,
			'w' => ChineseCheckersPlayer::White,
			_ => return Err(format!("Invalid FEN turn: {}", turn_char)),
		};

		Ok(game)
	}
}
#[cfg(test)]
mod tests {
	use crate::chinese_checkers::{ChineseCheckers, ChineseCheckersPlayer, Move};

	#[test]
	fn test_fen_roundtrip() {
		let game = ChineseCheckers::new(2);
		let fen = game.to_fen();
		let game2 = ChineseCheckers::from_fen(&fen).unwrap();
		assert_eq!(game.red, game2.red);
		assert_eq!(game.blue, game2.blue);
		assert_eq!(game.turn, game2.turn);
	}
	#[test]
	fn test_fen_roundtrip_complex_valid() {
		let mut game = ChineseCheckers::new(4);

		let mut moves = Vec::new();

		for player in [
			ChineseCheckersPlayer::Red,
			ChineseCheckersPlayer::Blue,
			ChineseCheckersPlayer::Green,
			ChineseCheckersPlayer::Yellow,
		] {
			let legal_moves = game.generate_moves_for_player(player);
			let mv = *legal_moves.first().expect("No legal moves");

			moves.push((player, mv.from, mv.to));
		}

		for (player, from, to) in &moves {
			game.play_unchecked_for_player(*player, Move { from: *from, to: *to });
		}

		let fen = game.to_fen();
		println!("Fen:\n{}", fen);
		let game2 = ChineseCheckers::from_fen(&fen).unwrap();

		assert_eq!(game.red, game2.red);
		assert_eq!(game.blue, game2.blue);
		assert_eq!(game.green, game2.green);
		assert_eq!(game.yellow, game2.yellow);
		assert_eq!(game.black, game2.black);
		assert_eq!(game.white, game2.white);
		assert_eq!(game.turn, game2.turn);

		println!("Fen:\n{}", fen);
	}
}