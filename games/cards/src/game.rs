use std::hash::{DefaultHasher, Hash, Hasher};

use kudchuet::{
	GameOutcome, Player,
	ai::move_search::{Evaluation, Evaluator, Game},
};

use crate::rules::{Manille, Move};

impl Game for Manille {
	type S = Manille;

	type M = Move;

	#[inline]
	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		state.legal_moves_inplace(moves);
		Self::get_outcome(state)
	}

	#[inline]
	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play(m);
		Some(s2)
	}

	fn get_outcome(state: &Self::S) -> GameOutcome {
		if state.on_turn == 4 {
			GameOutcome::Draw
		} else if state.players.iter().all(|p| p.is_empty()) {
			if state.scores[0] > state.scores[1] {
				GameOutcome::PLAYER1
			} else {
				GameOutcome::PLAYER2
			}
		} else {
			GameOutcome::OnGoing
		}
	}

	fn get_current_player(state: &Self::S) -> Player {
		if state.on_turn >= 4 {
			return Player(0);
		}
		Player(state.on_turn)
	}
	#[inline]
	fn get_hash(state: &Self::S) -> u64 {
		let mut hasher = DefaultHasher::new();
		state.hash(&mut hasher);
		hasher.finish()
		//state.get_hash()
		//state.compute_hash()
	}
}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct ManilleDumbEval;

impl ManilleDumbEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for ManilleDumbEval {
	type G = Manille;
	fn evaluate_for(&self, _state: &Manille, _p: Player) -> Evaluation {
		0 as Evaluation
	}
}
#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct ManilleMaterialEval;

impl ManilleMaterialEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for ManilleMaterialEval {
	type G = Manille;
	fn evaluate_for(&self, state: &Manille, p: Player) -> Evaluation {
		if p.0.is_multiple_of(2) {
			state.scores[0] as Evaluation - state.scores[1] as Evaluation
		} else {
			state.scores[1] as Evaluation - state.scores[0] as Evaluation
		}
	}
}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct ManilleTestEval;

impl ManilleTestEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for ManilleTestEval {
	type G = Manille;
	fn evaluate_for(&self, state: &Manille, p: Player) -> Evaluation {
		let team = p.0 % 2;

		let score_diff =
			state.scores[team as usize] as i32 - state.scores[(1 - team) as usize] as i32;

		let hand_value: i32 = state
			.players
			.iter()
			.enumerate()
			.filter(|(i, _)| (*i as u8 % 2) == team)
			.map(|(_, hand)| hand.iter().map(|c| crate::rules::value(&c) as i32).sum::<i32>())
			.sum();

		let opp_value: i32 = state
			.players
			.iter()
			.enumerate()
			.filter(|(i, _)| (*i as u8 % 2) != team)
			.map(|(_, hand)| hand.iter().map(|c| crate::rules::value(&c) as i32).sum::<i32>())
			.sum();

		(score_diff * 100 + (hand_value - opp_value)) as Evaluation
	}
}
// cargo test --release -p cards game::tests::simple_perft_test -- --nocapture
// depth           count        time        kn/s
//     0               1     300.0ns      3333.3
//     1               2       3.9µs       512.8
//     2              10       3.5µs      2857.1
//     3              23       7.9µs      2911.4
//     4              52     444.2µs       117.1
//     5             108     113.7µs       949.9
//     6             506     112.8µs      4485.8
//     7             979     208.8µs      4688.7
//     8            1802     199.7µs      9023.5
//     9            3168     459.5µs      6894.5
//    10           11306     633.2µs     17855.3
//    11           30294       1.4ms     21920.4
//    12           73796       3.3ms     22348.2
//    13          150578      10.5ms     14374.7
//    14          533064      26.3ms     20234.8
//    15         1337790      66.9ms     19997.9
//    16         3210182     165.1ms     19446.3
//    17         7753610     437.0ms     17742.0
//    18        25849116        1.2s     22165.6
#[cfg(test)]
mod tests {
	use crate::rules::Manille;
	use kudchuet::ai::move_search::util::perft;

	#[test]
	fn simple_perft_test() {
		println!("BMI1 enabled? {}", cfg!(target_feature = "bmi1"));
		let mut board = Manille::default();
		let max_depth = 18;
		let nodes = perft::<Manille>(&mut board, max_depth, true);
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
