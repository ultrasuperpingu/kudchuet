//! A definition of a dice race game.
//!
//! First player to reach RACE_LENGTH exactly wins.
//! If player go above RACE_LENGTH, he bounces (score above is substracted from RACE_LENGTH)
//! If a player reaches the position of the other player, the other player go back to the begining.
#![allow(dead_code)]
extern crate kudchuet;

use std::default::Default;
use std::fmt::{Display, Formatter, Result};

use kudchuet::ai::minimax::*;
use kudchuet::{GameOutcome, Player};

const RACE_LENGTH: u8 = 48;

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Dice {
	OneD4,
	TwoD4,
	OneD6,
	TwoD6,
	OneD8,
	TwoD8,
	OneD10,
	TwoD10,
}
#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Move {
	Choice(Dice),
	Random(u8),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board {
	p1: u8,
	p2: u8,
	to_move: bool,
	current_dice_choice: Option<Dice>,
}
impl Board {
	fn roll_dices(&mut self) -> Move {
		match self.current_dice_choice {
			Some(Dice::OneD4) => Move::Random(fastrand::u8(1..=4)),
			Some(Dice::TwoD4) => Move::Random(fastrand::u8(1..=4) + fastrand::u8(1..=4)),

			Some(Dice::OneD6) => Move::Random(fastrand::u8(1..=6)),
			Some(Dice::TwoD6) => Move::Random(fastrand::u8(1..=6) + fastrand::u8(1..=6)),

			Some(Dice::OneD8) => Move::Random(fastrand::u8(1..=8)),
			Some(Dice::TwoD8) => Move::Random(fastrand::u8(1..=8) + fastrand::u8(1..=8)),

			Some(Dice::OneD10) => Move::Random(fastrand::u8(1..=10)),
			Some(Dice::TwoD10) => Move::Random(fastrand::u8(1..=10) + fastrand::u8(1..=10)),

			None => Move::Random(0),
		}
	}
}

impl Default for Board {
	fn default() -> Board {
		Board {
			p1: 0,
			p2: 0,
			to_move: false,
			current_dice_choice: None,
		}
	}
}

impl Display for Board {
	fn fmt(&self, f: &mut Formatter) -> Result {
		writeln!(f, "P1: {}", self.p1)?;
		writeln!(f, "P2: {}", self.p2)?;
		writeln!(f, "To move: {}", if self.to_move { "P2" } else { "P1" })?;
		Ok(())
	}
}
pub(crate) const fn splitmix64(mut x: u64) -> u64 {
	x = x.wrapping_add(0x9E3779B97F4A7C15);
	x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
	x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
	x ^ (x >> 31)
}
#[derive(Debug)]
pub struct DiceGame;

impl Game for DiceGame {
	type S = Board;
	type M = Move;

	fn generate_moves(b: &Board, ms: &mut Vec<Move>) -> GameOutcome {
		let res = Self::get_outcome(b);
		if res.is_ended() {
			return res;
		}
		if let Some(d) = b.current_dice_choice {
			match d {
				Dice::OneD4 => {
					for i in 1..=4 {
						ms.push(Move::Random(i));
					}
				}
				Dice::TwoD4 => {
					for i in 2..=8 {
						ms.push(Move::Random(i));
					}
				}

				Dice::OneD6 => {
					for i in 1..=6 {
						ms.push(Move::Random(i));
					}
				}
				Dice::TwoD6 => {
					for i in 2..=12 {
						ms.push(Move::Random(i));
					}
				}

				Dice::OneD8 => {
					for i in 1..=8 {
						ms.push(Move::Random(i));
					}
				}
				Dice::TwoD8 => {
					for i in 2..=16 {
						ms.push(Move::Random(i));
					}
				}

				Dice::OneD10 => {
					for i in 1..=10 {
						ms.push(Move::Random(i));
					}
				}
				Dice::TwoD10 => {
					for i in 2..=20 {
						ms.push(Move::Random(i));
					}
				}
			}
		} else {
			ms.push(Move::Choice(Dice::OneD4));
			ms.push(Move::Choice(Dice::TwoD4));
			ms.push(Move::Choice(Dice::OneD6));
			ms.push(Move::Choice(Dice::TwoD6));
			ms.push(Move::Choice(Dice::OneD8));
			ms.push(Move::Choice(Dice::TwoD8));
			ms.push(Move::Choice(Dice::OneD10));
			ms.push(Move::Choice(Dice::TwoD10));
		}
		GameOutcome::OnGoing
	}

