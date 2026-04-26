use std::hash::{DefaultHasher, Hash, Hasher};

use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};


use kudchuet::{GameOutcome, Player};

use super::rules::{Move, FootBoard};


impl Game for FootBoard {
	type S =  FootBoard;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		let res = Self::get_outcome(state);
		if res.is_ended() {
			return res;
		}
		state.legal_moves(moves);
		GameOutcome::OnGoing
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s=*state;
		s.play_unchecked(&m);
		Some(s)
	}
	fn get_outcome(state: &Self::S) -> GameOutcome {
		state.result().into()
	}

	fn get_hash(state: &Self::S) -> u64 {
		let mut hasher = DefaultHasher::new();
		state.hash(&mut hasher);
		hasher.finish()
		//state.compute_hash()
	}
	fn get_current_player(state: &Self::S) -> Player {
		state.turn()
	}
}


#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct FootboardEvalDumb;

impl FootboardEvalDumb {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for FootboardEvalDumb {
	type G = FootBoard;
	fn evaluate_for(&self, state: &FootBoard, p: Player) -> Evaluation {
		let mut score = (state.score1 as Evaluation - state.score2 as Evaluation) * 1000;
		let owner = state.ball_owner();
		if owner == Some(state.turn()) {
			score += 100;
		} else if owner.is_some() {
			score -= 100;
		}
		if p == Player::PLAYER1 {
			score
		} else {
			-score
		}
	}
}
#[cfg(test)]
mod tests {
	
	use kudchuet::ai::minimax::util::perft_tt;
#[cfg(not(target_arch = "wasm32"))]
	use kudchuet::ai::minimax::{Game, IterativeOptions, ParallelOptions, ybw::ParallelSearch, Strategy};
	#[cfg(not(target_arch = "wasm32"))]
	use super::FootboardEvalDumb;

	use super::FootBoard;

	// cargo test --release -p footboard game::tests::perft_test -- --nocapture

	//depth           count        time        kn/s
	//    0               1       3.7µs       270.3
	//    1          354825       4.3ms     82049.9
	//    2     57568319673       78.6s    732295.6
	#[test]
	fn perft_test() {
		let mut board = FootBoard::default();
		let max_depth = 5;
		const NB_NODES: [u64;3] = [
			1,
			354825,
			57568319673,
		];
		//let nodes = perft::<FootBoard>(&mut board, max_depth, true);
		let nodes = perft_tt::<FootBoard>(&mut board, max_depth, false);
		for (i, n) in nodes.iter().enumerate() {
			if NB_NODES.len() <= i { break; }
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
		
	}
	
	#[test]
	#[cfg(not(target_arch = "wasm32"))]
	fn test_walk_full_game() {
		let mut state = FootBoard::default();
		let mut turn_count = 0;
		let mut strategy = ParallelSearch::new(FootboardEvalDumb::new(), IterativeOptions::new(), ParallelOptions::new());
		strategy.set_max_depth(2);
		println!("Initial state:\n{}", state);

		while turn_count < 200 && !state.result().is_ended() {
			let chosen_move = strategy.choose_move(&state);

			FootBoard::apply(&mut state, chosen_move.unwrap());

			turn_count += 1;

			if turn_count % 10 == 0 {
				println!("Turn {}\n{}", turn_count, state);
			}
		}
		println!("Winner: {:?}\n{}", state.result(), state);
	}
}
