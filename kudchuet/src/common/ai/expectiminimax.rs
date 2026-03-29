use std::time::Duration;

use minimax::{Evaluation, Evaluator, Game, Strategy};

use crate::common::{Player, gui::{BoardGame, BoardMove}};

pub struct ExpectiMinimax<E>
where
	E: Evaluator + Default,
	E::G: Game,
	<E::G as Game>::S: Clone + BoardGame,
	<E::G as Game>::M: Clone,
	<<E::G as Game>::S as Game>::M: BoardMove<<E::G as Game>::S>
{
	pub(crate) evaluator: E,
	pub(crate) depth: u8,
	pub(crate) timeout: Duration,
	pub(crate) best: Option<<E::G as Game>::M>
}

impl<E> Strategy<E::G> for ExpectiMinimax<E>
where
	E: Evaluator + Default,
	E::G: Game,
	<E::G as Game>::S: Clone + BoardGame,
	<E::G as Game>::M: Clone,
	<<E::G as Game>::S as Game>::M: BoardMove<<E::G as Game>::S>
{
	fn choose_move(&mut self, state: &<E::G as Game>::S) -> Option<<E::G as Game>::M> {
		let mut state = state.clone();
		let player = state.current_player();
		let _score = self.expectiminimax(&mut state, self.depth, player);
		self.best
	}
	fn set_max_depth(&mut self, depth: u8) {
		self.depth = depth;
		self.timeout = Duration::from_secs(0);
	}
	fn set_timeout(&mut self, _timeout: std::time::Duration) {
		self.timeout = _timeout;
		self.depth = u8::MAX;
	}
	fn set_depth_or_timeout(&mut self, _depth: u8, _timeout: std::time::Duration) {
		self.depth = _depth;
		self.timeout = _timeout;
	}
}
impl<E> ExpectiMinimax<E>
where
	E: Evaluator + Default,
	E::G: Game,
	<E::G as Game>::S: Clone + BoardGame,
	<E::G as Game>::M: Clone,
	<<E::G as Game>::S as Game>::M: BoardMove<<E::G as Game>::S>
{
	pub fn new(evaluator: E, depth: u8) -> Self {
		Self {
			evaluator,
			depth,
			timeout: Duration::from_secs(0),
			best:None
		}
	}
	pub fn expectiminimax(&mut self,
		node: &mut <E::G as Game>::S,
		depth: u8,
		player: Player
	) -> Option<Evaluation>
	where
		E: Default,
		E::G: Game,
		<E::G as Game>::S: Clone + BoardGame,
		<E::G as Game>::M: Clone,
		<<E::G as Game>::S as Game>::M: BoardMove<<E::G as Game>::S>
	{
		if depth == 0 {
			return Some(self.evaluator.evaluate(&node));
		}

		let mut moves = vec![];
		if let Some(winner) = E::G::generate_moves(&node, &mut moves) {
			return Some(winner.evaluate());
		}
		if moves.is_empty() {
			return Some(minimax::WORST_EVAL);
		}

		if node.current_player() == player {
			let mut best_score = minimax::WORST_EVAL;
			let mut best = None;

			for m in moves {
				let mut applied = E::G::apply(node, m);
				let score = if let Some(child) = applied.as_mut() {
					self.expectiminimax(child, depth - 1, player)?
				} else {
					let score = self.expectiminimax(node, depth - 1, player)?;
					E::G::undo(node, m);
					score
				};
				if score > best_score {
					best_score = score;
					//println!("{}", best_score);
					best = Some(m);
				}
			}
			self.best = best;
			Some(best_score)
		} else if node.current_player() == player.opponent() {
			let mut best_score = minimax::BEST_EVAL;

			for m in moves {
				let mut applied = E::G::apply(node, m);
				let score = if let Some(child) = applied.as_mut() {
					self.expectiminimax(child, depth - 1, player)?
				} else {
					let score = self.expectiminimax(node, depth - 1, player)?;
					E::G::undo(node, m);
					score
				};
				best_score = best_score.min(score);
			}
			Some(best_score)
		} else {// if node.current_player() == Player::RandomMove {
			let mut value = 0.0;
			let p = 1.0 / moves.len() as f32;

			for m in moves {
				let mut applied = E::G::apply(node, m);
				let score = if let Some(child) = applied.as_mut() {
					self.expectiminimax(child, depth - 1, player)?
				} else {
					let score = self.expectiminimax(node, depth - 1, player)?;
					E::G::undo(node, m);
					score
				};

				value += p * score as f32;
			}

			Some(value.round() as Evaluation)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::backgammon::game::BackgammonMaterialEval;
	use crate::backgammon::rules::Backgammon;

	#[test]
	fn test_expectiminimax_backgammon() {
		let mut board = Backgammon::new();
		board.roll_dice();
		let mut minimax = ExpectiMinimax::new(BackgammonMaterialEval::default(), 4);
		let score = minimax.expectiminimax(&mut board, 2, crate::common::Player::Player1);
		println!("Expectiminimax score: {}", score.unwrap());
		let best = minimax.choose_move(&mut board).unwrap();
		println!("Expectiminimax best: {:?}", best);
		let legals = board.legal_moves();
		assert!(legals.contains(&best));
		
	}
}
