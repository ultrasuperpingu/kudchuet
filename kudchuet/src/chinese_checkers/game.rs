use std::hash::{DefaultHasher, Hash};
use std::hash::Hasher;

use bitboard::{BitIter, Bitboard};

use crate::chinese_checkers::ChineseCheckersPlayer;

//use crate::common::Player;
use super::{ChineseCheckers, ChineseCheckerBoard, Move};


impl minimax::Game for ChineseCheckers {
	type S =  ChineseCheckers;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<minimax::Winner> {
		if let Some(w) = Self::get_winner(state) {
			return Some(w);
		}
		let mut mvs: Vec<Move>=state.generate_moves();
		moves.append(&mut mvs);
		None
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		let _ = s2.play(m);
		Some(s2)
	}

	fn get_winner(_state: &Self::S) -> Option<minimax::Winner> {
		if _state.winner().is_some() {
			return Some(minimax::Winner::PlayerJustMoved);
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
pub struct ChineseCheckersMaterialEval(ChineseCheckersPlayer);

impl ChineseCheckersMaterialEval {
	pub fn new(p:ChineseCheckersPlayer) -> Self {
		Self(p)
	}
}
impl minimax::Evaluator for ChineseCheckersMaterialEval {
	type G = ChineseCheckers;
	fn evaluate(&self, state: &ChineseCheckers) -> minimax::Evaluation {
		let mut score = 0;

		for p in ChineseCheckers::active_players(state.nb_players) {
			let b = state.board(p);
			let target = ChineseCheckerBoard::target_board(p);

			let mut dist = 0;

			for i in b.iter_bits() {
				let (x, y) = ChineseCheckerBoard::coords_from_index(i as usize);

				let mut best = i32::MAX;

				for t in target.iter_bits() {
					let (tx, ty) = ChineseCheckerBoard::coords_from_index(t as usize);

					let dx = x as i32 - tx as i32;
					let dy = y as i32 - ty as i32;
					let dz = -(dx + dy);
					let d = dx.abs().max(dy.abs()).max(dz.abs());

					best = best.min(d);
				}

				dist += best;
			}

			let in_target = (b.clone() & target.clone()).count() as i32;

			let value = 300 - dist + in_target * 20;

			if p == state.turn {
				score += value;
			} else {
				score -= value;
			}
		}

		score as minimax::Evaluation
	}
}

#[cfg(test)]
mod tests {

	use minimax::perft;

	use super::ChineseCheckers;
	//cargo test --release chinese_checkers::game::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	//0               1       9.2µs       108.7
	//1              14      18.7µs       748.7
	//2             196      21.8µs      8990.8
	//3            4452     204.9µs     21727.7
	//4          101124       1.6ms     62782.6
	//5         2660706      23.2ms    114561.7
	//6        70006689     765.8ms     91422.1
	//7      2089172964       21.3s     97994.0
	#[test]
	fn perft_test() {
		let mut board = ChineseCheckers::default();

		let nodes = perft::<ChineseCheckers>(&mut board, 10, true);
		const NB_NODES: [u64; 8] = [
			1,
			14,
			196,
			4452,
			101124,
			2660706,
			70006689,
			2089172964,
		];
		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}