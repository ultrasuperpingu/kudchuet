
//use std::hash::{DefaultHasher, Hash, Hasher};


use crate::common::{GameResult, bitboards::Bitboard5x5};

use super::{Move, Player, Neutron};

impl minimax::Game for Neutron {
	type S =  Neutron;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<minimax::Winner> {
		state.legal_moves_inplace(moves);
		if !(state.neutron & Bitboard5x5::SOUTH_BORDER).is_empty() {
			return if state.turn == Player::Player1 {
						Some(minimax::Winner::PlayerToMove)
					} else {
						Some(minimax::Winner::PlayerJustMoved)
					};
		}

		if !(state.neutron & Bitboard5x5::NORTH_BORDER).is_empty() {
			return if state.turn == Player::Player1 {
						Some(minimax::Winner::PlayerJustMoved)
					} else {
						Some(minimax::Winner::PlayerToMove)
					};
		}
		if moves.is_empty() {
			match state.turn {
				Player::Player1 =>  {
					if state.turn == Player::Player1 {
						Some(minimax::Winner::PlayerToMove)
					} else {
						Some(minimax::Winner::PlayerJustMoved)
					}
				},
				Player::Player2 => {
					if state.turn == Player::Player1 {
						Some(minimax::Winner::PlayerJustMoved)
					} else {
						Some(minimax::Winner::PlayerToMove)
					}
				},
				_ => unreachable!(),
			}
		} else {
			None
		}
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play(&m);
		Some(s2)
	}

	fn get_winner(state: &Self::S) -> Option<minimax::Winner> {
		match state.result() {
			GameResult::Draw => Some(minimax::Winner::Draw),
			GameResult::OnGoing => None,
			GameResult::Player1 => {
				if state.turn == Player::Player1 {
					Some(minimax::Winner::PlayerToMove)
				} else {
					Some(minimax::Winner::PlayerJustMoved)
				}
			},
			GameResult::Player2 => {
				if state.turn == Player::Player1 {
					Some(minimax::Winner::PlayerJustMoved)
				} else {
					Some(minimax::Winner::PlayerToMove)
				}
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
}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct NeutronDumbEval;

impl NeutronDumbEval {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for NeutronDumbEval {
	type G = Neutron;
	fn evaluate(&self, _state: &Neutron) -> minimax::Evaluation {
		0 as minimax::Evaluation
	}
}
/*
#[derive(Clone, Default, Copy)]
pub struct NeutronMaterialEval;

impl NeutronMaterialEval {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for NeutronMaterialEval {
	
	type G=NeutronGame;
	fn evaluate(&self, state: &Neutron) -> minimax::Evaluation {
		if state.turn == Player::White {
			state.white_pawns_count()as minimax::Evaluation - state.black_pawns_count()as minimax::Evaluation 
		} else {
			state.black_pawns_count() as minimax::Evaluation - state.white_pawns_count() as minimax::Evaluation
		}
	}
	
}*/
// cargo test --release neutron::game::tests::perft_test -- --nocapture
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

	use super::super::{game::Neutron};
	use minimax::perft;
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