use std::hash::{DefaultHasher, Hash};
use std::hash::Hasher;
use bitboard::Bitboard;

use crate::common::Player;
use super::rules::{Checkers10, Move};


impl minimax::Game for Checkers10 {
	type S =  Checkers10;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<minimax::Winner> {
		let mut mvs: Vec<Move>=state.legal_moves();
		moves.append(&mut mvs);
		// TODO: check winner
		Self::get_winner(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play_unchecked(&m);
		Some(s2)
	}

	fn get_winner(state: &Self::S) -> Option<minimax::Winner> {
		if state.is_victory() {
			Some(minimax::Winner::PlayerJustMoved)
		} else if state.is_over() {
			Some(minimax::Winner::Draw)
		} else {
			None
		}
	}

	fn zobrist_hash(state: &Self::S) -> u64 {
		let mut hasher = DefaultHasher::new();
		state.hash(&mut hasher);
		hasher.finish()
	}
}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct CheckersEval;

impl CheckersEval {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for CheckersEval {
	type G = Checkers10;
	fn evaluate(&self, state: &Checkers10) -> minimax::Evaluation {
		if state.current_player == Player::PLAYER1 {
			state.whites().count() as minimax::Evaluation - state.blacks().count() as minimax::Evaluation
		} else {
			state.blacks().count() as minimax::Evaluation - state.whites().count() as minimax::Evaluation
		}
	}
}

#[cfg(test)]
mod tests {

	use super::Checkers10;
	use minimax::perft;
	//https://damforum.nl/viewtopic.php?t=2308
	//cargo test --release checkers::game::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	//    0               1      11.2µs        89.3
	//    1               9      26.5µs       339.6
	//    2              81      19.7µs      4111.7
	//    3             658     279.2µs      2356.7
	//    4            4265     862.3µs      4946.1
	//    5           27117       1.3ms     21533.4
	//    6          167140       5.3ms     31393.7
	//    7         1049442      38.6ms     27184.2
	//    8         6483971     234.2ms     27684.6
	//    9        41022614        1.5s     28048.8
	//   10       258935682        9.0s     28910.9
	#[test]
	fn perft_test() {
		let mut board = Checkers10::default();

		let nodes = perft::<Checkers10>(&mut board, 10, true);
		const NB_NODES: [u64; 11] = [
			1,
			9,
			81,
			658,
			4265,
			27117,
			167140,
			1049442,
			6483971,
			41022614,
			258935682,
		];
		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}