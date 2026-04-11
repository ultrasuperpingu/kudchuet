//use std::hash::{DefaultHasher, Hash, Hasher};

//use minimax::Evaluation;


use bitboard::BitIter;
use minimax::Evaluation;

use kudchuet::common::{GameResult, Player};

use super::rules::{Move, Diaballik};

impl minimax::Game for Diaballik {
	type S =  Diaballik;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<minimax::Winner> {
		if let Some(w) = Self::get_winner(state) {
			return Some(w);
		}
		
		state.legal_moves(moves);
		None
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		// both give the exact same benchmark on perft
		//let mut s=*state;
		//s.play_unchecked(&m);
		//Some(s)
		state.play_unchecked(&m);
		None
	}
	fn undo(_state: &mut Self::S, _m: Self::M) {
		_state.undo_unchecked(&_m);
	}
	fn get_winner(state: &Self::S) -> Option<minimax::Winner> {
		match state.result() {
			GameResult::Player1 => if state.turn() == Player::PLAYER2 {Some(minimax::Winner::PlayerJustMoved)} else {Some(minimax::Winner::PlayerToMove)},
			GameResult::Player2 => if state.turn() == Player::PLAYER1 {Some(minimax::Winner::PlayerJustMoved)} else {Some(minimax::Winner::PlayerToMove)},
			GameResult::Player(_) => unreachable!(),
			GameResult::Draw => unreachable!(),
			GameResult::OnGoing => None,
		}
	}

	fn zobrist_hash(state: &Self::S) -> u64 {
		//let mut hasher = DefaultHasher::new();
		//state.hash(&mut hasher);
		//hasher.finish()
		state.hash
	}
}


#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct DiaballikEvalMaterial;

impl DiaballikEvalMaterial {
	pub fn new() -> Self {
		Self
	}
}
impl minimax::Evaluator for DiaballikEvalMaterial {
	type G = Diaballik;
	fn evaluate(&self, state: &Diaballik) -> Evaluation {
		let mut score = 0;
		
		let b1_y = state.ball_player1 / 7;
		let b2_y = state.ball_player2 / 7;
		
		score += b1_y as Evaluation * 100;
		score -= (6 - b2_y as Evaluation) * 100;
		for p in state.player1.iter_bits() {
			score += (p/7) as Evaluation * 10;
		}
		for p in state.player2.iter_bits() {
			score -= (6-p/7) as Evaluation * 10;
		}
		if state.turn == Player::PLAYER1 {
			score
		} else {
			-score
		}
	}
}
#[cfg(test)]
mod tests {

	use minimax::{Game, IterativeOptions, Strategy};
	
	#[cfg(not(target_arch = "wasm32"))]
	use minimax::{ParallelSearch, ParallelOptions};
	use super::Diaballik;
	use super::DiaballikEvalMaterial;
	//use super::DiaballikEvalDumb;

	// cargo test --release diaballik::game::tests::perft_test -- --nocapture

	//depth           count        time        kn/s
	//    0               1       5.2µs       192.3
	//    1            1129      35.2µs     32073.9
	//    2         1274641       2.3ms    561664.3
	//    3      2488349845        1.1s   2192698.6
	//    4   4857129629421       38.5s 126093962.6
	#[test]
	fn perft_test() {
		let mut board = Diaballik::default();
		let max_depth = 4;
		//let nodes = perft::<Diaballik>(&mut board, max_depth, true);
		const NB_NODES: [u64;5] = [
			1,
			1129,
			1274641,
			2488349845,
			4857129629421,
		];
		let nodes = minimax::util::perft_tt::<Diaballik>(&mut board, max_depth, false);
		for (i, n) in nodes.iter().enumerate() {
			if NB_NODES.len() <= i { break; }
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
		
	}
	#[test]
	#[cfg(not(target_arch = "wasm32"))]
	fn test_walk_full_game() {
		let mut state = Diaballik::default();
		let mut turn_count = 0;
		let mut strategy = ParallelSearch::new(DiaballikEvalMaterial, IterativeOptions::new(), ParallelOptions::new());
		strategy.set_max_depth(3);
		println!("Initial state:\n{}", state);

		while turn_count < 200 && !state.result().is_finished() {
			let chosen_move = strategy.choose_move(&state);

			Diaballik::apply(&mut state, chosen_move.unwrap());

			turn_count += 1;

			if turn_count % 10 == 0 {
				println!("Turn {}\n{}", turn_count, state);
			}
		}
		println!("Winner: {:?}\n{}", state.result(), state);
	}
}
