//use std::hash::{DefaultHasher, Hash, Hasher};

use kudchuet::{GameOutcome, Player};

use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

use super::rules::{Clobber, Move};

impl Game for Clobber {
	type S = Clobber;
	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		*moves = state.legal_moves();
		Self::get_outcome(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s=state.clone();
		s.play_unchecked(m);
		Some(s)
	}

	fn notation(_state: &Self::S, mv: Self::M) -> Option<String> {
		Some(format!("{:?}", mv))
	}

	fn get_hash(state: &Self::S) -> u64 {
		//let mut hasher = DefaultHasher::new();
		//state.hash(&mut hasher);
		//hasher.finish()
		state.get_hash()
	}
	fn get_current_player(state: &Self::S) -> Player {
		if state.is_black { Player::PLAYER2 } else { Player::PLAYER1 }
	}
	fn get_outcome(state: &Self::S) -> GameOutcome {
		state.result()
	}
}
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ClobberDumbEval {
	turn:Player
}

impl Evaluator for ClobberDumbEval {
	type G = Clobber;

	fn evaluate_for(&self, _state: &Clobber, _p: Player) -> Evaluation {
		0
	}
}
