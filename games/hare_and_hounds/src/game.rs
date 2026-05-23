
use bitboard::BitIter;
use kudchuet::{GameOutcome, Player};

use kudchuet::ai::move_search::{Evaluation, Evaluator, Game};


use crate::rules::{Board, NEIGHBORS_HARE};

use super::rules::{Move, HareAndHounds};


impl Game for HareAndHounds {
	type S =  HareAndHounds;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		let res = Self::get_outcome(state);
		if res.is_ended()  {
			return res;
		}
		let mut mvs = [Move::default();HareAndHounds::MAX_MOVES];
		let mut nb = 0;
		state.legal_moves(&mut mvs, &mut nb);
		moves.extend_from_slice(&mvs[0..nb]);
		GameOutcome::OnGoing
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s=*state;
		s.play_unchecked(m);
		Some(s)
	}
	fn get_outcome(state: &Self::S) -> GameOutcome {
		state.result()
	}
	fn get_hash(state: &Self::S) -> u64 {
		state.compute_hash()
	}
	
	fn get_current_player(state: &Self::S) -> Player {
		match state.turn() {
			true => Player::PLAYER2,
			false => Player::PLAYER1,
		}
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

		let hare_moves = (NEIGHBORS_HARE[hare as usize] & !state.houds).count() as i16;

		let mut dist_sum = 0i16;
		let mut count = 0i16;
		for h in state.houds.iter_bits() {
			dist_sum += manhattan_dist(h as u8, hare) as i16;
			count += 1;
		}
		let avg_dist = if count > 0 { dist_sum / count } else { 0 };

		let hare_col = Board::column(hare) as i16;

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
	
	use kudchuet::ai::move_search::{IterativeOptions, Strategy, gametree::GameTree, iterative::IterativeSearch, util::perft};
	use crate::game::HareAndHoundsEval;

	use super::HareAndHounds;
// expand_all/sans pruning
//Winner: Player(Player(0))
//nb states: 23246
//nb nodes: 85602
//outcome: Player(Player(0)) (23)

// expand_all/avec pruning
//Winner: Player(Player(0))
//nb states: 11220
//nb nodes: 30963
//outcome: Player(Player(0)) (67)

// expand_all_iterative/sans pruning
//Winner: Player(Player(0))
//nb states: 23246
//nb nodes: 85602
//outcome: Player(Player(0)) (23)


// expand_all_iterative/avec pruning
//Winner: Player(Player(0))
//nb states: 19677
//nb nodes: 64739
//outcome: Player(Player(0)) (91)

	#[test]
	fn test_solve() {
		let mut tree=GameTree::<HareAndHounds, ()>::from(HareAndHounds::default());
		let winner = tree.expand_all(0);
		println!("Winner: {:?}", winner);
		println!("nb states: {:?}", tree.nb_states());
		println!("nb nodes: {:?}", tree.len());
		println!("outcome: {:?} ({})", tree.get_root().outcome(), tree.get_root().depth_to_end());
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