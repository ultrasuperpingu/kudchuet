use std::hash::{DefaultHasher, Hash};
use std::hash::Hasher;


use kudchuet::Player;

use super::rules::{Abalone,Move};



impl minimax::Game for Abalone {
	type S =  Abalone;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<minimax::Winner> {
		if let Some(winner) = Self::get_winner(state) {
			return Some(winner);
		}
		let mut mvs = vec![];
		state.legal_moves_inplace(&mut mvs);
		moves.append(&mut mvs);
		None
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play(m);
		Some(s2)
	}

	fn get_winner(state: &Self::S) -> Option<minimax::Winner> {
		if state.is_over() {
			if state.winner().is_some() {
				return Some(minimax::Winner::PlayerJustMoved);
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
	fn notation(_state: &Self::S, _move: Self::M) -> Option<String> {
		Some(_move.to_string())
	}
}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct AbaloneMaterialEval;

impl AbaloneMaterialEval {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for AbaloneMaterialEval {
	type G = Abalone;
	fn evaluate(&self, state: &Abalone) -> minimax::Evaluation {
		if state.turn == Player::PLAYER1 {
			state.black_out as minimax::Evaluation - state.white_out as minimax::Evaluation
		} else {
			state.white_out as minimax::Evaluation - state.black_out as minimax::Evaluation
		}
	}
}
#[cfg(test)]
mod tests {

	use super::super::rules::Abalone;
	use minimax::perft;

	//cargo test --release -p abalone game::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	//    0               1       2.0µs       500.0
	//    1              46      24.5µs      1877.6
	//    2            2116     315.8µs      6700.4
	//    3          110216      11.6ms      9529.7
	//    4         5723180     123.5ms     46351.3
	//    5       328440016        5.6s     58327.4
	#[test]
	fn perft_test() {
		let mut board = Abalone::default();

			let nodes = perft::<Abalone>(&mut board, 5, true);
			const NB_NODES: [u64;6] = [
				1,
				46,
				2116,
				110216,
				5723180,
				328440016,
			];
			for (i, n) in nodes.iter().enumerate() {
				assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
			}
	}
}