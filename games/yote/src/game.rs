


use kudchuet::{GameResult, Player, ai::minimax::{Evaluation, Evaluator, Game, Winner}};

use super::rules::{Move, Yote};




impl Game for Yote {
	type S =  Yote;

	type M = Move;

	#[inline]
	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> Option<Winner> {
		state.legal_moves_inplace(moves);
		Self::get_winner(state)
	}

	#[inline]
	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s2 = state.clone();
		s2.play(m);
		Some(s2)
	}

	fn get_winner(state: &Self::S) -> Option<Winner> {
		match state.result() {
			GameResult::Draw => Some(Winner::Draw),
			GameResult::OnGoing => None,
			GameResult::Player(p) => {
				Some(Winner::Player(p))
			},
		}
	}
	fn current_player(state: &Self::S) -> Player {
		state.turn
	}
	#[inline]
	fn zobrist_hash(state: &Self::S) -> u64 {
		//let mut hasher = DefaultHasher::new();
		//state.hash(&mut hasher);
		//hasher.finish()
		state.get_hash()
		//state.compute_hash()
	}
}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct YoteDumbEval;

impl YoteDumbEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for YoteDumbEval {
	type G = Yote;
	fn evaluate_for(&self, _state: &Yote, _p: Player) -> Evaluation {
		0 as Evaluation
	}
}
#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct YoteMaterialEval;

impl YoteMaterialEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Evaluator for YoteMaterialEval {
	type G=Yote;
	fn evaluate_for(&self, state: &Yote, p: Player) -> Evaluation {
		if p == Player::PLAYER1 {
			state.white_pawns_count()as Evaluation - state.black_pawns_count()as Evaluation 
		} else {
			state.black_pawns_count() as Evaluation - state.white_pawns_count() as Evaluation
		}
	}
}
// cargo test --release -p yote game::tests::simple_perft_test -- --nocapture
// depth           count        time        kn/s
//     0               1       1.6µs       625.0
//     1              30       1.7µs     17647.1
//     2             870       7.7µs    112987.0
//     3           27188      32.9µs    826383.0
//     4          829062     639.1µs   1297233.6
//     5        26864614       7.8ms   3460374.1
//     6       859118308     185.0ms   4644897.1
//     7     28919879036        6.8s   4252708.9

// cargo test --release -p yote game::tests::mandatory_perft_test -- --nocapture
// depth           count        time        kn/s
//     0               1       1.3µs       769.2
//     1              30       3.0µs     10000.0
//     2             870       4.4µs    197727.3
//     3            2642      18.8µs    140531.9
//     4            9954     361.5µs     27535.3
//     5           36864     316.4µs    116510.7
//     6          131846     502.2µs    262536.8
//     7          485740       1.6ms    300117.4
//     8         1759482       5.6ms    314896.1
//     9         6443332      13.7ms    469076.7
//    10        23478556      40.5ms    580087.0
//    11        85873122     157.6ms    544792.4
//    12       313402418     569.4ms    550391.5
//    13      1146228466        2.1s    547945.6
#[cfg(test)]
mod tests {
	use kudchuet::ai::minimax::util::perft;
	use crate::rules::Yote;

	#[test]
	fn simple_perft_test() {
		println!("BMI1 enabled? {}", cfg!(target_feature = "bmi1"));
		let mut board = Yote::new();

		let _nodes = perft::<Yote>(&mut board, 7, true);
	}
	#[test]
	fn mandatory_perft_test() {
		println!("BMI1 enabled? {}", cfg!(target_feature = "bmi1"));
		let mut board = Yote::new();
		board.rules.set_mandatory_takes(true);
		println!("{}", board.rules.mandatory_takes());

		let _nodes = perft::<Yote>(&mut board, 13, true);
	}
}