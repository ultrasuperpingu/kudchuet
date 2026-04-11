use std::hash::{DefaultHasher, Hash};
use std::hash::Hasher;

use crate::rules::{Cell, Reversi};





impl minimax::Game for Reversi {
	type S =  Reversi;

	type M = (u8, u8);

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<minimax::Winner> {
		state.legal_moves(moves);
		Self::get_winner(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play_unchecked(m.0, m.1);
		Some(s2)
	}

	fn get_winner(state: &Self::S) -> Option<minimax::Winner> {
		if state.is_over() {
			if let Some(winner) = state.winner() {
				if winner == state.turn() {
					return Some(minimax::Winner::PlayerJustMoved);
				} else {
					return Some(minimax::Winner::PlayerToMove);
				}
			} else {
				return Some(minimax::Winner::Draw);
			}
		}
		None
	}

	fn zobrist_hash(state: &Self::S) -> u64 {
		let mut hasher = DefaultHasher::new();
		state.hash(&mut hasher);
		hasher.finish()
	}
}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct ReversiEval;

impl ReversiEval {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for ReversiEval {
	type G = Reversi;
	fn evaluate(&self, state: &Reversi) -> minimax::Evaluation {
		if state.turn() == Cell::White {
			0
		} else {
			//state.score_top as minimax::Evaluation - state.score_bottom as minimax::Evaluation
			0
		}
	}
}
#[cfg(test)]
mod tests {
	// cargo test --release reversi::game::tests::perft_test -- --nocapture
	// depth           count        time        kn/s
	//  0               1       3.1µs       322.6
	//  1               4     700.0ns      5714.3
	//  2              12     700.0ns     17142.9
	//  3              56       2.9µs     19310.3
	//  4             244     311.1µs       784.3
	//  5            1396      50.2µs     27808.8
	//  6            8200      91.0µs     90109.9
	//  7           55092     175.3µs    314272.7
	//  8          390216     688.0µs    567174.4
	//  9         3005264       5.0ms    596495.6
	// 10        24571000      27.5ms    891985.9
	// 11       212257448     237.9ms    892206.1
	// 12      1939875880        1.9s   1031846.7
	use crate::rules::Reversi;
	use minimax::perft;

	#[test]
	fn perft_test() {
		let mut board = Reversi::default();
		let max_depth = 12;

		let nodes = perft::<Reversi>(&mut board, max_depth, true);
		assert!(nodes.len() == (max_depth + 1) as usize);

		const NB_NODES: [u64; 15] = [
			1,
			4,
			12,
			56,
			244,
			1396,
			8200,
			55092,
			390216,
			3005264,
			24571284,
			212258236,
			1939886636,
			18429618408,
			184043884512,
		];

		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}