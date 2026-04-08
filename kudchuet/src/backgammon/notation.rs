use arrayvec::ArrayVec;

use crate::{backgammon::rules::{Backgammon, P1_BAR, P1_OUT, P2_BAR, P2_OUT, PlayerMove, SingleMove}, common::Player};

impl Backgammon {
	pub fn to_position_notation(&self) -> String {
		let board_str = self.board.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");
		let bar_str = format!("bar:{}-{}", self.on_bar[0], self.on_bar[1]);
		let off_str = format!("off:{}-{}", self.outside[0], self.outside[1]);
		let current = match self.current_player {
			Player::PLAYER1 => "P1",
			Player::PLAYER2 => "P2",
			_ => "RND",
		};
		let mut parts = vec![board_str, bar_str, off_str, format!("current:{}", current)];
		if !self.dice.is_empty() {
			let dice_str = self.dice.iter().map(|d| d.to_string()).collect::<Vec<_>>().join("-");
			parts.push(format!("dice:{}", dice_str));
		}
		parts.join("|")
	}

	pub fn from_position_notation(s: &str) -> Result<Self, String> {
		let mut parts = s.split('|');
		let mut game = Backgammon::default();

		// Board
		let board_str = parts.next().ok_or("Missing board part")?;
		let board_vals: Vec<&str> = board_str.split(',').collect();
		if board_vals.len() != 24 {
			return Err(format!("Board must have 24 values, got {}", board_vals.len()));
		}
		for (i, val) in board_vals.iter().enumerate() {
			game.board[i] = val.parse::<i8>().map_err(|_| format!("Invalid board value at {}: {}", i, val))?;
		}

		// Bar
		let bar_str = parts.next().ok_or("Missing bar part")?;
		let bar_vals = bar_str.strip_prefix("bar:").ok_or("Invalid bar prefix")?;
		let nums: Vec<&str> = bar_vals.split('-').collect();
		if nums.len() != 2 {
			return Err("Bar must have 2 values".to_string());
		}
		game.on_bar[0] = nums[0].parse().map_err(|_| "Invalid bar[0]")?;
		game.on_bar[1] = nums[1].parse().map_err(|_| "Invalid bar[1]")?;

		// Off
		let off_str = parts.next().ok_or("Missing off part")?;
		let off_vals = off_str.strip_prefix("off:").ok_or("Invalid off prefix")?;
		let nums: Vec<&str> = off_vals.split('-').collect();
		if nums.len() != 2 {
			return Err("Off must have 2 values".to_string());
		}
		game.outside[0] = nums[0].parse().map_err(|_| "Invalid outside[0]")?;
		game.outside[1] = nums[1].parse().map_err(|_| "Invalid outside[1]")?;

		// Current player
		let current_str = parts.next().ok_or("Missing current part")?;
		let cur = current_str.strip_prefix("current:").ok_or("Invalid current prefix")?;
		game.current_player = match cur {
			"P1" => Player::PLAYER1,
			"P2" => Player::PLAYER2,
			_ => return Err(format!("Invalid current player: {}", cur)),
		};

		// Dice
		if let Some(dice_str) = parts.next() {
			if let Some(values) = dice_str.strip_prefix("dice:") {
				game.dice = values
					.split('-')
					.map(|d| d.parse::<u8>().map_err(|_| format!("Invalid dice: {}", d)))
					.collect::<Result<ArrayVec<_, 4>, _>>()?;
			}
		}

		Ok(game)
	}
}
impl PlayerMove {
	pub fn to_notation(&self) -> Option<String> {
		let pmove = self;
		if pmove.len == 0 {
			return None;
		}

		let mut parts = Vec::new();
		for i in 0..pmove.len as usize {
			let SingleMove { from, to, captured: _ } = pmove.moves[i];
			let s = match (from, to) {
				(P1_BAR, to) | (P2_BAR, to) => format!("bar/{}", to),
				(from, P1_OUT) | (from, P2_OUT) => format!("{}/off", from),
				(from, to) => format!("{}/{}", from, to),
			};
			parts.push(s);
		}

		Some(parts.join(" "))
	}

	pub fn from_notation(m_str: &String) -> Result<Self, String> {
		let mut pmove = PlayerMove{ moves:[SingleMove::default();4] , len: 0 };

		for token in m_str.split_whitespace() {
			let sides: Vec<_> = token.split('/').collect();
			if sides.len() != 2 {
				return Err(format!("Invalid move token: {}", token));
			}

			let from = if sides[0] == "bar" {
				P1_BAR // à adapter selon le joueur courant si besoin
			} else {
				sides[0].parse::<u8>()
					.map_err(|_| format!("Invalid from: {}", sides[0]))?
			};

			let to = if sides[1] == "off" {
				if from == P1_BAR { P1_OUT } else { P2_OUT }
			} else {
				sides[1].parse::<u8>()
					.map_err(|_| format!("Invalid to: {}", sides[1]))?
			};

			pmove.push(SingleMove::new(from, to, false));
		}

		Ok(pmove)
	}
}

#[cfg(test)]
mod tests {
	use crate::backgammon::rules::Backgammon;

	#[test]
	fn test_position() {
		let game = Backgammon::new();
		let bpn = game.to_position_notation();
		println!("BPN: {}", bpn);

		let restored_game = Backgammon::from_position_notation(&bpn);
		assert_eq!(game, restored_game.unwrap());
		let game = Backgammon::from_position_notation("-4,0,0,0,0,0,0,0,0,0,0,0,5,0,0,0,0,0,0,0,0,0,0,0|bar:10-0|off:0-11|current:P1").unwrap();
		println!("{}", game);
		//-4,0,0,0,0,0,0,0,0,0,0,0,5,0,0,0,0,0,0,0,0,0,0,0|bar:10-0|off:0-11|current:P1
	}
}