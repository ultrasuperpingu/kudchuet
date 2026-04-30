use std::hash::{DefaultHasher, Hash, Hasher};

use kudchuet::{GameOutcome, Player};

use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

use super::rules::{EinsteinWurfeltNicht, MovePlay};

impl Game for EinsteinWurfeltNicht {
	type S = EinsteinWurfeltNicht;
	type M = MovePlay;

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
		let mut hasher = DefaultHasher::new();
		state.hash(&mut hasher);
		hasher.finish()
		//state.hash
	}
	fn get_current_player(state: &Self::S) -> Player {
		if state.is_red { Player::PLAYER2 } else { Player::PLAYER1 }
	}
	fn get_outcome(state: &Self::S) -> GameOutcome {
		state.result()
	}
	fn is_random_move(state: &Self::S) -> bool {
		state.dice.is_none()
	}

	fn get_probability(_state: &Self::S, _mv: Self::M) -> f32 {
		1.0/6.0
	}
}
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct EinsteinWurfeltNichtDumbEval {
	turn:Player
}

impl Evaluator for EinsteinWurfeltNichtDumbEval {
	type G = EinsteinWurfeltNicht;

	fn evaluate_for(&self, _state: &EinsteinWurfeltNicht, _p: Player) -> Evaluation {
		0
	}
}
