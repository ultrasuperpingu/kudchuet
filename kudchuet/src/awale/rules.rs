use std::fmt::Display;

use crate::common::Player;

#[derive(Clone, Hash, Debug)]
pub struct Awale {
	pub(crate) pits: [u8; 12],
	pub(crate) score_bottom: u8,
	pub(crate) score_top: u8,
	pub(crate) turn: Player,
	pub(crate) game_over: bool,
}
impl Default for Awale {
	fn default() -> Self {
		Self {
			pits: [4; 12],
			score_bottom: 0,
			score_top: 0,
			turn: Player::Player1,
			game_over: false,
		}
	}
}
impl Display for Awale {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "+-----------------------+\n|  |")?;
		for pit in 6..12 {
			write!(f, "{:>2}|", &self.pits[12-pit+5])?;
		}
		write!(f, "  |\n+{:>2}+--+--+--+--+--+--+{:>2}+\n|  ", self.score_top, self.score_bottom)?;
		for pit in 0..6 {
			write!(f, "|{:>2}", &self.pits[pit])?;
		}
		write!(f, "|  |\n+-----------------------+\n")
	}
}
impl Awale {
	fn pits_of(&self, p: Player) -> std::ops::Range<usize> {
		match p {
			Player::Player1 => 0..6,
			Player::Player2 => 6..12,
			_ => unreachable!(),
		}
	}

	fn is_own_pit(&self, pit: usize, p: Player) -> bool {
		self.pits_of(p).contains(&pit)
	}
	pub fn pit(&self, pit: usize) -> u8 {
		self.pits[pit]
	}
}
impl Awale {
	fn opponent_has_seeds(&self) -> bool {
		self.pits_of(self.turn.opponent())
			.any(|i| self.pits[i] > 0)
	}

	fn would_feed_opponent(&self, pit: usize) -> bool {
		let seeds = self.pits[pit];
		if seeds == 0 {
			return false;
		}

		let mut pos = pit;
		for _ in 0..seeds {
			pos = (pos + 1) % 12;
			if self.is_own_pit(pos, self.turn.opponent()) {
				return true;
			}
		}
		false
	}

	fn is_legal_move(&self, pit: usize) -> bool {
		if !self.is_own_pit(pit, self.turn) {
			//println!("Not own pit");
			return false;
		}
		if self.pits[pit] == 0 {
			//println!("Empty pit");
			return false;
		}

		// Si l’adversaire a des graines, on doit le nourrir
		if !self.opponent_has_seeds() {
			//println!("Need to feed");
			self.would_feed_opponent(pit)
		} else {
			let nb = self.pits[pit];
			let last = (nb + pit as u8) % 12 + nb / 11;
			let opponent_pits = self.pits_of(self.turn.opponent());
			let mut captured_pits = self.pits_of(self.turn.opponent()).filter(|pit| *pit <= last as usize);
			let mut not_captured_pits = self.pits_of(self.turn.opponent()).filter(|pit| *pit > last as usize);
			if opponent_pits.contains(&(last as usize)) && captured_pits.all(|i| self.pits[i] < 3) && not_captured_pits.all(|i| self.pits[i] == 0) {
				// Capturer toutes les graines adverses est illégal
				return false;
			}
				
			// Sinon, tous les coups sont légaux
			true
		}
	}

	pub fn legal_moves(&self) -> Vec<usize> {
		let mut moves = vec![];
		for pit in self.pits_of(self.turn) {
			if self.is_legal_move(pit) {
				moves.push(pit);
			}
		}
		moves
	}
}
impl Awale {
	fn sow(&mut self, pit: usize) -> usize {
		let mut seeds = self.pits[pit];
		self.pits[pit] = 0;

		let mut pos = pit;
		while seeds > 0 {
			pos = (pos + 1) % 12;
			if pos == pit {
				continue; // on saute le trou d’origine
			}
			self.pits[pos] += 1;
			seeds -= 1;
		}

		pos // dernier trou
	}
}
impl Awale {
	fn capture(&mut self, last: usize) {
		let opp = self.turn.opponent();
		let range = self.pits_of(opp);

		let mut pos = last;
		while range.contains(&pos) {
			let seeds = self.pits[pos];
			if seeds == 2 || seeds == 3 {
				self.pits[pos] = 0;
				match self.turn {
					Player::Player1 => self.score_bottom += seeds,
					Player::Player2 => self.score_top += seeds,
					_ => unreachable!(),
				}
			} else {
				break;
			}
			if pos == 0 {
				break;
			}
			pos = pos.wrapping_sub(1);
		}
	}
}
impl Awale {
	fn check_game_over(&mut self) {
		if self.score_bottom >= 25 || self.score_top >= 25 {
			self.game_over = true;
			return;
		}

		// Si le prochain joueur n’a plus de graines
		if self.turn == Player::Player2 {
			let bottom_empty = (0..6).all(|i| self.pits[i] == 0);
			if bottom_empty {
				// On ramasse tout ce qui reste
				self.score_top += (6..12).map(|i| self.pits[i]).sum::<u8>();
				self.pits = [0; 12];
				self.game_over = true;
			}
		} else {
			let top_empty = (6..12).all(|i| self.pits[i] == 0);
			if top_empty {
				// On ramasse tout ce qui reste
				self.score_bottom += (0..6).map(|i| self.pits[i]).sum::<u8>();
				self.pits = [0; 12];
				self.game_over = true;
			}
		}
	}
	pub fn is_over(&self) -> bool {
		self.game_over
	}
	pub fn is_draw(&self) -> bool {
		self.game_over && self.score_bottom == self.score_top
	}
	pub fn winner(&self) -> Option<Player> {
		if self.game_over {
			if self.score_bottom > self.score_top {
				Some(Player::Player1)
			} else if self.score_bottom < self.score_top {
				Some(Player::Player2)
			} else {
				None
			}
		} else {
			None
		}
	}
}
impl Awale {
	pub fn play(&mut self, pit: usize) -> bool {
		if self.game_over {
			return false;
		}
		if !self.is_legal_move(pit) {
			return false;
		}

		self.play_unchecked(pit);

		true
	}

