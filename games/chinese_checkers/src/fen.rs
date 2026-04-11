
use crate::rules::{ChineseCheckers, ChineseCheckersPlayer};
impl ChineseCheckers {
	pub fn to_fen(&self) -> String {
		let mut fen = String::new();
		fen.push_str("turn=");
		fen.push(match self.turn {
			ChineseCheckersPlayer::Red => 'R',
			ChineseCheckersPlayer::Blue => 'B',
			ChineseCheckersPlayer::Green => 'G',
			ChineseCheckersPlayer::Yellow => 'Y',
			ChineseCheckersPlayer::Black => 'b',
			ChineseCheckersPlayer::White => 'w',
		});
		fen.push('\n');
		fen.push_str(format!("{}", self).as_str());
		fen
	}

	pub fn from_fen(fen: &str) -> Result<Self, String> {
		//let fen = fen.trim();
		let mut lines = fen.lines();
		let header = lines.next().ok_or("Missing header")?;

		let turn = header
			.strip_prefix("turn=")
			.ok_or("Invalid header")?;
		
		let turn_char = turn.chars().next().unwrap();
		
		let mut game = ChineseCheckers::empty();
		let mut colors=[false;6];
		for (y, line) in lines.enumerate() {
			if line.len() > 26 {
				return Err(format!("Invalid line length at y={}: {}", y, line.len()));
			}

			for (x, ch) in line.chars().enumerate() {
				match ch {
					' '|'_' => continue,
					'R' => {
						game.red.set(x as u8/2, y as u8);
						colors[0]=true;
					},
					'B' => {
						game.blue.set(x as u8/2, y as u8);
						colors[1]=true;
					},
					'G' => {
						game.green.set(x as u8/2, y as u8);
						colors[2]=true;
					},
					'Y' => {
						game.yellow.set(x as u8/2, y as u8);
						colors[3]=true;
					},
					'b' => {
						game.black.set(x as u8/2, y as u8);
						colors[4]=true;
					},
					'w' => {
						game.white.set(x as u8/2, y as u8);
						colors[5]=true;
					},
					'.' => {}
					_ => return Err(format!("Invalid FEN character: {}", ch)),
				}
			}
		}
		game.nb_players=colors.iter().filter(|e| **e).count() as u8;
		
		let turn_index = match turn_char {
			'R' => 0,
			'B' => 1,
			'G' => 2,
			'Y' => 3,
			'b' => 4,
			'w' => 5,
			_ => return Err(format!("Invalid FEN turn: {}", turn_char)),
		};

		if !colors[turn_index] {
			return Err("Turn player not present on board".to_string());
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
	use crate::rules::{ChineseCheckers, ChineseCheckersPlayer, Move};

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
	fn test_fen_roundtrip_complex() {
		let mut game = ChineseCheckers::new(4);

		let mut moves = Vec::new();

		for player in [
			ChineseCheckersPlayer::Red,
			ChineseCheckersPlayer::Blue,
			ChineseCheckersPlayer::Green,
			ChineseCheckersPlayer::Yellow,
		] {
			let mut legal_moves=vec![];
			game.generate_moves_for_player(player, &mut legal_moves);
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