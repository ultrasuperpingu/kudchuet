use crate::{GameOutcome, Player};
use crate::utils::Rng;

use super::interface::{BEST_EVAL, Evaluation, Evaluator, Game, Strategy, WORST_EVAL};
use super::util::{AppliedMove, MovePool};

pub struct ExpectiMinimax<E: Evaluator> {
	max_depth: u8,
	move_pool: MovePool<<E::G as Game>::M>,
	rng: Rng,
	shuffle_moves: bool,
	pub prev_value: Evaluation,
	pub eval: E,
}

impl<E> Strategy<E::G> for ExpectiMinimax<E>
where
	E: Evaluator,
	<E::G as Game>::S: Clone,
	<E::G as Game>::M: Copy,
{
	fn choose_move(&mut self, s: &<E::G as Game>::S) -> Option<<E::G as Game>::M> {
		if self.max_depth == 0 {
			return None;
		}
		let mut best = WORST_EVAL;
		let mut moves = self.move_pool.alloc();
		let player = E::G::get_current_player(s);
		if E::G::generate_moves(s, &mut moves).is_ended() {
			return None;
		}
		if self.shuffle_moves {
			// Randomly permute order that we look at the moves.
			// We'll pick the first best score from this list.
			self.rng.shuffle(&mut moves);
		}
		let mut best_move = *moves.first()?;
		let mut s_clone = s.clone();
		for &m in moves.iter() {
			// determine value for this move
			let mut new = AppliedMove::<E::G>::new(&mut s_clone, m);
			let value =
				self.expectiminimax(&mut new, self.max_depth - 1, player, WORST_EVAL, BEST_EVAL);
			// Strictly better than any move found so far.
			if value > best {
				best = value;
				best_move = m;
			}
		}
		self.move_pool.free(moves);
		self.prev_value = best;
		Some(best_move)
	}
	fn set_max_depth(&mut self, depth: u8) {
		self.max_depth = depth;
	}
	fn set_timeout(&mut self, _timeout: std::time::Duration) {
		self.max_depth = u8::MAX;
	}
	fn set_depth_or_timeout(&mut self, _depth: u8, _timeout: std::time::Duration) {
		self.max_depth = _depth;
	}
}
impl<E: Evaluator> ExpectiMinimax<E> {
	pub fn new(evaluator: E, depth: u8, shuffle_moves: bool) -> Self {
		Self {
			max_depth: depth,
			move_pool: MovePool::<_>::default(),
			shuffle_moves,
			rng: Rng::new(),
			prev_value: 0,
			eval: evaluator,
		}
	}
	pub fn root_value(&self) -> Evaluation {
		self.prev_value
	}
	pub fn expectiminimax(
		&mut self,
		s: &mut <E::G as Game>::S,
		depth: u8,
		player_to_move: Player,
		mut alpha: Evaluation,
		mut beta: Evaluation,
	) -> Evaluation {
		if depth == 0 {
			let res = E::G::get_winner(s);
			if res.is_ended() {
				return match res {
					GameOutcome::Player(p) if p == player_to_move => BEST_EVAL,
					GameOutcome::Player(_) => WORST_EVAL,
					GameOutcome::Draw => 0,
					_ => unreachable!()
				};
			}
			return self.eval.evaluate_for(s, player_to_move);
		}
		let mut moves = self.move_pool.alloc();
		let res = E::G::generate_moves(s, &mut moves);
		if res.is_ended() {
			return match res {
				GameOutcome::Player(p) if p == player_to_move => BEST_EVAL,
				GameOutcome::Draw => 0,
				GameOutcome::Player(_) => WORST_EVAL,
				_ => unreachable!()
			};
		}

		if moves.is_empty() {
			return WORST_EVAL;
		}
		let best_score = if E::G::is_random_move(s) {
			let mut value = 0.0;

			for m in moves.iter() {
				let p = E::G::get_probability(s, *m);
				let mut new = AppliedMove::<E::G>::new(s, *m);
				let score = self.expectiminimax(&mut new, depth - 1, player_to_move, alpha, beta);
				value += p * score as f32;
			}

			value.round() as Evaluation
		} else if E::G::get_current_player(s) == player_to_move {
			let mut best_score = WORST_EVAL;

			for m in moves.iter() {
				let mut new = AppliedMove::<E::G>::new(s, *m);
				let score = self.expectiminimax(&mut new, depth - 1, player_to_move, alpha, beta);
				if score > best_score {
					best_score = score;
				}
				if best_score >= beta {
					break;
				}
				alpha = alpha.max(best_score)
			}
			best_score
		} else {
			// if state.current_player() == to_choose_player.opponent() {
			let mut best_score = BEST_EVAL;

			for m in moves.iter() {
				let mut new = AppliedMove::<E::G>::new(s, *m);
				let score = self.expectiminimax(&mut new, depth - 1, player_to_move, alpha, beta);
				if score < best_score {
					best_score = score;
				}
				if best_score <= alpha {
					break;
				}
				beta = beta.min(best_score);
			}
			best_score
		};
		self.move_pool.free(moves);
		best_score
	}
}

