//use std::hash::{DefaultHasher, Hash, Hasher};

use kudchuet::{GameOutcome, Player};

use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

use super::rules::{Hex, Move};

impl Game for Hex {
	type S = Hex;
	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		state.legal_moves_inplace(moves);
		Self::get_outcome(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s=state.clone();
		s.play_unchecked(m);
		Some(s)
	}

	fn notation(_state: &Self::S, mv: Self::M) -> Option<String> {
		Some(format!("{:?}", mv))
	}

	fn get_hash(state: &Self::S) -> u64 {
		state.hash
	}
	fn get_current_player(state: &Self::S) -> Player {
		state.current_player()
	}
	fn get_outcome(state: &Self::S) -> GameOutcome {
		state.result()
	}
}
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct HexMaterialEval {
	turn:Player
}

impl Evaluator for HexMaterialEval {
	type G = Hex;

	fn evaluate_for(&self, _state: &Hex, _p: Player) -> Evaluation {
		0
	}
}

#[cfg(test)]
mod tests {

	use kudchuet::ai::minimax::util::perft;

	use super::Hex;
	//cargo test -p hex_game --release game::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	//    0               1       2.6µs       384.6
	//    1              49       4.9µs     10000.0
	//    2            2401       9.7µs    247525.8
	//    3          112896     123.7µs    912659.7
	//    4         5195568       1.9ms   2705038.8
	//    5       233911104      59.3ms   3945828.9
	//    6     10297173600        2.7s   3763305.1
	#[test]
	fn perft_test() {
		let mut board = Hex::default();

		let nodes = perft::<Hex>(&mut board, 6, true);
		const NB_NODES: [u64; 7] = [1, 49, 2401, 112896, 5195568, 233911104, 10297173600];
		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}
