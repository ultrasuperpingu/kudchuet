
use bitboard::{BitIter, Bitboard};

use kudchuet::{GameResult, Player};

use crate::bitboard::Bitboard9x13;


const ACTION_PER_MOVE : usize = 3;

#[derive(Clone, Copy)]
pub enum Cell {
	White,
	Black,
	WhiteWithBall,
	BlackWithBall,
	Ball,
	Empty
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
	Move{from: u8, to: u8},
	Shoot{from:u8, to:u8}
}
impl Action {
	pub fn is_move(&self) -> bool {
		matches!(self, Action::Move{..})
	}
	pub fn from(&self) -> u8 {
		match self {
			Action::Move{from, ..} => *from,
			Action::Shoot{from, ..} => *from
		}
	}
	pub fn to(&self) -> u8 {
		match self {
			Action::Move{to, ..} => *to,
			Action::Shoot{to, ..} => *to
		}
	}
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Move (pub [Option<Action>;ACTION_PER_MOVE]);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct FootBoard {
	pub(crate) player1: Bitboard9x13,
	pub(crate) player2: Bitboard9x13,
	pub(crate) ball: u8,
	pub(crate) turn: u8,
	pub(crate) score1:u8,
	pub(crate) score2:u8,
}
impl Default for FootBoard {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}
impl FootBoard {
	pub const fn new() -> Self {
		let mut s=Self {
			player1: Bitboard9x13::from_storage(0b011111111),
			player2: Bitboard9x13::from_storage(0b011111111_000000000),
			ball:58,
			turn: 0,
			score1:0,
			score2:0,
		};
		s.reset_after_goal(Player::PLAYER1);
		s
	}
	#[inline]
	pub fn play_unchecked(&mut self, mvs: &Move) {
		for a in mvs.0.into_iter().flatten() {
			if self.turn() == Player::PLAYER1 {
				Self::play_action(&a, &mut self.player1, &mut self.ball);
			} else {
				Self::play_action(&a, &mut self.player2, &mut self.ball);
			}
		}
		if (Bitboard9x13::from_index(self.ball as usize) & Bitboard9x13::PLAYER1_GOAL).any() {
			self.score2 += 1;
			self.reset_after_goal(Player::PLAYER1);
		} else if (Bitboard9x13::from_index(self.ball as usize) & Bitboard9x13::PLAYER2_GOAL).any() {
			self.score1 += 1;
			self.reset_after_goal(Player::PLAYER2);
		}
		self.turn += 1;
	}
	/*#[inline]
	pub fn undo_unchecked(&mut self, mvs: &Move) {
		self.turn -= 1;
		for m in mvs.0.iter().rev() {
			if let Some(a) = m {
				if self.turn() == Player::Player1 {
					Self::undo_action(&a, &mut self.player1, &mut self.ball);
				} else {
					Self::undo_action(&a, &mut self.player2, &mut self.ball);
				}
			}
		}
		if (Bitboard9x13::from_index(self.ball as usize) & Bitboard9x13::PLAYER1_GOAL).any() {
			self.score2 -= 1;
			self.reset_after_goal(Player::Player1);
		} else if (Bitboard9x13::from_index(self.ball as usize) & Bitboard9x13::PLAYER2_GOAL).any() {
			self.score1 -= 1;
			self.reset_after_goal(Player::Player2);
		}
	}*/
	const fn reset_after_goal(&mut self, p: Player) {
		self.ball = 58;
		match p {
			Player::PLAYER1 => {
				self.player1 = Bitboard9x13::from_storage(0b000110000_010000010_000000000_000010000_001000100_000010000_000000000);
				self.player2 = Bitboard9x13::from_storage(0b000010000_001000100_000010000_000101000_010000010_000000000_000000000_000000000_000000000_000000000_000000000_000000000);
			},
			Player::PLAYER2 => {
				self.player1 = Bitboard9x13::from_storage(0b000000000_010000010_000101000_000010000_001000100_000010000_000000000);
				self.player2 = Bitboard9x13::from_storage(0b000010000_001000100_000010000_000000000_010000010_000110000_000000000_000000000_000000000_000000000_000000000_000000000);
			}
			_ => unreachable!()
		}
	}
	#[inline(always)]
	pub(crate) fn play_action(a:&Action, player_mask: &mut Bitboard9x13, ball: &mut u8) {
		match a {
			Action::Move { from, to } => {
				player_mask.reset_at_index(*from as usize);
				player_mask.set_at_index(*to as usize);
				if ball == from {
					*ball = *to;
				}
			},
			Action::Shoot { from:_, to } => {
				*ball = *to;
			},
		}
	}
	/*#[inline(always)]
	pub(crate) fn undo_action(a:&Action, player_mask: &mut Bitboard9x13, ball: &mut u8) {
		match a {
			Action::Move { from, to } => {
				player_mask.set_at_index(*from as usize);
				player_mask.reset_at_index(*to as usize);
				if ball == to {
					*ball = *from;
				}
			},
			Action::Shoot { from, to:_ } => {
				*ball = *from;
			},
		}
	}*/

