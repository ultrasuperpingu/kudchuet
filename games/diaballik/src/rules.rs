
use bitboard::{BitIter, Bitboard};

use kudchuet::{GameResult, Player};

use crate::bitboard::Bitboard7x7;


#[derive(Clone, Copy)]
pub enum Cell {
	White,
	Black,
	WhiteWithBall,
	BlackWithBall,
	Empty
}


#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Action {
	Move{from: u8, to: u8},
	Pass{from:u8, to:u8}
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Move (pub(crate) [Option<Action>;3]);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Diaballik {
	pub(crate) player1: Bitboard7x7,
	pub(crate) player2: Bitboard7x7,
	pub(crate) ball_player1: u8,
	pub(crate) ball_player2: u8,
	pub(crate) turn: Player,
	pub(crate) hash: u64,
}
impl Default for Diaballik {
	#[inline(always)]
	fn default() -> Self {
		let mut s = Self {
			player1: Bitboard7x7::SOUTH_BORDER,
			player2: Bitboard7x7::NORTH_BORDER,
			ball_player1: 3,
			ball_player2: 45,
			turn: Player::PLAYER1,
			hash: 0
		};
		s.hash = s.compute_zobrist();
		s
	}
}
impl Diaballik {
	#[inline]
	pub fn play_unchecked(&mut self, moves: &Move) {
		for a in moves.0.iter().flatten() {
			if self.turn == Player::PLAYER1 {
				Self::play_action(a, &mut self.player1, &mut self.ball_player1);
			} else {
				Self::play_action(a, &mut self.player2, &mut self.ball_player2);
			}
		}
		self.update_hash_move(moves);
		self.turn = self.turn.opponent();
		self.update_hash_turn();
	}
	#[inline(always)]
	pub(crate) fn play_action(a:&Action, player_mask: &mut Bitboard7x7, ball: &mut u8) {
		match a {
			Action::Move { from, to } => {
				player_mask.reset_at_index(*from as usize);
				player_mask.set_at_index(*to as usize);
			},
			Action::Pass { from:_, to } => {
				*ball = *to;
			},
		}
	}
	#[inline]
	pub fn undo_unchecked(&mut self, moves: &Move) {
		self.turn = self.turn.opponent();
		self.update_hash_turn();
		for a in moves.0.iter().rev().flatten() {
			if self.turn == Player::PLAYER1 {
				Self::undo_action(a, &mut self.player1, &mut self.ball_player1);
			} else {
				Self::undo_action(a, &mut self.player2, &mut self.ball_player2);
			}
		}
		self.update_hash_move(moves);
	}
	#[inline(always)]
	pub(crate) fn undo_action(a:&Action, player_mask: &mut Bitboard7x7, ball: &mut u8) {
		match a {
			Action::Move { from, to } => {
				player_mask.set_at_index(*from as usize);
				player_mask.reset_at_index(*to as usize);
			},
			Action::Pass { from, to:_ } => {
				*ball = *from;
			},
		}
	}
	#[inline]
	pub fn legal_moves(&self, moves:&mut Vec<Move>) {
		// empty move is legal
		moves.push(Move([None,None,None]));
		let (mine, theirs,ball) = if self.turn == Player::PLAYER1 {
			(self.player1, self.player2, self.ball_player1)
		} else {
			(self.player2, self.player1, self.ball_player2)
		};
		let all = self.player1 | self.player2;
		let empty = all.flipped();
		Self::generate_possible_actions(mine, ball, all, empty,theirs, moves, 0, moves[0], false);
	}
	fn generate_possible_actions(
			player_mask: Bitboard7x7,
			ball: u8,
			all: Bitboard7x7,
			empty: Bitboard7x7,
			opponent_mask: Bitboard7x7,
			moves: &mut Vec<Move>,
			current_action_idx: u8,
			last_move: Move,
			pass_done: bool
		)
	{
		if current_action_idx >= 3 {
			return;
		}
		// moves
		if pass_done || current_action_idx < 2 {
			for p in player_mask.iter_bits() {
				for to in (Bitboard7x7::NEIGHBORS_ORTHO[p as usize] & empty).iter_bits() {
					if p as u8 == ball { // ball owner can't move
						continue;
					}
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
					Self::generate_possible_actions(player_mask_after, ball_after, all_after, empty_after, opponent_mask, moves, current_action_idx+1, mv, pass_done);
				}
			}
		}
		if !pass_done {
			for to in (Bitboard7x7::PASS_MASK[ball as usize] & player_mask).iter_bits() {
				if (opponent_mask&Bitboard7x7::ray_between_mask(ball as usize, to as usize)).any() {
					continue;
				}
				let a=Action::Pass { from: ball, to: to as u8 };
				let mut mv = last_move;
				mv.0[current_action_idx as usize] = Some(a);
				moves.push(mv);
				let mut player_mask_after=player_mask;
				let mut ball_after=ball;
				Self::play_action(&a, &mut player_mask_after, &mut ball_after);
				Self::generate_possible_actions(player_mask_after, ball_after, all, empty, opponent_mask, moves, current_action_idx+1, mv, true);
			}
		}
	}
	#[inline]
	pub fn result(&self) -> GameResult {
		if self.turn == Player::PLAYER2 && self.ball_player1 > 41 {
			GameResult::PLAYER1
		}
		else if self.turn == Player::PLAYER1 && self.ball_player2 < 7 {
			GameResult::PLAYER2
		} else {
			// Anti-Game (Blocking) Rules
			if self.is_blocking(Player::PLAYER1) { return GameResult::PLAYER2; }
			if self.is_blocking(Player::PLAYER2) { return GameResult::PLAYER1; }
			GameResult::OnGoing
		}
	}
	pub(crate) fn is_blocking(&self, p: Player) -> bool {
		let (me, opponent) = match p {
			Player::PLAYER1 => (self.player1, self.player2),
			Player::PLAYER2 => (self.player2, self.player1),
			_ => unreachable!(),
		};

		let mut prev_row: Option<i8> = None;
		let mut contact_count = 0;

		for col in 0..7 {
			let col_mask = Bitboard7x7::col_mask(col);
			let my_piece_in_col = me & col_mask;

			if my_piece_in_col.count() != 1 { return false; }

			let idx = my_piece_in_col.lsb() as usize;
			let row = (idx / 7) as i8;

			if let Some(p_row) = prev_row {
				if row.abs_diff(p_row) > 1 { return false; }
			}

			let opponent_idx = if p == Player::PLAYER1 {
				idx + Bitboard7x7::V_OFFSET
			} else {
				idx.wrapping_sub(Bitboard7x7::V_OFFSET)
			};
			if opponent_idx < Bitboard7x7::NB_SQUARES && opponent.get_at_index(opponent_idx) {
				contact_count += 1;
			}

			prev_row = Some(row);
		}

		contact_count >= 3
	}
	pub fn is_blocking2(&self, p: Player) -> bool {
		let (me, opponent) = match p {
			Player::PLAYER1 => (self.player1, self.player2),
			Player::PLAYER2 => (self.player2, self.player1),
			_ => unreachable!(),
		};

		let mut flood = me.storage() & Bitboard7x7::WEST_BORDER.storage();
		if flood == 0 { return false; }

		for _ in 0..6 {
			let next = (
				(flood << Bitboard7x7::H_OFFSET) | 
				(flood << Bitboard7x7::V_OFFSET) | 
				(flood >> Bitboard7x7::V_OFFSET)
			) & me.storage();
			if next == flood { break; }
			flood = next;
		}

		if (flood & Bitboard7x7::EAST_BORDER.storage()) == 0 { return false; }

		let front_attack = if p == Player::PLAYER1 {
			(flood << Bitboard7x7::V_OFFSET) & opponent.storage()
		} else {
			(flood >> Bitboard7x7::V_OFFSET) & opponent.storage()
		};

		front_attack.count_ones() >= 3
	}

