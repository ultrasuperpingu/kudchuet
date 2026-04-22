
use bitboard::BitIter;
use kudchuet::Player;
use kudchuet::ai::minimax::{Evaluation, Evaluator, Game, Winner};

use super::rules::{Move, Diaballik};

impl Game for Diaballik {
	type S =  Diaballik;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<Winner> {
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
	fn get_winner(state: &Self::S) -> Option<Winner> {
		if state.turn == Player::PLAYER2 && state.ball_player1 > 41 {
			Some(Winner::PLAYER1)
		}
		else if state.turn == Player::PLAYER1 && state.ball_player2 < 7 {
			Some(Winner::PLAYER2)
		} else {
			// Anti-Game (Blocking) Rules
			if state.is_blocking(Player::PLAYER1) { return Some(Winner::PLAYER2); }
			if state.is_blocking(Player::PLAYER2) { return Some(Winner::PLAYER1); }
			None
		}
	}
	fn current_player(state: &Self::S) -> Player {
		state.turn()
	}

	fn zobrist_hash(state: &Self::S) -> u64 {
		state.hash
	}
}


#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct DiaballikEvalMaterial;

impl DiaballikEvalMaterial {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for DiaballikEvalMaterial {
	type G = Diaballik;
	fn evaluate_for(&self, state: &Diaballik, p: Player) -> Evaluation {
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
	use kudchuet::ai::minimax::{Game, IterativeOptions, Strategy};
	#[cfg(target_arch = "wasm32")]
	use kudchuet::ai::minimax::IterativeSearch;
	#[cfg(not(target_arch = "wasm32"))]
	use kudchuet::ai::minimax::{ParallelSearch, ParallelOptions};
	use super::Diaballik;
	use super::DiaballikEvalMaterial;
	//use super::DiaballikEvalDumb;

	// cargo test --release -p diaballik game::tests::perft_test -- --nocapture

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
		let nodes = perft_tt::<Diaballik>(&mut board, max_depth, false);
		for (i, n) in nodes.iter().enumerate() {
			if NB_NODES.len() <= i { break; }
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
		
	}
	#[test]
	//#[cfg(not(target_arch = "wasm32"))]
	fn test_walk_full_game() {
		let mut state = Diaballik::default();
		let mut turn_count = 0;
		let mut strategy = {
			#[cfg(not(target_arch = "wasm32"))]
			{
				ParallelSearch::new(
					DiaballikEvalMaterial::new(),
					IterativeOptions::new(),
					ParallelOptions::new(),
				)
			}

			#[cfg(target_arch = "wasm32")]
			{
				IterativeSearch::new(
					DiaballikEvalMaterial::new(),
					IterativeOptions::new(),
				)
			}
		};
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
