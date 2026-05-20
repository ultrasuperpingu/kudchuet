
use kudchuet::{
	GameOutcome, Player,
	ai::minimax::{Evaluation, Evaluator, Game},
};

use crate::{
	bitboard::Bitboard5x5,
	rules::{Move, Teeko},
};

impl Game for Teeko {
	type S = Teeko;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		state.legal_moves_inplace(moves);
		state.result()
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play_unchecked(m);
		Some(s2)
	}

	fn get_outcome(state: &Self::S) -> GameOutcome {
		state.result()
	}
	fn notation(_state: &Self::S, _move: Self::M) -> Option<String> {
		if let Some(from) = _move.from {
			let (x1, y1) = Bitboard5x5::coords_from_index(from as usize);
			let (x2, y2) = Bitboard5x5::coords_from_index(_move.to as usize);
			let file_char1 = (b'a' + x1) as char;
			let rank_char1 = (b'1' + y1) as char;
			let file_char2 = (b'a' + x2) as char;
			let rank_char2 = (b'1' + y2) as char;

			Some(format!(
				"{}{}-{}{}",
				file_char1, rank_char1, file_char2, rank_char2
			))
		} else {
			let (x2, y2) = Bitboard5x5::coords_from_index(_move.to as usize);
			let file_char2 = (b'a' + x2) as char;
			let rank_char2 = (b'1' + y2) as char;

			Some(format!(
				"{}{}",
				file_char2, rank_char2
			))
		}
	}

	fn get_hash(state: &Self::S) -> u64 {
		state.get_hash()
	}

	fn get_current_player(state: &Self::S) -> Player {
		if state.turn%2 == 0 {
			Player::PLAYER1
		} else {
			Player::PLAYER2
		}
	}
}

#[derive(Clone, Default, Copy, Debug, PartialEq, Eq)]
pub struct TeekoEvalDumb;

impl TeekoEvalDumb {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for TeekoEvalDumb {
	type G = Teeko;
	fn evaluate_for(&self, _state: &Teeko, _p: Player) -> Evaluation {
		0
	}
}
// cargo test --release -p teeko game::tests::perft_test -- --nocapture
//depth           count        time        kn/s
//    0               1     300.0ns      3333.3
//    1              24       1.6µs     15000.0
//    2             576       2.2µs    261818.2
//    3           13248      26.4µs    501818.2
//    4          291456     631.0µs    461895.4
//    5         6120576       1.8ms   3365912.9
//    6       122411520      43.7ms   2802987.7
//    7      2325818880     531.3ms   4377470.6
//    8     41723398080        8.8s   4725776.8
#[cfg(test)]
mod tests {

	use crate::rules::Teeko;
	use kudchuet::ai::minimax::util::perft;

	#[test]
	fn perft_test() {
		println!("BMI1 enabled? {}", cfg!(target_feature = "bmi1"));
		let mut board = Teeko::new();

		let max_depth = 9;
		let nodes = perft::<Teeko>(&mut board, max_depth, true);
		assert!(nodes.len() == (max_depth + 1) as usize);

		const NB_NODES: [u64; 10] = [
			1,
			24,
			576,
			13248,
			291456,
			6120576,
			122411520,
			2325818880,
			41723398080,
			676850893056,
		];

		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}