	#[inline]
	pub fn legal_moves(&self, moves:&mut Vec<Move>) {
		moves.clear();
		if self.turn >= 30 {
			return;
		}
		// empty move is legal
		moves.push(Move([None;ACTION_PER_MOVE]));
		let (mine, theirs) = if self.turn() == Player::PLAYER1 {
			(self.player1, self.player2)
		} else {
			(self.player2, self.player1)
		};
		let all = self.player1 | self.player2;
		let empty = all.flipped();
		self.generate_possible_actions(mine, self.ball, all, empty,theirs, moves, 0, moves[0]);
	}
	fn generate_possible_actions(&self,
			player_mask: Bitboard9x13,
			ball: u8,
			all: Bitboard9x13,
			empty: Bitboard9x13,
			opponent_mask: Bitboard9x13,
			moves: &mut Vec<Move>,
			current_action_idx: u8,
			last_move: Move,
		)
	{
		if current_action_idx >= ACTION_PER_MOVE as u8 {
			return;
		}
		let ball_mask = Bitboard9x13::from_index(ball as usize);
		// moves
		for p in player_mask.iter_bits() {
			if last_move.0.iter().any(|m| m.iter().any(|a| a.is_move() && a.to() == p as u8)) {
				continue;
			}
			for to in (Bitboard9x13::NEIGHBORS_8[p as usize] & empty & !Bitboard9x13::BEHIND_GOALS).iter_bits() {
				let a=Action::Move { from: p as u8, to: to as u8 };
				let mut mv = last_move.clone();
				mv.0[current_action_idx as usize] = Some(a);
				
				moves.push(mv);

				let mut player_mask_after=player_mask;
				let mut ball_after=ball;
				let mut all_after=all;
				all_after.reset_at_index(p as usize);
				all_after.set_at_index(to as usize);
				let mut empty_after=empty;
				empty_after.set_at_index(p as usize);
				empty_after.reset_at_index(to as usize);
				Self::play_action(&a, &mut player_mask_after, &mut ball_after);
				self.generate_possible_actions(player_mask_after, ball_after, all_after, empty_after, opponent_mask, moves, current_action_idx+1, mv);
			}
			if (opponent_mask & ball_mask).any() {
				//tackle
				let tackle_mask = if self.turn() == Player::PLAYER1 {Bitboard9x13::PLAYER1_TACKLE_POS_MASK[p as usize]} else {Bitboard9x13::PLAYER2_TACKLE_POS_MASK[p as usize]};
				if (tackle_mask & ball_mask).any() {
					if let Some(direction) = Bitboard9x13::get_direction(p as u8, ball) {
						let ray = Self::ray_from_direction(ball, direction);
						//println!("{}\n{}\n{:?}\n{}\n{}\n{}", opponent_mask, ball_mask, last_move, p, ball, ray);
						for to in ray.iter_bits() {
							if opponent_mask.ray_between(ball as usize, to as usize).any() {
								continue;
							}
							let a=Action::Shoot { from: p as u8, to: to as u8 };
							let mut mv = last_move;
							mv.0[current_action_idx as usize] = Some(a);
							
							moves.push(mv);
							
							let mut player_mask_after=player_mask;
							let mut ball_after=ball;
							Self::play_action(&a, &mut player_mask_after, &mut ball_after);
							self.generate_possible_actions(player_mask_after, ball_after, all, empty, opponent_mask, moves, current_action_idx+1, mv);
							//Self::undo_action(&a, &mut player_mask, &mut ball);
						}
					} else {
						println!("Ahhhhhhhhh!\n{}opponent_mask\n{}tackle_mask\n{}ball_mask\n", opponent_mask, tackle_mask, ball_mask);
					}
				}
			}
		}

		self.generate_shoots(player_mask, ball, all, empty, opponent_mask, moves, current_action_idx, last_move, ball_mask);
		
	}
	
