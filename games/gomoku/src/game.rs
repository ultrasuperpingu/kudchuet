
use std::hash::{DefaultHasher, Hash, Hasher};

use kudchuet::{GameResult, Player};

use crate::bitboard::Goban;

use super::rules::{Move, Gomoku};


impl minimax::Game for Gomoku {
	type S =  Gomoku;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<minimax::Winner> {
		state.legal_moves_inplace(moves);
		Self::get_winner(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play_unchecked(m);
		Some(s2)
	}

	fn get_winner(state: &Self::S) -> Option<minimax::Winner> {
		match state.result() {
			GameResult::OnGoing => None,
			GameResult::PLAYER1 => {
				if state.turn == Player::PLAYER1 {
					Some(minimax::Winner::PlayerToMove)
				} else {
					Some(minimax::Winner::PlayerJustMoved)
				}
			},
			GameResult::PLAYER2 => {
				if state.turn == Player::PLAYER1 {
					Some(minimax::Winner::PlayerJustMoved)
				} else {
					Some(minimax::Winner::PlayerToMove)
				}
			},
			GameResult::Player(_) => unreachable!(),
			GameResult::Draw => unreachable!(),
		}
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

}

#[derive(Clone, Default, Copy, Debug)]
pub struct GomokuEvalDumb;

impl GomokuEvalDumb {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for GomokuEvalDumb {
	type G = Gomoku;
	fn evaluate(&self, _state: &Gomoku) -> minimax::Evaluation {
		0
	}
}
#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct GomokuEvalSimple;

impl GomokuEvalSimple {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for GomokuEvalSimple {
	type G = Gomoku;
	fn evaluate(&self, state: &Gomoku) -> minimax::Evaluation {
		let mut score = 0;
		score += if state.white.has_n_aligned(4) {100} else {0};
		score += if state.white.has_n_aligned(3) {50} else {0};
		score += if state.white.has_n_aligned(2) {10} else {0};
		score -= if state.black.has_n_aligned(4) {100} else {0};
		score -= if state.black.has_n_aligned(3) {50} else {0};
		score -= if state.black.has_n_aligned(2) {10} else {0};
		if state.turn == Player::PLAYER2 {
			score
		} else {
			-score
		}
	}
}
// cargo test --release -p gomoku game::tests::perft_test -- --nocapture
//depth           count        time        kn/s
//    0               1       4.6µs       217.4
//    1             361       6.4µs     56406.2
//    2          129960     183.5µs    708228.9
//    3        46655850      64.9ms    718347.0
//    4     16703020890        3.7s   4525654.5
#[cfg(test)]
mod tests {

	use minimax::perft;

	use crate::rules::Gomoku;
	#[test]
	fn perft_test() {
		println!("BMI1 enabled? {}", cfg!(target_feature = "bmi1"));
		let mut board = Gomoku::new();
		
		let max_depth = 4;
		let nodes = perft::<Gomoku>(&mut board, max_depth, true);
		assert!(nodes.len() == (max_depth + 1) as usize);

		const NB_NODES: [u64; 5] = [
			1,
			361,
			129960,
			46655850,
			16703020890,
		];

		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}