#[cfg(test)]
mod tests {
	use crate::{GameOutcome, Player};

	use super::{Evaluator, ExpectiMinimax, Game, Strategy};

	#[derive(Debug, Default, Clone)]
	struct DumbGame {
		choice: Option<DiceChoice>,
		score: i16,
		to_move: bool,
	}
	#[derive(Copy, Clone, Debug, PartialEq, Eq)]
	enum DiceChoice {
		D4,
		D6,
		D8,
	}
	#[derive(Copy, Clone, Debug, PartialEq, Eq)]
	enum Move {
		DiceChoice(DiceChoice),
		Random(i16),
	}
	impl super::Game for DumbGame {
		type S = DumbGame;
		type M = Move;

		fn generate_moves(
			state: &Self::S,
			moves: &mut Vec<Self::M>,
		) -> GameOutcome {
			match state.choice {
				Some(d) => {
					moves.push(Move::Random(1));
					moves.push(Move::Random(2));
					moves.push(Move::Random(3));
					moves.push(Move::Random(4));
					match d {
						DiceChoice::D4 => {}
						DiceChoice::D6 => {
							moves.push(Move::Random(5));
							moves.push(Move::Random(6));
						}
						DiceChoice::D8 => {
							moves.push(Move::Random(5));
							moves.push(Move::Random(6));
							moves.push(Move::Random(7));
							moves.push(Move::Random(8));
						}
					}
				}
				None => {
					moves.push(Move::DiceChoice(DiceChoice::D4));
					moves.push(Move::DiceChoice(DiceChoice::D6));
					moves.push(Move::DiceChoice(DiceChoice::D8));
				}
			}
			GameOutcome::OnGoing
		}

		fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
			let mut clone = state.clone();
			match m {
				Move::DiceChoice(dice_choice) => clone.choice = Some(dice_choice),
				Move::Random(r) => {
					clone.score += r;
					clone.choice = None;
					clone.to_move = !clone.to_move;
				}
			}
			Some(clone)
		}

		fn get_winner(_state: &Self::S) -> GameOutcome {
			GameOutcome::OnGoing
		}
		fn get_current_player(state: &Self::S) -> Player {
			if state.to_move {
				Player(1)
			} else {
				Player(0)
			}
		}
		fn is_random_move(state: &Self::S) -> bool {
			state.choice.is_some()
		}

		fn get_probability(state: &Self::S, _mv: Self::M) -> f32 {
			match state.choice.unwrap() {
				DiceChoice::D4 => 1.0 / 4.0,
				DiceChoice::D6 => 1.0 / 6.0,
				DiceChoice::D8 => 1.0 / 8.0,
			}
		}
	}
	#[derive(Debug, Default)]
	struct Eval;
	impl Evaluator for Eval {
		type G = DumbGame;

		fn evaluate_for(&self, s: &<Self::G as super::Game>::S, p: Player) -> super::Evaluation {
			if p == Player::PLAYER2 { -s.score * 10 } else { s.score * 10 }
		}
	}
	#[test]
	fn test() {
		let mut strat = ExpectiMinimax::new(Eval::default(), 8, true);
		let mut i = 0;
		let mut s = DumbGame::default();
		while i < 10 {
			let m = strat.choose_move(&s);
			if !s.to_move {
				assert_eq!(m, Some(Move::DiceChoice(DiceChoice::D8)));
			} else {
				assert_eq!(m, Some(Move::DiceChoice(DiceChoice::D4)));
			}
			s = DumbGame::apply(&mut s, m.unwrap()).unwrap();
			let m = strat.choose_move(&s);
			if !s.to_move {
				assert_eq!(m, Some(Move::Random(8)));
			} else {
				assert_eq!(m, Some(Move::Random(1)));
			}

			s = DumbGame::apply(&mut s, m.unwrap()).unwrap();
			i += 1;
		}
	}
}