	fn generate_shoots(&self, player_mask: Bitboard9x13, ball: u8, all: Bitboard9x13, empty: Bitboard9x13, opponent_mask: Bitboard9x13, moves: &mut Vec<Move>, current_action_idx: u8, last_move: Move, ball_mask: Bitboard9x13) {
		if (player_mask & ball_mask).any() {
			for to in (Bitboard9x13::SHOOT_MASK[ball as usize]).iter_bits() {
				if opponent_mask.ray_between(ball as usize, to as usize).any() {
					continue;
				}
				let a=Action::Shoot { from: ball, to: to as u8 };
				let mut mv = last_move;
				mv.0[current_action_idx as usize] = Some(a);

				moves.push(mv);

				let mut player_mask_after=player_mask;
				let mut ball_after=ball;
				Self::play_action(&a, &mut player_mask_after, &mut ball_after);
				self.generate_possible_actions(player_mask_after, ball_after, all, empty, opponent_mask, moves, current_action_idx+1, mv);
				//Self::undo_action(&a, &mut player_mask, &mut ball);
			
			}
		}
	}

	fn ray_from_direction(ball: u8, direction: u8) -> Bitboard9x13 {
		match direction {
			0 => {
				Bitboard9x13::RAY_E[ball as usize]
			},
			1 => {
				Bitboard9x13::RAY_NE[ball as usize]
			},
			2 => {
				Bitboard9x13::RAY_N[ball as usize]
			},
			3 => {
				Bitboard9x13::RAY_NW[ball as usize]
			},
			4 => {
				Bitboard9x13::RAY_W[ball as usize]
			},
			5 => {
				Bitboard9x13::RAY_SW[ball as usize]
			},
			6 => {
				Bitboard9x13::RAY_S[ball as usize]
			},
			7 => {
				Bitboard9x13::RAY_SE[ball as usize]
			},
			_ => unreachable!()
		}
	}

	#[inline]
	pub fn result(&self) -> GameResult {
		if self.turn < 30 {
			GameResult::OnGoing
		} else {
			if self.score1 > self.score2 {
				GameResult::PLAYER1
			} else if self.score1 < self.score2 {
				GameResult::PLAYER2
			} else {
				GameResult::Draw
			}
		}
	}
	
	pub fn cell(&self, x:u8, y:u8) -> Cell {
		if self.player1.get(x, y) {
			if self.ball == Bitboard9x13::index_from_coords(x, y) as u8 {
				Cell::WhiteWithBall
			} else {
				Cell::White
			}
		} else if self.player2.get(x, y) {
			if self.ball == Bitboard9x13::index_from_coords(x, y) as u8 {
				Cell::BlackWithBall
			} else {
				Cell::Black
			}
		} else if self.ball == Bitboard9x13::index_from_coords(x, y) as u8 {
			Cell::Ball
		} else {
			Cell::Empty
		}
	}
	#[inline(always)]
	pub fn turn(&self) -> Player {
		if self.turn.is_multiple_of(2) { Player::PLAYER1 } else { Player::PLAYER2 }
	}
	pub fn ball_owner(&self) -> Option<Player> {
		let ball_mask = Bitboard9x13::from_index(self.ball as usize);
		if (ball_mask & self.player1).any() {
			Some(Player::PLAYER1)
		} else if (ball_mask & self.player2).any() {
			Some(Player::PLAYER2)
		} else {
			None
		}
	}
	#[inline(always)]
	pub fn compute_hash(&self) -> u64 {
		//TODO
		0
	}
}

impl std::fmt::Display for FootBoard {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Turn : {:?}", self.turn)?;