	fn apply(b: &mut Board, m: Move) -> Option<Board> {
		let mut b = b.clone();
		match m {
			Move::Choice(dice) => {
				b.current_dice_choice = Some(dice);
			}
			Move::Random(i) => {
				if b.to_move {
					b.p2 += i;
					if b.p2 > RACE_LENGTH {
						b.p2 = RACE_LENGTH - (b.p2 - RACE_LENGTH);
					}
					if b.p1 == b.p2 {
						b.p1 = 0;
					}
				} else {
					b.p1 += i;
					if b.p1 > RACE_LENGTH {
						b.p1 = RACE_LENGTH - (b.p1 - RACE_LENGTH);
					}
					if b.p1 == b.p2 {
						b.p2 = 0;
					}
				}
				b.current_dice_choice = None;
				b.to_move = !b.to_move;
			}
		}
		Some(b)
	}
	fn notation(_state: &Self::S, _move: Self::M) -> Option<String> {
		Some(format!("{:?}", _move))
	}
	fn get_hash(_state: &Self::S) -> u64 {
		let hash = (_state.p1 as u64)
			| ((_state.p2 as u64) << 8)
			| ((_state.to_move as u64) << 17)
			| ((_state.current_dice_choice.is_some() as u64) << 18);
		splitmix64(hash)
	}
	fn get_current_player(state: &Self::S) -> Player {
		if state.to_move {
			Player::PLAYER2
		} else {
			Player::PLAYER1
		}
	}
	fn get_outcome(b: &Board) -> GameOutcome {
		if b.p1 == RACE_LENGTH {
			GameOutcome::PLAYER1
		} else if b.p2 == RACE_LENGTH {
			GameOutcome::PLAYER2
		} else {
			GameOutcome::OnGoing
		}
	}
	fn is_random_move(state: &Self::S) -> bool {
		state.current_dice_choice.is_some()
	}

	fn get_probability(state: &Self::S, mv: Self::M) -> f32 {
		match mv {
			Move::Choice(_) => 0.0,
			Move::Random(sum) => match state.current_dice_choice {
				Some(Dice::OneD4) => 1.0 / 4.0,
				Some(Dice::TwoD4) => prob_2d(4, sum),

				Some(Dice::OneD6) => 1.0 / 6.0,
				Some(Dice::TwoD6) => prob_2d(6, sum),

				Some(Dice::OneD8) => 1.0 / 8.0,
				Some(Dice::TwoD8) => prob_2d(8, sum),

				Some(Dice::OneD10) => 1.0 / 10.0,
				Some(Dice::TwoD10) => prob_2d(10, sum),

				None => 0.0,
			},
		}
	}
}

fn prob_2d(n: u8, sum: u8) -> f32 {
	if sum < 2 || sum > 2 * n {
		return 0.0;
	}
	let count = (sum - 1).min(2 * n + 1 - sum);
	count as f32 / (n * n) as f32
}
#[derive(Clone)]
pub struct DiceRaceEvaluator;

impl Default for DiceRaceEvaluator {
	fn default() -> Self {
		Self {}
	}
}
impl Evaluator for DiceRaceEvaluator {
	type G = DiceGame;
	fn evaluate_for(&self, b: &Board, p: Player) -> Evaluation {
		let remaining1 = RACE_LENGTH - b.p1;
		let remaining2 = RACE_LENGTH - b.p2;
		//println!("{remaining1} {remaining2}");
		let mut score_p1 = if remaining1 <= 5 {
			// =25% chance to win
			200 as i16
		} else if remaining1 <= 11 {
			// between 10% (10-11) and 18.75%(6) to win
			160 - remaining1 as i16
		} else if remaining1 <= 20 {
			// win is possible
			100 - remaining1 as i16
		} else {
			100 - remaining1 as i16 * 2
		};
		let mut score_p2 = if remaining2 <= 5 {
			// =25% chance to win
			200 as i16
		} else if remaining2 <= 11 {
			// between 10% (10-11) and 18.75%(6) to win
			160 - remaining2 as i16
		} else if remaining2 <= 20 {
			// win is possible
			100 - remaining2 as i16
		} else {
			100 - remaining2 as i16 * 2
		};

		// penalty when opponent close behind
		if b.p1 > b.p2 {
			let diff = b.p1.abs_diff(b.p2);
			if diff <= 5 {
				score_p1 -= 30;
			} else if diff <= 11 {
				score_p1 -= 20 - diff as i16;
			}
		} else {
			let diff = b.p1.abs_diff(b.p2);
			if diff <= 5 {
				score_p2 -= 30;
			} else if diff <= 11 {
				score_p2 -= 20 - diff as i16;
			}
		}

		let score = score_p2 - score_p1;
		let score = score as Evaluation;
		if p == Player::PLAYER1 { score } else { -score }
	}
}

fn main() {
	let mut minimax = ExpectiMinimax::new(DiceRaceEvaluator::default(), 8, true);
	let mut minimax2 = IterativeSearch::new(
		DiceRaceEvaluator::default(),
		IterativeOptions::new().with_null_window_search(true),
	);
	minimax2.set_max_depth(8);
	let mut b = Board::default();
	while !DiceGame::get_outcome(&b).is_ended() {
		println!("{}", b);
		let strategy: &mut dyn Strategy<DiceGame> = if b.to_move {
			&mut minimax2
		} else {
			&mut minimax
		};
		match strategy.choose_move(&mut b) {
			Some(m) => {
				println!("Choose: {:?}", m);
				b = DiceGame::apply(&mut b, m).unwrap()
			}
			None => break,
		};
		let dices = b.roll_dices();
		println!("Rolled: {:?}", dices);
		b = DiceGame::apply(&mut b, dices).unwrap();
	}
	println!("{}", b);
}
