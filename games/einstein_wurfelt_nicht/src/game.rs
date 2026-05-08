use std::hash::{DefaultHasher, Hash, Hasher};

use kudchuet::{GameOutcome, Player};

use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

use super::rules::{EinsteinWurfeltNicht, MovePlay};

impl Game for EinsteinWurfeltNicht {
	type S = EinsteinWurfeltNicht;
	type M = MovePlay;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		*moves = state.legal_moves();
		Self::get_outcome(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s = state.clone();
		s.play_unchecked(m);
		Some(s)
	}

	fn notation(_state: &Self::S, mv: Self::M) -> Option<String> {
		Some(format!("{:?}", mv))
	}

	fn get_hash(state: &Self::S) -> u64 {
		let mut hasher = DefaultHasher::new();
		state.hash(&mut hasher);
		hasher.finish()
		//state.hash
	}
	fn get_current_player(state: &Self::S) -> Player {
		if state.is_red {
			Player::PLAYER2
		} else {
			Player::PLAYER1
		}
	}
	fn get_outcome(state: &Self::S) -> GameOutcome {
		state.result()
	}
	fn is_random_move(state: &Self::S) -> bool {
		state.dice.is_none()
	}

	fn get_probability(_state: &Self::S, _mv: Self::M) -> f32 {
		1.0 / 6.0
	}
}
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct EinsteinWurfeltNichtDumbEval {
	turn: Player,
}

impl Evaluator for EinsteinWurfeltNichtDumbEval {
	type G = EinsteinWurfeltNicht;

	fn evaluate_for(&self, _state: &EinsteinWurfeltNicht, _p: Player) -> Evaluation {
		0
	}
}

#[cfg(test)]
mod tests {

	use kudchuet::ai::minimax::util::perft;

	use super::EinsteinWurfeltNicht;
	//cargo test -p einstein_wurfelt_nicht --release game::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	// 0               1     300.0ns      3333.3
	// 1               6       5.4µs      1111.1
	// 2              36       3.1µs     11612.9
	// 3             216       9.7µs     22268.0
	// 4            1296     382.0µs      3392.7
	// 5            7776     184.7µs     42100.7
	// 6           46656     499.2µs     93461.5
	// 7          279936       2.0ms    141711.0
	// 8         1679616      12.5ms    134250.1
	// 9        10077696      73.4ms    137215.1
	//10        60466176     446.3ms    135471.0
	//11       362797056     528.2ms    686831.0
	//12      2176782336        3.1s    700671.1
	#[test]
	fn perft_test() {
		let mut board = EinsteinWurfeltNicht::default();

		let nodes = perft::<EinsteinWurfeltNicht>(&mut board, 12, true);
		const NB_NODES: [u64; 13] = [
			1, 6, 36, 216, 1296, 7776, 46656, 279936, 1679616, 10077696, 60466176, 362797056,
			2176782336,
		];
		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}
