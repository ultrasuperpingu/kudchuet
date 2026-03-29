use std::{sync::{Arc, atomic::AtomicBool}, time::Duration};

use minimax::{Evaluation, Evaluator, Game, IterativeOptions, Strategy, Table, TranspositionTable};

use crate::common::{Player, gui::{BoardGame, BoardMove}};

pub struct ExpectiMinimaxAlphaBeta<E>
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
	pub(crate) stop_search: Arc<AtomicBool>,
	pub(crate) best: Option<<E::G as Game>::M>,
	pub(crate) opts: IterativeOptions,
	pub(crate) table: TranspositionTable<<E::G as Game>::M>
}

impl<E> Strategy<E::G> for ExpectiMinimaxAlphaBeta<E>
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
		let _score = self.expectiminimax(&mut state, self.depth, player, minimax::WORST_EVAL, minimax::BEST_EVAL);
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

impl<E> ExpectiMinimaxAlphaBeta<E>
where
	E: Evaluator + Default,
	E::G: Game,
	<E::G as Game>::S: Clone + BoardGame,
	<E::G as Game>::M: Clone,
	<<E::G as Game>::S as Game>::M: BoardMove<<E::G as Game>::S>
{
	pub fn new(evaluator: E, iter: IterativeOptions) -> Self {
		Self {
			evaluator,
			depth: 5,
			timeout: Duration::from_secs(0),
			stop_search: Arc::new(AtomicBool::new(false)),
			opts: iter,
			best:None,
			table: TranspositionTable::new(128*1024*1024, minimax::Replacement::TwoTier)
		}
	}
	pub fn expectiminimax(&mut self,
		s: &mut <E::G as Game>::S,
		depth: u8,
		player: Player,
		mut alpha: Evaluation, mut beta: Evaluation
	) -> Option<Evaluation>
	where
		E: Default,
		E::G: Game,
		<E::G as Game>::S: Clone + BoardGame,
		<E::G as Game>::M: Clone,
		<<E::G as Game>::S as Game>::M: BoardMove<<E::G as Game>::S>
	{
		if depth == 0 {
			return Some(self.evaluator.evaluate(&s));
		}

		let mut moves = vec![];
		if let Some(winner) = E::G::generate_moves(&s, &mut moves) {
			return Some(winner.evaluate());
		}
		if moves.is_empty() {
			return Some(minimax::WORST_EVAL);
		}
		let alpha_orig = alpha;
		let hash = E::G::zobrist_hash(s);
		let mut good_move = None;
		if let Some(value) = self.table.check(hash, depth, &mut good_move, &mut alpha, &mut beta) {
			return Some(value);
		}

		if s.current_player() == player {
			let mut best_score = minimax::WORST_EVAL;
			let mut best = None;

			for m in moves {
				let mut applied = E::G::apply(s, m);
				let score = if let Some(child) = applied.as_mut() {
					self.expectiminimax(child, depth - 1, player, alpha, beta)?
				} else {
					let score = self.expectiminimax(s, depth - 1, player, alpha, beta)?;
					E::G::undo(s, m);
					score
				};
				if score > best_score {
					best_score = score;
					//println!("{}", best_score);
					best = Some(m);
				}
				if best_score >= beta {
					break;
				}
				alpha = alpha.max(best_score)
			}
			self.best = best;
			if let Some(best) = best {
				self.table.update(hash, alpha_orig, beta, depth, best_score, best);
			}
			Some(best_score)
		} else if s.current_player() == player.opponent() {
			let mut best_score = minimax::BEST_EVAL;
			let mut best = None;

			for m in moves {
				let mut applied = E::G::apply(s, m);
				let score = if let Some(child) = applied.as_mut() {
					self.expectiminimax(child, depth - 1, player, alpha, beta)?
				} else {
					let score = self.expectiminimax(s, depth - 1, player, alpha, beta)?;
					E::G::undo(s, m);
					score
				};
				if score < best_score {
					best_score = score;
					best = Some(m);
				}
				if best_score <= alpha {
					break;
				}
				beta = beta.min(best_score);
			}
			if let Some(best) = best {
				self.table.update(hash, alpha_orig, beta, depth, best_score, best);
			}
			Some(best_score)
		} else {// if node.current_player() == Player::RandomMove {
			let mut value = 0.0;
			let p = 1.0 / moves.len() as f32;

			for m in moves {
				let mut applied = E::G::apply(s, m);
				let score = if let Some(child) = applied.as_mut() {
					self.expectiminimax(child, depth - 1, player, alpha, beta)?
				} else {
					let score = self.expectiminimax(s, depth - 1, player, alpha, beta)?;
					E::G::undo(s, m);
					score
				};

				value += p * score as f32;
			}

			Some(value.round() as Evaluation)
		}
	}
	
	/// Return the search options used in this search.
	pub fn options(&self) -> &IterativeOptions {
		&self.opts
	}
	/// Return the search options used in this search.
	pub fn get_max_depth(&self) -> u8 {
		self.depth
	}
	/// Return the search options used in this search.
	pub fn get_max_time(&self) -> &Duration {
		&self.timeout
	}

	/// Get the flag used to end the best move search
	pub fn stop_search_flag(&self) -> Arc<AtomicBool> {
		self.stop_search.clone()
	}
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
	use super::*;
	use crate::backgammon::game::BackgammonSimpleEval;
	use crate::backgammon::rules::{Backgammon, Move};
	use crate::common::ai::expectiminimax::ExpectiMinimax;
	fn eq_unordered<T: Ord + Clone, const N: usize>(a: &[T; N], b: &[T; N]) -> bool {
		let mut a = a.clone();
		let mut b = b.clone();
		a.sort();
		b.sort();
		a == b
	}
	fn eq_move(a: &Move, b: &Move) -> bool {
		match (a,b) {
			(Move::Dice(d1, d2), Move::Dice(d3, d4)) => d1 == d3 && d2 == d4 || d1 == d4 && d2 == d3,
			(Move::Dice(_, _), Move::Player(_)) => false,
			(Move::Player(_), Move::Dice(_, _)) => false,
			(Move::Player(bg_player_move), Move::Player(bg_player_move2)) => bg_player_move.len == bg_player_move2.len && eq_unordered(&bg_player_move.moves, &bg_player_move2.moves),
		}
	}
	#[test]
	fn test_expectiminimax_backgammon() {
		let mut board = Backgammon::new();
		board.roll_dice();
		let mut minimax = ExpectiMinimax::new(BackgammonSimpleEval::default(), 3);
		minimax.set_max_depth(3);
		let mut minimax_ab = ExpectiMinimaxAlphaBeta::new(BackgammonSimpleEval::default(), IterativeOptions::new());
		minimax_ab.set_max_depth(3);
		while !board.is_game_over() {
			let best = minimax_ab.choose_move(&mut board).unwrap();
			let best2 = minimax.choose_move(&mut board).unwrap();

			if !eq_move(&best, &best2) {
				//panic!("{:?} != {:?}", best, best2);
				println!("{}, {}", BackgammonSimpleEval::default().evaluate(&board), BackgammonSimpleEval::default().evaluate(&board));
			}
			let legals = board.legal_moves();
			assert!(legals.contains(&best));
			board.play_unchecked(best);
			board.roll_dice();
		}
		println!("{}", board);
	}
	// cargo test --release common::ai::expectiminimax_alphabeta::tests::test_bench -- --nocapture
	#[test]
	fn test_bench() {
		let bench = crate::utils::Benchmark::new("ExpectiMinimax");
		let mut board = Backgammon::new();
		board.roll_dice();
		let mut minimax = ExpectiMinimax::new(BackgammonSimpleEval::default(), 4);
		let mut minimax_ab = ExpectiMinimaxAlphaBeta::new(BackgammonSimpleEval::default(), IterativeOptions::new());
		minimax_ab.set_max_depth(4);
		bench.record("no alpha", 60.0, |t| {
			t.add(|| { minimax.choose_move(&board); });
		});
		bench.record("alpha", 60.0, |t| {
			t.add(|| { minimax_ab.choose_move(&board); });
		});
		println!("{}", bench);
	}
}
