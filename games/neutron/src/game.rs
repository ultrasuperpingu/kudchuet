
//use std::hash::{DefaultHasher, Hash, Hasher};


use kudchuet::{GameOutcome, Player};
use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

use crate::{bitboard::Bitboard5x5, rules::{Move, Neutron}};

impl Game for Neutron {
	type S =  Neutron;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		state.legal_moves_inplace(moves);
		if !(state.neutron & Bitboard5x5::SOUTH_BORDER).is_empty() {
			return GameOutcome::Player(Player::PLAYER2)
		}

		if !(state.neutron & Bitboard5x5::NORTH_BORDER).is_empty() {
			return GameOutcome::Player(Player::PLAYER1)
		}
		if moves.is_empty() {
			GameOutcome::Player(state.turn.opponent())
		} else {
			GameOutcome::OnGoing
		}
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play(&m);
		Some(s2)
	}

	fn get_winner(state: &Self::S) -> GameOutcome {
		state.result()
	}

	fn get_hash(state: &Self::S) -> u64 {
		//let mut hasher = DefaultHasher::new();
		//state.hash(&mut hasher);
		//hasher.finish()
		state.get_hash()
		//state.compute_hash()
	}
	
	fn get_current_player(state: &Self::S) -> Player {
		state.turn()
	}
}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct NeutronDumbEval;

impl NeutronDumbEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for NeutronDumbEval {
	type G = Neutron;
	fn evaluate_for(&self, _state: &Neutron, _p: Player) -> Evaluation {
		0 as Evaluation
	}
}
/*
#[derive(Clone, Default, Copy)]
pub struct NeutronMaterialEval(Player);

impl NeutronMaterialEval {
	pub fn new() -> Self {
		Self
	}
}
impl Evaluator for NeutronMaterialEval {
	
	type G=NeutronGame;
	fn evaluate(&self, state: &Neutron) -> Evaluation {
		if state.turn == Player::White {
			state.white_pawns_count()as Evaluation - state.black_pawns_count()as Evaluation 
		} else {
			state.black_pawns_count() as Evaluation - state.white_pawns_count() as Evaluation
		}
	}
	
}*/
// cargo test --release -p neutron game::tests::perft_test -- --nocapture
//depth           count        time        kn/s
//    0               1       2.2µs       454.5
//    1              13      20.1µs       646.8
//    2            1070      29.9µs     35786.0
//    3           66480     861.1µs     77203.6
//    4         3155916       6.1ms    518383.0
//    5       147545320     185.7ms    794640.3
//    6      6502302474        8.9s    729054.5
//    7    287147347484      388.9s    738430.6
#[cfg(test)]
mod tests {

	use kudchuet::ai::minimax::util::perft;

	use super::super::{game::Neutron};
	#[test]
	fn perft_test() {
		println!("BMI1 enabled? {}", cfg!(target_feature = "bmi1"));
		let mut board = Neutron::new();
		let max_depth = 6;
		let nodes = perft::<Neutron>(&mut board, max_depth, true);
			
		const NB_NODES: [u64;8] = [
			1,
			13,
			1070,
			66480,
			3155916,
			147545320,
			6502302474,
			287147347484
		];
		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}