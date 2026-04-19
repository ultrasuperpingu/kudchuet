use std::hash::{DefaultHasher, Hash, Hasher};

use kudchuet::Player;

use crate::bitboard::Goban;

use super::rules::{Gomoku, Move};
use kudchuet::ai::minimax::{Evaluation, Evaluator, Game, Winner};

impl Game for Gomoku {
	type S = Gomoku;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<Winner> {
		state.legal_moves_inplace(moves);
		Self::get_winner(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play_unchecked(m);
		Some(s2)
	}

	fn get_winner(state: &Self::S) -> Option<Winner> {
		state.result().into()
	}
	fn notation(_state: &Self::S, _move: Self::M) -> Option<String> {
		let (x2, y2) = Goban::coords_from_index(_move.to as usize);
		let file_char2 = (b'a' + x2) as char;
		let rank_char2 = (b'1' + y2) as char;

		Some(format!("{}{}", file_char2, rank_char2))
	}

	fn zobrist_hash(state: &Self::S) -> u64 {
		let mut hasher = DefaultHasher::new();
		state.hash(&mut hasher);
		hasher.finish()
	}
	fn current_player(state: &Self::S) -> Player {
		state.turn
	}
}

#[derive(Clone, Default, Copy, Debug)]
pub struct GomokuEvalDumb;

impl GomokuEvalDumb {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for GomokuEvalDumb {
	type G = Gomoku;
	fn evaluate_for(&self, _state: &Gomoku, _p: Player) -> Evaluation {
		0
	}
}
#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct GomokuEvalSimple;

impl GomokuEvalSimple {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for GomokuEvalSimple {
	type G = Gomoku;
	fn evaluate_for(&self, state: &Gomoku, p: Player) -> Evaluation {
		let mut score = 0;
		score += if state.white.has_aligned::<4>() {
			100
		} else {
			0
		};
		score += if state.white.has_aligned::<3>() {
			50
		} else {
			0
		};
		score += if state.white.has_aligned::<2>() {
			10
		} else {
			0
		};
		score -= if state.black.has_aligned::<4>() {
			100
		} else {
			0
		};
		score -= if state.black.has_aligned::<3>() {
			50
		} else {
			0
		};
		score -= if state.black.has_aligned::<2>() {
			10
		} else {
			0
		};
		if p == Player::PLAYER2 { score } else { -score }
	}
}
// cargo test --release -p gomoku game::tests::perft_test -- --nocapture
//depth           count        time        kn/s
//    0               1     400.0ns      2500.0
//    1             361       2.4µs    150416.7
//    2          129960     134.5µs    966245.4
//    3        46655640      43.0ms   1084187.9
//    4     16702719120        2.6s   6400302.3
#[cfg(test)]
mod tests {

	use kudchuet::ai::minimax::util::perft;

	use crate::rules::Gomoku;
	#[test]
	fn perft_test() {
		println!("BMI1 enabled? {}", cfg!(target_feature = "bmi1"));
		let mut board = Gomoku::new();

		let max_depth = 4;
		let nodes = perft::<Gomoku>(&mut board, max_depth, true);
		assert!(nodes.len() == (max_depth + 1) as usize);

		const NB_NODES: [u64; 5] = [1, 361, 129960, 46655640, 16702719120];

		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}
