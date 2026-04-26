

use crate::rules::Cell;

use super::rules::{Column, ConnectFour};
use kudchuet::{GameOutcome, Player};
use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

impl Game for ConnectFour {
	type S =  ConnectFour;

	type M = Column;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		let mut mvs: [Column;7] = [Column::from_index(0);7];
		let nb = state.legal_moves_array(&mut mvs);
		moves.extend_from_slice(&mvs[0..nb]);
		// TODO: check winner
		Self::get_outcome(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		state.play_unchecked(m);
		None
	}
	fn undo(state: &mut Self::S, m: Self::M) {
		state.undo_unchecked(m);
	}
	fn notation(_state: &Self::S, mv: Self::M) -> Option<String> {
		Some(mv.0.to_string())
	}
	fn get_outcome(state: &Self::S) -> GameOutcome {
		if state.is_victory() {
			match state.player_turn() {
				Cell::Empty => unreachable!(),
				Cell::PlayerOne => GameOutcome::PLAYER2,
				Cell::PlayerTwo => GameOutcome::PLAYER1,
			}
		} else if state.is_over() {
			GameOutcome::Draw
		} else {
			GameOutcome::OnGoing
		}
	}
	fn get_current_player(state: &Self::S) -> Player {
		
		match state.player_turn() {
			Cell::Empty => panic!(),
			Cell::PlayerOne => Player::PLAYER1,
			Cell::PlayerTwo => Player::PLAYER2,
		}
	}
	fn get_hash(state: &Self::S) -> u64 {
		state.encode()
	}
}


#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ConnectFourEval;

impl ConnectFourEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for ConnectFourEval {
	type G = ConnectFour;
	fn evaluate_for(&self, state: &ConnectFour, _p: Player) -> Evaluation {
		//TODO
		state.heuristic() as i16 - state.opponent_heuristic() as i16 as Evaluation
	}
}
#[cfg(test)]
mod tests {
	use kudchuet::ai::minimax::util::perft;

use super::ConnectFour;

	// cargo test --release -p connect4 game::tests::perft_test -- --nocapture

	//depth           count        time        kn/s
	//    0               1       1.8µs       555.6
	//    1               7      11.0µs       636.4
	//    2              49     700.0ns     70000.0
	//    3             343       3.2µs    107187.5
	//    4            2401     333.2µs      7205.9
	//    5           16807     201.4µs     83450.8
	//    6          117649     363.0µs    324101.9
	//    7          823536       1.7ms    487184.1
	//    8         5673234       8.6ms    658430.4
	//    9        39394572      58.0ms    678869.1
	//   10       268031646     382.3ms    701057.8
	//   11      1844590828        2.8s    649762.8
	#[test]
	fn perft_test() {
		let mut board = ConnectFour::default();
		let max_depth = 13;
		let nodes = perft::<ConnectFour>(&mut board, max_depth, true);
		const NB_NODES: [u64;14] = [
			1,
			7,
			49,
			343,
			2401,
			16807,
			117649,
			823536,
			5673234,
			39394572,
			268031646,
			1844590828,
			12418296244,
			84496181330,
		];
		for (i, n) in nodes.iter().enumerate() {
			if i >= NB_NODES.len() {
				break;
			}
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}