	pub fn cell(&self, x:u8, y:u8) -> Cell {
		if self.player1.get(x, y) {
			if self.ball_player1 == Bitboard7x7::index_from_coords(x, y) as u8 {
				Cell::WhiteWithBall
			} else {
				Cell::White
			}
		} else if self.player2.get(x, y) {
			if self.ball_player2 == Bitboard7x7::index_from_coords(x, y) as u8 {
				Cell::BlackWithBall
			} else {
				Cell::Black
			}
		} else {
			Cell::Empty
		}
	}
	#[inline(always)]
	pub fn turn(&self) -> Player {
		self.turn
	}
}
impl std::fmt::Display for Diaballik {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Turn : {:?}", self.turn)?;

		for y in (0..7).rev() {
			for x in 0..7 {
				
				let c = if self.player1.get(x,y) {
					if self.ball_player1 == Bitboard7x7::index_from_coords(x, y) as u8 {
						'W'
					} else {
						'w'
					}
				} else if self.player2.get(x,y) {
					if self.ball_player2 == Bitboard7x7::index_from_coords(x, y) as u8 {
						'B'
					} else {
						'b'
					}
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

pub struct Zobrist {
	pub pieces: [[u64; 2]; Bitboard7x7::NB_SQUARES],
	pub ball: [[u64; 2]; Bitboard7x7::NB_SQUARES],
	pub turn: u64,
}

impl Zobrist {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);
		let mut pieces = [[0u64; 2]; Bitboard7x7::NB_SQUARES];
		let mut ball = [[0u64; 2]; Bitboard7x7::NB_SQUARES];
		
		let mut i = 0;
		while i < Bitboard7x7::NB_SQUARES {
			pieces[i][0] = rng.u64();
			pieces[i][1] = rng.u64();
			ball[i][0] = rng.u64();
			ball[i][1] = rng.u64();
			i += 1;
		}
		
		Self {
			pieces,
			ball,
			turn: rng.u64(),
		}
	}
}
impl Diaballik {
	pub const ZOBRIST_KEYS: Zobrist = Zobrist::new(0x91F4A12);
	fn compute_zobrist(&self) -> u64 {
		let mut h = 0u64;
		for i in self.player1.iter_bits() { h ^= Self::ZOBRIST_KEYS.pieces[i as usize][0]; }
		for i in self.player2.iter_bits() { h ^= Self::ZOBRIST_KEYS.pieces[i as usize][1]; }
		h ^= Self::ZOBRIST_KEYS.ball[self.ball_player1 as usize][0];
		h ^= Self::ZOBRIST_KEYS.ball[self.ball_player2 as usize][1];
		if self.turn == Player::PLAYER2 { h ^= Self::ZOBRIST_KEYS.turn; }
		h
	}

	fn update_hash_move(&mut self, m: &Move) {
		for a in m.0.into_iter().flatten() {
			self.update_hash_action(&a);
		}
	}
	fn update_hash_action(&mut self, action: &Action) {
		let p_idx = if self.turn == Player::PLAYER1 { 0 } else { 1 };

		match action {
			Action::Move { from, to } => {
				self.hash ^= Self::ZOBRIST_KEYS.pieces[*from as usize][p_idx];
				self.hash ^= Self::ZOBRIST_KEYS.pieces[*to as usize][p_idx];
			}
			Action::Pass { from, to } => {
				self.hash ^= Self::ZOBRIST_KEYS.ball[*from as usize][p_idx];
				self.hash ^= Self::ZOBRIST_KEYS.ball[*to as usize][p_idx];
			}
		}
	}
	fn update_hash_turn(&mut self) {
		self.hash ^= Self::ZOBRIST_KEYS.turn
	}
}
impl Diaballik {
	pub fn to_fen(&self) -> String {
		let mut fen = String::new();

		for y in (0..7).rev() {
			let mut empty_count = 0;

			for x in 0..7 {
				let idx = Bitboard7x7::index_from_coords(x, y) as u8;

				let c = if self.player1.get(x, y) {
					if self.ball_player1 == idx {
						'W'
					} else {
						'w'
					}
				} else if self.player2.get(x, y) {
					if self.ball_player2 == idx {
						'B'
					} else {
						'b'
					}
				} else {
					empty_count += 1;
					continue;
				};

				if empty_count > 0 {
					fen.push_str(&empty_count.to_string());
					empty_count = 0;
				}

				fen.push(c);
			}

			if empty_count > 0 {
				fen.push_str(&empty_count.to_string());
			}

			if y > 0 {
				fen.push('/');
			}
		}

		fen.push(' ');
		fen.push(match self.turn {
			Player::PLAYER1 => 'w',
			Player::PLAYER2 => 'b',
			_ => unreachable!(),
		});

		fen
	}
	pub fn from_fen(fen: &str) -> Result<Self, String> {
		let mut parts = fen.split_whitespace();

		let board_part = parts.next().ok_or("Missing board")?;
		let turn_part = parts.next().ok_or("Missing turn")?;

		let rows: Vec<_> = board_part.split('/').collect();
		if rows.len() != 7 {
			return Err("Invalid number of rows".into());
		}

		let mut player1 = Bitboard7x7(0);
		let mut player2 = Bitboard7x7(0);
		let mut ball_player1 = 0;
		let mut ball_player2 = 0;

		let mut ball1_found = false;
		let mut ball2_found = false;

		let mut y: i8 = 6;

		for row in rows {
			let mut x = 0;

			for ch in row.chars() {
				if ch.is_ascii_digit() {
					let d = ch.to_digit(10).ok_or("Invalid digit")? as u8;
					x += d;
				} else {
					if x >= 7 {
						return Err("Row overflow".into());
					}

					let idx = Bitboard7x7::index_from_coords(x, y as u8) as u8;

					match ch {
						'w' => player1.set_at_index(idx as usize),
						'W' => {
							if ball1_found {
								return Err("Multiple balls for Player1".into());
							}
							player1.set_at_index(idx as usize);
							ball_player1 = idx;
							ball1_found = true;
						}
						'b' => player2.set_at_index(idx as usize),
						'B' => {
							if ball2_found {
								return Err("Multiple balls for Player2".into());
							}
							player2.set_at_index(idx as usize);
							ball_player2 = idx;
							ball2_found = true;
						}
						_ => return Err(format!("Invalid char: {}", ch)),
					}

					x += 1;
				}
			}

			if x != 7 {
				return Err("Row does not sum to 7".into());
			}

			y -= 1;
		}

		if !ball1_found || !ball2_found {
			return Err("Missing ball".into());
		}

		let turn = match turn_part {
			"w" => Player::PLAYER1,
			"b" => Player::PLAYER2,
			_ => return Err("Invalid turn".into()),
		};

		let mut s = Self {
			player1,
			player2,
			ball_player1,
			ball_player2,
			turn,
			hash: 0,
		};

		s.hash = s.compute_zobrist();
		Ok(s)
	}
}

#[cfg(test)]
mod tests {
    use kudchuet::GameResult;

    use crate::rules::Diaballik;


	#[test]
	fn test_play() {
		let mut game=Diaballik::default();
		println!("{}", game);
		let mut moves=vec![];
		game.legal_moves(&mut moves);
		println!("{:?}", moves.len());
		println!("{:?}", moves[128]);
		//println!("{:?}", moves);
		game.play_unchecked(&moves[128]);
		println!("{}", game);
		
	}
	#[test]
	fn test_play2() {
		let mut game=Diaballik::default();
		let mut rng = kudchuet::utils::Rng::new();
		println!("{}", game);
		while game.result() == GameResult::OnGoing {
			let mut moves=vec![];
			game.legal_moves(&mut moves);
			println!("{:?}", moves.len());
			let m = moves[rng.range(0, moves.len())];
			println!("{:?}", m);
			game.play_unchecked(&m);
			assert_eq!(game.compute_zobrist(), game.hash);
			println!("{}", game);
		}
		
	}
	#[test]
	fn test_fen_roundtrip_default() {
		let game = Diaballik::default();

		let fen = game.to_fen();
		let game2 = Diaballik::from_fen(&fen).unwrap();

		assert_eq!(game.player1, game2.player1);
		assert_eq!(game.player2, game2.player2);
		assert_eq!(game.ball_player1, game2.ball_player1);
		assert_eq!(game.ball_player2, game2.ball_player2);
		assert_eq!(game.turn, game2.turn);
		assert_eq!(game.hash, game2.hash);
	}
}