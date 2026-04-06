
use bitboard::{BitIter, Bitboard};

use crate::common::{GameResult, bitboards::Bitboard5x5};

use super::{Move, ThreeMusketeers};




impl minimax::Game for ThreeMusketeers {
	type S =  ThreeMusketeers;

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
			GameResult::Player1 => {
				if state.turn == 0 {
					Some(minimax::Winner::PlayerToMove)
				} else {
					Some(minimax::Winner::PlayerJustMoved)
				}
			},
			GameResult::Player2 => {
				if state.turn == 0 {
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
		let (x1, y1) = Bitboard5x5::coords_from_index(_move.from as usize);
		let (x2, y2) = Bitboard5x5::coords_from_index(_move.to as usize);
		let file_char1 = (b'a' + x1) as char;
		let rank_char1 = (b'1' + y1) as char;
		let file_char2 = (b'a' + x2) as char;
		let rank_char2 = (b'1' + y2) as char;

		Some(format!("{}{}-{}{}", file_char1, rank_char1, file_char2, rank_char2))
	}

	fn zobrist_hash(state: &Self::S) -> u64 {
		state.get_hash()
	}
	
}

#[derive(Clone, Default, Copy, Debug)]
pub struct ThreeMusketeersEvalDumb;

impl ThreeMusketeersEvalDumb {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for ThreeMusketeersEvalDumb {
	type G = ThreeMusketeers;
	fn evaluate(&self, _state: &ThreeMusketeers) -> minimax::Evaluation {
		0
	}
}
#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct ThreeMusketeersEvalSimple;

impl ThreeMusketeersEvalSimple {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for ThreeMusketeersEvalSimple {
	type G = ThreeMusketeers;
	fn evaluate(&self, state: &ThreeMusketeers) -> minimax::Evaluation {
		let mut nei = 0;
		let mut last = [(0,0);3];
		for (i, m) in state.musketeers.iter_bits().enumerate() {
			nei += state.guards.neighbors_ortho(m as usize).count();
			let coords=Bitboard5x5::coords_from_index(m as usize);
			last[i]=coords;
		}
		let mut max = 0;
		for a in 0..3 {
			for b in (a+1)..3 {
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

		let score = max as i16-nei as i16;
		if state.turn == 1 {
			score
		} else {
			-score
		}
	}
}
#[derive(Clone, Default, Copy)]
pub struct ThreeMusketeersEval2;

impl ThreeMusketeersEval2 {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for ThreeMusketeersEval2 {
	
	type G=ThreeMusketeers;
	fn evaluate(&self, state: &ThreeMusketeers) -> minimax::Evaluation {
		let mut pressure = 0;
		let mut coords = vec![(0,0); 3];

		for (i, m) in state.musketeers.iter_bits().enumerate() {
			pressure += state.guards.neighbors_ortho(m as usize).count();
			coords[i] = Bitboard5x5::coords_from_index(m as usize);
		}

		let mut dispersion = 0;
		for a in 0..3 {
			for b in (a+1)..3 {
				let (x1, y1) = coords[a];
				let (x2, y2) = coords[b];
				let d = (x1 as i32 - x2 as i32).abs()
					.max((y1 as i32 - y2 as i32).abs());
				dispersion = dispersion.max(d);
			}
		}

		let mut border_score = 0;
		for &(x, y) in &coords {
			let d = x.min(4-x).min(y.min(4-y));
			border_score += (4 - d) as i32;
		}

		let mobility = state.legal_moves().len() as i32;

		let score =
			20 * dispersion
		- 15 * pressure as i32
		+ 10 * border_score
		+  5 * mobility;

		if state.turn == 1 {
			score as minimax::Evaluation
		} else {
			-score as minimax::Evaluation
		}
	}
	
}

#[derive(Clone, Default, Copy, PartialEq, Eq)]
pub struct ThreeMusketeersEvalAdvance;

impl ThreeMusketeersEvalAdvance {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for ThreeMusketeersEvalAdvance {
	
	type G=ThreeMusketeers;
	fn evaluate(&self, state: &ThreeMusketeers) -> minimax::Evaluation {
		let mut pressure = 0;
		let mut coords = [(0,0); 3];

		for (i, m) in state.musketeers.iter_bits().enumerate() {
			pressure += state.guards.neighbors_ortho(m as usize).count();
			coords[i] = Bitboard5x5::coords_from_index(m as usize);
		}

		let mut dispersion = 0;
		for a in 0..3 {
			for b in (a+1)..3 {
				let (x1, y1) = coords[a];
				let (x2, y2) = coords[b];
				let d = (x1 as i32 - x2 as i32).abs()
					.max((y1 as i32 - y2 as i32).abs());
				dispersion = dispersion.max(d);
			}
		}

		let mut border_score = 0;
		for &(x, y) in &coords {
			let d = x.min(4-x).min(y.min(4-y));
			border_score += (4 - d) as i32;
		}

		let score =
			40 * dispersion
			+ 30 * border_score
			-  5 * pressure as i32;

		if state.turn == 1 {
			-score as minimax::Evaluation
		} else {
			score as minimax::Evaluation
		}
	}
}

// cargo test --release three_musketeers::game::tests::perft_test -- --nocapture
//    0               1       2.5µs       400.0
//    1               8       3.3µs      2424.2
//    2              16     900.0ns     17777.8
//    3             132       2.1µs     62857.1
//    4             736     291.6µs      2524.0
//    5            5540      82.0µs     67561.0
//    6           38740     139.6µs    277507.2
//    7          263040     410.1µs    641404.5
//    8         2326184       4.0ms    574849.0
//    9        14924066      18.9ms    787907.2
//   10       152389642     150.9ms   1009541.2
//   11       917688274     769.9ms   1191952.3
//   12     10611440635        9.3s   1135164.4
#[cfg(test)]
mod tests {

	use super::super::ThreeMusketeers;
	use minimax::perft;
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
			2326184,
			14924066,
			152389642,
			917688274,
			10611440635,
		];

		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}