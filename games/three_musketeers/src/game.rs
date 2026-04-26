use bitboard::{BitIter, Bitboard};

use kudchuet::{
	GameOutcome, Player, ai::minimax::{Evaluation, Evaluator, Game}
};

use crate::{
	bitboard::Bitboard5x5,
	rules::{Move, ThreeMusketeers},
};

impl Game for ThreeMusketeers {
	type S = ThreeMusketeers;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		state.legal_moves_inplace(moves);
		Self::get_outcome(state)
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
		let (x1, y1) = Bitboard5x5::coords_from_index(_move.from as usize);
		let (x2, y2) = Bitboard5x5::coords_from_index(_move.to as usize);
		let file_char1 = (b'a' + x1) as char;
		let rank_char1 = (b'1' + y1) as char;
		let file_char2 = (b'a' + x2) as char;
		let rank_char2 = (b'1' + y2) as char;

		Some(format!(
			"{}{}-{}{}",
			file_char1, rank_char1, file_char2, rank_char2
		))
	}

	fn get_hash(state: &Self::S) -> u64 {
		state.get_hash()
	}

	fn get_current_player(state: &Self::S) -> Player {
		if state.turn == 0 {
			Player::PLAYER1
		} else {
			Player::PLAYER2
		}
	}
}

#[derive(Clone, Default, Copy, Debug)]
pub struct ThreeMusketeersEvalDumb;

impl ThreeMusketeersEvalDumb {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for ThreeMusketeersEvalDumb {
	type G = ThreeMusketeers;
	fn evaluate_for(&self, _state: &ThreeMusketeers, _p: Player) -> Evaluation {
		0
	}
}
#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct ThreeMusketeersEvalSimple;

impl ThreeMusketeersEvalSimple {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for ThreeMusketeersEvalSimple {
	type G = ThreeMusketeers;
	fn evaluate_for(&self, state: &ThreeMusketeers, p: Player) -> Evaluation {
		let mut nei = 0;
		let mut last = [(0, 0); 3];
		for (i, m) in state.musketeers.iter_bits().enumerate() {
			nei += state.guards.neighbors_ortho(m as usize).count();
			let coords = Bitboard5x5::coords_from_index(m as usize);
			last[i] = coords;
		}
		let mut max = 0;
		for a in 0..3 {
			for b in (a + 1)..3 {
				let (x1, y1) = last[a];
				let (x2, y2) = last[b];
				let dx = (x1 as i32 - x2 as i32).abs();
				let dy = (y1 as i32 - y2 as i32).abs();
				let d = dx.max(dy);
				if d > max {
					max = d;
				}
			}
		}

		let score = max as i16 - nei as i16;
		if p == Player::PLAYER1 { score } else { -score }
	}

}
#[derive(Clone, Default, Copy)]
pub struct ThreeMusketeersEvalAdvanced;

impl ThreeMusketeersEvalAdvanced {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for ThreeMusketeersEvalAdvanced {
	type G = ThreeMusketeers;
	fn evaluate_for(&self, state: &ThreeMusketeers, p: Player) -> Evaluation {
		let mut pressure = 0;
		let mut coords = vec![(0, 0); 3];

		for (i, m) in state.musketeers.iter_bits().enumerate() {
			pressure += state.guards.neighbors_ortho(m as usize).count();
			coords[i] = Bitboard5x5::coords_from_index(m as usize);
		}

		let mut dispersion = 0;
		for a in 0..3 {
			for b in (a + 1)..3 {
				let (x1, y1) = coords[a];
				let (x2, y2) = coords[b];
				let d = (x1 as i32 - x2 as i32)
					.abs()
					.max((y1 as i32 - y2 as i32).abs());
				dispersion = dispersion.max(d);
			}
		}

		let mut border_score = 0;
		for &(x, y) in &coords {
			let d = x.min(4 - x).min(y.min(4 - y));
			border_score += (4 - d) as i32;
		}

		let mobility = state.legal_moves().len() as i32;

		let score = 20 * dispersion - 15 * pressure as i32 + 10 * border_score + 5 * mobility;

		if p == Player::PLAYER1 {
			score as Evaluation
		} else {
			-score as Evaluation
		}
	}
}

// cargo test --release -p three_musketeers game::tests::perft_test -- --nocapture
//depth           count        time        kn/s
//    0               1     400.0ns      2500.0
//    1               8       6.5µs      1230.8
//    2              16       1.7µs      9411.8
//    3             132       2.1µs     62857.1
//    4             736     538.6µs      1366.5
//    5            5540      44.2µs    125339.4
//    6           38740     168.3µs    230184.2
//    7          263040     338.7µs    776616.5
//    8         2325116       2.7ms    856554.1
//    9        14915148      10.1ms   1476148.1
//   10       151944996     152.2ms    998238.0
//   11       914687972     601.8ms   1519838.6
//   12     10525876764        8.9s   1180777.0
#[cfg(test)]
mod tests {

	use kudchuet::ai::minimax::util::perft;

	use crate::rules::ThreeMusketeers;
	#[test]
	fn perft_test() {
		println!("BMI1 enabled? {}", cfg!(target_feature = "bmi1"));
		let mut board = ThreeMusketeers::new();

		let max_depth = 12;
		let nodes = perft::<ThreeMusketeers>(&mut board, max_depth, true);
		assert!(nodes.len() == (max_depth + 1) as usize);

		const NB_NODES: [u64; 13] = [
			1,
			8,
			16,
			132,
			736,
			5540,
			38740,
			263040,
			2325116,
			14915148,
			151944996,
			914687972,
			10525876764,
		];

		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}