		for y in (0..Bitboard9x13::HEIGHT).rev() {
			for x in 0..Bitboard9x13::WIDTH {
				
				let c = if self.player1.get(x,y) {
					if self.ball == Bitboard9x13::index_from_coords(x, y) as u8 {
						'W'
					} else {
						'w'
					}
				} else if self.player2.get(x,y) {
					if self.ball == Bitboard9x13::index_from_coords(x, y) as u8 {
						'B'
					} else {
						'b'
					}
				} else if self.ball == Bitboard9x13::index_from_coords(x, y) as u8 {
					'O'
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
#[cfg(test)]
mod tests {
	use crate::{bitboard::Bitboard9x13, rules::FootBoard};
	#[test]
	fn test_bitboard() {
		println!("{}", Bitboard9x13::PLAYER1_GOAL);
		println!("{}", Bitboard9x13::PLAYER2_GOAL);
		println!("{}", Bitboard9x13::all_lines_at_north(50));
		println!("{}", Bitboard9x13::all_lines_at_south(50));
		println!("{}", Bitboard9x13::compute_black_tackle_position_mask(50));
		println!("{}", Bitboard9x13::compute_white_tackle_position_mask(50));
		println!("{}", Bitboard9x13::PLAYER2_TACKLE_POS_MASK[50]);
		//println!("{:?}", Bitboard9x13::get_direction(50, 51));
		//println!("{:?}", Bitboard9x13::get_direction(50, 60));
		//println!("{:?}", Bitboard9x13::get_direction(50, 59));
		//println!("{:?}", Bitboard9x13::get_direction(50, 58));
		//println!("{:?}", Bitboard9x13::get_direction(50, 49));
		//println!("{:?}", Bitboard9x13::get_direction(50, 40));
		//println!("{:?}", Bitboard9x13::get_direction(50, 41));
		//println!("{:?}", Bitboard9x13::get_direction(50, 42));
		//println!("{:?}", Bitboard9x13::get_direction(50, 43));
		println!("{:?}", Bitboard9x13::get_direction(53, 54));
		println!("{:?}", Bitboard9x13::get_direction(53, 63));
		println!("{:?}", Bitboard9x13::get_direction(53, 62));
		println!("{:?}", Bitboard9x13::get_direction(53, 61));
		println!("{:?}", Bitboard9x13::get_direction(53, 52));
		println!("{:?}", Bitboard9x13::get_direction(53, 43));
		println!("{:?}", Bitboard9x13::get_direction(53, 44));
		println!("{:?}", Bitboard9x13::get_direction(53, 45));
		println!("{:?}", Bitboard9x13::get_direction(53, 112));
		println!("{}", Bitboard9x13::from_index(58));
		println!("{}", Bitboard9x13::from_index(67));
		println!("{}", FootBoard::ray_from_direction(58, Bitboard9x13::get_direction(Bitboard9x13::index_from_coords(5, 7) as u8, 58).unwrap()));
		//println!("{}", Bitboard9x13::from_index(51));
		//println!("{}", Bitboard9x13::from_index(60));
		//println!("{}", Bitboard9x13::from_index(59));
		//println!("{}", Bitboard9x13::from_index(51)|Bitboard9x13::from_index(51)|Bitboard9x13::from_index(60)|Bitboard9x13::from_index(59)|Bitboard9x13::from_index(58)|Bitboard9x13::from_index(49)|Bitboard9x13::from_index(41)|Bitboard9x13::from_index(42)|Bitboard9x13::from_index(43));
		//println!("{}", Bitboard9x13::from_index(50)|Bitboard9x13::from_index(41));
	}
	#[test]
	fn test_play() {
		let mut game=FootBoard::default();
		println!("{}", game);
		let mut moves=vec![];
		game.legal_moves(&mut moves);
		println!("{:?}", moves.len());
		game.play_unchecked(&moves[60]);
		println!("{}", game);
		
	}
}