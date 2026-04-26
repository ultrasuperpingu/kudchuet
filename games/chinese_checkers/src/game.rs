
use bitboard::BitIter;

use crate::bitboard::ChineseCheckerBoard;
use crate::rules::ChineseCheckersPlayer;
use super::rules::{ChineseCheckers, Move};

use kudchuet::{GameOutcome, Player};
use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

impl Game for ChineseCheckers {
	type S =  ChineseCheckers;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		let res = Self::get_outcome(state);
		if res.is_ended()  {
			return res;
		}
		state.generate_moves(moves);
		GameOutcome::OnGoing
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		state.play_unchecked(m);
		None
	}
	fn undo(state: &mut Self::S, m: Self::M) {
		state.undo_unchecked(m);
	}

	fn get_hash(state: &Self::S) -> u64 {
		state.hash
	}
	fn get_current_player(state: &Self::S) -> Player {
		match state.turn {
			ChineseCheckersPlayer::Red => Player(0),
			ChineseCheckersPlayer::Blue => Player(1),
			ChineseCheckersPlayer::Green => Player(2),
			ChineseCheckersPlayer::Yellow => Player(3),
			ChineseCheckersPlayer::Black => Player(4),
			ChineseCheckersPlayer::White => Player(5),
		}
	}
	fn get_outcome(state: &Self::S) -> GameOutcome {
		match state.winner() {
			Some(w) => GameOutcome::Player(Player(w.idx() as u8)),
			None => GameOutcome::OnGoing,
		}
	}
}
#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct ChineseCheckersMaterialEval;

impl ChineseCheckersMaterialEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for ChineseCheckersMaterialEval {
	type G = ChineseCheckers;

	fn evaluate_for(&self, state: &ChineseCheckers, player: Player) -> Evaluation {
		let mut score = 0;

		for p in ChineseCheckers::active_players(state.nb_players) {
			let b = state.board(*p);
			let target = ChineseCheckerBoard::final_square(*p);
			let target_board = ChineseCheckerBoard::target_board(*p);
			

			let mut dist = 0;

			for i in b.iter_bits() {
				let (x, y) = ChineseCheckerBoard::coords_from_index(i as usize);

				let dx = target.0.abs_diff(x);
				let dy = target.1.abs_diff(y);
				let d = dx.max(dy);

				dist += d as i32;
			}
			let in_target = (b.clone() & target_board.clone()).count() as i32;

			let value = 300 - dist + in_target * 20;

			if Player(p.idx()) == player {
				score += value;
			} else {
				score -= value;
			}
		}

		score as Evaluation
	}
}

#[cfg(test)]
mod tests {


	use kudchuet::ai::minimax::util::perft;

use super::ChineseCheckers;
	//cargo test -p chinese_checkers --release game::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	//    0               1       2.6µs       384.6
	//    1              14      57.9µs       241.8
	//    2             196      30.3µs      6468.6
	//    3            4452     147.4µs     30203.5
	//    4          101124       1.1ms     94579.1
	//    5         2603784      10.9ms    238894.6
	//    6        67043344     288.8ms    232138.1
	//    7      1937526440        6.9s    282148.9
	#[test]
	fn perft_test() {
		let mut board = ChineseCheckers::default();

		let nodes = perft::<ChineseCheckers>(&mut board, 7, true);
		const NB_NODES: [u64; 8] = [
			1,
			14,
			196,
			4452,
			101124,
			2603784,
			67043344,
			1937526440,
		];
		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}