	pub fn play_unchecked(&mut self, pit: usize) {
		let last = self.sow(pit);
		self.capture(last);
		self.check_game_over();

		if !self.game_over {
			self.turn = self.turn.opponent();
		}
	}
}
#[cfg(test)]
mod tests {
	use super::super::rules::{Awale, Player};

	#[test]
	fn test_awale_init() {
		let awale = Awale::default();
		println!("{}", awale);
		assert!(awale.pits == [4,4,4,4,4,4,4,4,4,4,4,4]);
		assert!(awale.score_bottom == 0);
		assert!(awale.score_top == 0);
		assert!(awale.turn == Player::Player1);
		assert!(awale.game_over == false);
	}
	#[test]
	fn test_awale_legal_moves() {
		let mut awale = Awale::default();
		// not own pit
		assert!(awale.play(11) == false);
		assert!(awale.play(5));
		assert!(awale.pits == [4,4,4,4,4,0,5,5,5,5,4,4]);
		awale.pits = [0,0,0,0,0,0,1,1,0,2,4,4];
		println!("{}", awale);
		// need to feed
		assert!(awale.play(6) == false);
		assert!(awale.play(7) == false);
		// empty pit
		assert!(awale.play(8) == false);
		// need to feed
		assert!(awale.play(9) == false);
		//ok
		assert!(awale.play(10));
		println!("{}", awale);
		assert!(awale.pits == [1,1,1,0,0,0,1,1,0,2,0,5]);
	}
	#[test]
	fn test_awale_12_seed_rule() {
		let mut awale = Awale::default();
		awale.pits = [0,0,0,0,0,13,2,2,1,2,2,5];
		awale.turn = Player::Player1;
		assert!(awale.play(5));
		assert!(awale.pits == [1,1,1,1,1,0,4,4,2,3,3,6]);
		awale.pits = [0,0,0,0,0,24,2,2,1,2,2,5];
		awale.turn = Player::Player1;
		assert!(awale.play(5));
		assert!(awale.pits == [2,2,2,2,2,0,5,5,3,4,4,7]);
	}
	#[test]
	fn test_awale_captures() {
		let mut awale = Awale::default();

		//capture
		awale.pits = [0,0,0,0,0,5,1,1,1,2,2,5];
		awale.turn = Player::Player1;
		awale.score_bottom = 0;
		awale.score_top = 0;
		awale.game_over = false;
		assert!(awale.play(5));
		assert!(awale.pits == [0,0,0,0,0,0,0,0,0,0,0,5]);
		assert!(awale.score_bottom == 12);

		//no capture on owned pits
		awale.pits = [1,0,0,0,0,7,1,1,1,2,2,2];
		awale.turn = Player::Player1;
		awale.score_bottom = 0;
		awale.score_top = 0;
		awale.game_over = false;
		assert!(awale.play(5));
		assert!(awale.pits == [2,0,0,0,0,0,2,2,2,3,3,3]);
		assert!(awale.score_bottom == 0);

		// capture all (not allowed)
		awale.pits = [0,0,0,0,0,6,1,1,1,2,2,2];
		awale.turn = Player::Player1;
		awale.score_bottom = 0;
		awale.score_top = 0;
		awale.game_over = false;
		assert!(awale.play(5) == false);
		awale.pits = [0,0,0,0,0,17,1,1,1,2,2,2];
		awale.turn = Player::Player1;
		awale.score_bottom = 0;
		awale.score_top = 0;
		awale.game_over = false;
		assert!(awale.play(5) == false);
		awale.pits = [0,0,0,0,0,28,1,1,1,2,2,2];
		awale.turn = Player::Player1;
		awale.score_bottom = 0;
		awale.score_top = 0;
		awale.game_over = false;
		assert!(awale.play(5) == false);
	}
}