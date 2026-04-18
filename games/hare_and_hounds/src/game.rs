//use std::hash::{DefaultHasher, Hash, Hasher};

use bitboard::BitIter;
use kudchuet::{GameResult, Player};
use bitboard::Bitboard;

use kudchuet::ai::minimax::{Evaluation, Evaluator, Game, Winner};


use crate::rules::{Board, NEIGHBORS_HARE};

use super::rules::{Move, HareAndHounds};


impl Game for HareAndHounds {
	type S =  HareAndHounds;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<Winner> {
		if let Some(w) = Self::get_winner(state) {
			return Some(w);
		}
		let mut mvs = [Move::default();HareAndHounds::MAX_MOVES];
		let mut nb = 0;
		state.legal_moves(&mut mvs, &mut nb);
		moves.extend_from_slice(&mvs[0..nb]);
		//println!("GEN: {:?}", moves);
		None
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s=*state;
		//println!("apply\n{}", state);
		s.play_unchecked(m);
		//println!("apply after\nstate\n{}\ns\n{}", state, s);
		Some(s)
	}
	fn get_winner(state: &Self::S) -> Option<Winner> {
		match state.result() {
			GameResult::Player(p) => Some(Winner::Player(p)),
			GameResult::Draw => unreachable!(),
			GameResult::OnGoing => None,
		}
	}
	fn zobrist_hash(state: &Self::S) -> u64 {
		state.compute_hash()
	}
	
	fn current_player(state: &Self::S) -> Player {
		
		match state.turn() {
			true => Player::PLAYER2,
			false => Player::PLAYER1,
		}
	}
}


#[derive(Clone, Default)]
pub struct HareAndHoundsEvalDumb;

impl HareAndHoundsEvalDumb {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for HareAndHoundsEvalDumb {
	type G = HareAndHounds;
	fn evaluate_for(&self, _state: &HareAndHounds, _p: Player) -> Evaluation {
		0
	}
}
#[inline(always)]
fn manhattan_dist(index1: u8, index2: u8) -> u8 {
	let (x1,y1)=Board::coords_from_index(index1 as usize);
	let (x2,y2)=Board::coords_from_index(index2 as usize);
	x1.abs_diff(x2).max(y1.abs_diff(y2))
}
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct HareAndHoundsEval;

impl HareAndHoundsEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for HareAndHoundsEval {
	type G = HareAndHounds;

	#[inline(always)]
	fn evaluate_for(&self, state: &HareAndHounds, p: Player) -> Evaluation {
		let hare = state.hare;

		// 1. Liberté du lièvre (cases disponibles)
		let hare_moves = (NEIGHBORS_HARE[hare as usize] & !state.houds).count() as i16;

		// 2. Distance moyenne chiens → lièvre (plus c'est petit, mieux pour les chiens)
		let mut dist_sum = 0i16;
		let mut count = 0i16;
		for h in state.houds.iter_bits() {
			dist_sum += manhattan_dist(h as u8, hare) as i16;
			count += 1;
		}
		let avg_dist = if count > 0 { dist_sum / count } else { 0 };

		// 3. Progression du lièvre vers la droite (colonne)
		let hare_col = Board::column(hare) as i16;

		// Score final :
		// + liberté du lièvre
		// + progression du lièvre
		// - proximité des chiens
		// pondéré légèrement
		let score = 10 * hare_moves + 5 * hare_col - 3 * avg_dist;
		if p == Player::PLAYER1 {
			-score
		} else {
			score
		}
	}
}

#[cfg(test)]
mod tests {
	
	use kudchuet::ai::minimax::{IterativeOptions, iterative::IterativeSearch, Strategy, util::perft};
	use crate::game::HareAndHoundsEval;

	use super::HareAndHounds;
	use std::collections::HashMap;

	#[derive(Clone, Copy, PartialEq, Eq, Debug)]
	enum GameResult {
		OnGoing,
		HoundsWin,
		HareWin,
	}

	fn legal_moves(state: &HareAndHounds) -> Vec<HareAndHounds> {
		let mut children=vec![];
		let mut moves= [crate::rules::Move::default(); HareAndHounds::MAX_MOVES];
		let mut len=0;
		children.reserve(9);
		state.legal_moves(&mut moves, &mut len);
		for i in 0..len {
			let m=moves[i];
			let mut c = *state;
			c.play_unchecked(m);
			children.push(c);
		}
		children
	}

