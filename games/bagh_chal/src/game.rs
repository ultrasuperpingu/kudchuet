
//use std::hash::{DefaultHasher, Hash, Hasher};


use kudchuet::{GameResult, Player};

use kudchuet::ai::minimax::{Evaluation, Evaluator, Game, Winner};
use crate::rules::{Move, BaghChal};

impl Game for BaghChal {
	type S =  BaghChal;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<Winner> {
		if state.goats_captured >= 5 {
			return Some(Winner::Player(1));
		}
		state.legal_moves_inplace(moves);
		if state.tigers_turn() && moves.is_empty() {
			return Some(Winner::Player(0));
		}
		None
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		//let mut s2 = state.clone();
		//s2.play_unchecked(&m);
		//Some(s2)
		state.play_unchecked(&m);
		None
	}
	fn undo(_state: &mut Self::S, _m: Self::M) {
		_state.undo_unchecked(&_m);
	}
	fn get_winner(state: &Self::S) -> Option<Winner> {
		match state.result() {
			GameResult::Draw => Some(Winner::Draw),
			GameResult::OnGoing => None,
			GameResult::Player(p) => {
				Some(Winner::Player(p))
			},
		}
	}

	fn zobrist_hash(state: &Self::S) -> u64 {
		//let mut hasher = DefaultHasher::new();
		//state.hash(&mut hasher);
		//hasher.finish()
		state.get_hash()
		//state.compute_hash()
	}
	fn current_player(state: &Self::S) -> Player {
		if state.turn_tiger {
			Player::PLAYER2
		} else {
			Player::PLAYER1
		}
	}
}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct BaghChalMaterialEval;

impl BaghChalMaterialEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for BaghChalMaterialEval {
	type G = BaghChal;
	fn evaluate_for(&self, state: &BaghChal, p: Player) -> Evaluation {
		if p == Player::PLAYER2 {
			state.goats_captured as Evaluation
		} else {
			-(state.goats_captured as Evaluation)
		}
	}
}

// cargo test --release -p bagh_chal game::tests::perft_test -- --nocapture
//depth           count        time        kn/s
//    0               1       3.8µs       263.2
//    1              21       1.3µs     16153.8
//    2             252       6.6µs     38181.8
//    3            5052       9.1µs    555164.8
//    4           68204     393.9µs    173150.5
//    5         1304788     549.5µs   2374500.5
//    6        18592000       9.3ms   2002132.2
//    7       339123476      81.2ms   4176428.3
//    8      4933406760        2.5s   1961832.2
//    9     85724933628       21.1s   4070769.6
#[cfg(test)]
mod tests {

	use kudchuet::ai::minimax::util::perft;

	use super::super::{game::BaghChal};
	#[test]
	fn perft_test() {
		println!("BMI1 enabled? {}", cfg!(target_feature = "bmi1"));
		let mut board = BaghChal::default();
		let max_depth = 9;
		let nodes = perft::<BaghChal>(&mut board, max_depth, true);
		assert!(nodes.len() == (max_depth + 1) as usize);

		const NB_NODES: [u64; 10] = [
			1,
			21,
			252,
			5052,
			68204,
			1304788,
			18592000,
			339123476,
			4933406760,
			85724933628,
		];

		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}