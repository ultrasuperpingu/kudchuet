use std::hash::{DefaultHasher, Hash};
use std::hash::Hasher;

use kudchuet::Player;

use super::rules::Awale;



impl minimax::Game for Awale {
	type S =  Awale;

	type M = usize;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<minimax::Winner> {
		let mut mvs: Vec<Self::M>=state.legal_moves();
		moves.append(&mut mvs);
		// TODO: check winner
		Self::get_winner(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play_unchecked(m);
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
}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct AwaleMaterialEval;

impl AwaleMaterialEval {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for AwaleMaterialEval {
	type G = Awale;
	fn evaluate(&self, state: &Awale) -> minimax::Evaluation {
		if state.turn == Player::PLAYER1 {
			state.score_bottom as minimax::Evaluation - state.score_top as minimax::Evaluation
		} else {
			state.score_top as minimax::Evaluation - state.score_bottom as minimax::Evaluation
		}
	}
}
#[cfg(test)]
mod tests {

	use super::super::rules::Awale;
	use minimax::perft;

	//cargo test --release awale::game::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	//    0               1       4.2µs       238.1
	//    1               6      24.7µs       242.9
	//    2              36       3.7µs      9729.7
	//    3             190       7.4µs     25675.7
	//    4            1014     447.2µs      2267.4
	//    5            5219     155.5µs     33562.7
	//    6           27332     285.4µs     95767.3
	//    7          139157       1.2ms    118330.8
	//    8          711414       5.3ms    133318.5
	//    9         3592872      18.9ms    190381.1
	//   10        18137964     102.0ms    177904.8
	//   11        91558687     510.6ms    179299.0
	//   12       459952410        2.6s    174655.5
	#[test]
	fn perft_test() {
		let mut board = Awale::default();

		let nodes = perft::<Awale>(&mut board, 12, true);
		const NB_NODES: [u64; 13] = [
			1,
			6,
			36,
			190,
			1014,
			5219,
			27332,
			139157,
			711414,
			3592872,
			18137964,
			91558687,
			459952410,
		];
		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}

}