	// Solveur itératif avec rétro-propagation
	fn solve_hare_and_hounds() -> GameResult {
		let root = HareAndHounds::default();
		let mut max_moves=0;
		// 1. Exploration DFS itérative
		let mut memo: HashMap<HareAndHounds, GameResult> = HashMap::new();
		let mut stack: Vec<HareAndHounds> = vec![root];

		while let Some(state) = stack.pop() {
			if memo.contains_key(&state) { continue; }
			let res=state.result();
			if res.is_finished() {
				// terminal
				let r = if res.is_player1() { GameResult::HoundsWin } else { GameResult::HareWin };
				memo.insert(state, r);
				continue;
			}
			let children = legal_moves(&state);
			if max_moves < children.len() {
				max_moves=children.len();
			}
			// On marque temporairement OnGoing
			memo.insert(state, GameResult::OnGoing);

			// On empile l'état à revisiter + ses enfants
			stack.push(state);
			for child in children {
				if !memo.contains_key(&child) {
					stack.push(child);
				}
			}
		}
		println!("len: {}",memo.len());
		println!("max_moves: {}",max_moves);
		
		// 2. Rétro-propagation
		let mut changed = true;
		while changed {
			changed = false;
			for (&state, &val) in memo.clone().iter() {
				if val != GameResult::OnGoing { continue; }

				let children = legal_moves(&state);
				let new_val = if state.turn() {
					// lièvre joue
					if children.iter().all(|c| memo[c] == GameResult::HoundsWin) {
						GameResult::HoundsWin
					} else if children.iter().any(|c| memo[c] == GameResult::HareWin) {
						GameResult::HareWin
					}  else {
						continue; // pas encore déterminé
					}
				} else {
					// chiens jouent
					if children.iter().all(|c| memo[c] == GameResult::HareWin) {
						GameResult::HareWin
					} else if children.iter().any(|c| memo[c] == GameResult::HoundsWin) {
						GameResult::HoundsWin
					} else  {
						continue; // pas encore déterminé
					}
				};
				memo.insert(state, new_val);
				changed = true;
			}
		}

		memo[&root]
	}

	#[test]
	fn test_solve() {
		let winner = solve_hare_and_hounds();
		println!("Winner: {:?}", winner);
		assert!(matches!(winner, GameResult::HareWin | GameResult::HoundsWin));
	}

	#[test]
	fn perft_best() {
		let mut board = HareAndHounds::default();
		let mut ai = IterativeSearch::<HareAndHoundsEval>::new(HareAndHoundsEval::new(), IterativeOptions::new());
		ai.set_max_depth(12);
		let _m = ai.choose_move(&board);
		println!("{:?}", ai.principal_variation().len());
		for m in ai.principal_variation() {
			board.play_unchecked(m);
		}
		println!("{}", board)
	}
	// cargo test --release -p hare_and_hounds game::tests::perft_test -- --nocapture

	//depth           count        time        kn/s
	//    0               1       2.6µs       384.6
	//    1               7       9.4µs       744.7
	//    2              21     400.0ns     52500.0
	//    3             133       3.2µs     41562.5
	//    4             443     311.3µs      1423.1
	//    5            2879      85.4µs     33711.9
	//    6            9044      79.7µs    113475.5
	//    7           55384     146.5µs    378047.8
	//    8          169004     386.4µs    437381.0
	//    9          995512       1.4ms    706839.0
	//   10         2968790       4.6ms    642345.0
	//   11        16896668      19.9ms    847401.0
	//   12        49014852      92.6ms    529600.0
	//   13       266512540     315.6ms    844457.2
	//   14       763038916        1.4s    564269.1
	#[test]
	fn perft_test() {
		let mut board = HareAndHounds::default();
		let max_depth = 15;
		let nodes = perft::<HareAndHounds>(&mut board, max_depth, true);
		const NB_NODES: [u64;16] = [
			1,
			7,
			21,
			133,
			443,
			2879,
			9044,
			55384,
			169004,
			995512,
			2968790,
			16896668,
			49014852,
			266512540,
			763038916,
			3975969122
		];
		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}