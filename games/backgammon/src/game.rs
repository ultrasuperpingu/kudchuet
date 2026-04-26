//use std::hash::{DefaultHasher, Hash, Hasher};

use kudchuet::{GameOutcome, Player};

use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

use super::rules::{Backgammon, Move};

impl Game for Backgammon {
	type S = Backgammon;
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
		state.hash
	}
	fn get_current_player(state: &Self::S) -> Player {
		state.current_player()
	}
	fn get_outcome(state: &Self::S) -> GameOutcome {
		if state.is_game_over() {
			if let Some(winner) = state.winner() {
				GameOutcome::Player(winner)
			} else {
				GameOutcome::Draw
			}
		} else {
			GameOutcome::OnGoing
		}
	}
	fn is_random_move(state: &Self::S) -> bool {
		state.dice.is_empty()
	}

	fn get_probability(_state: &Self::S, _mv: Self::M) -> f32 {
		1.0/36.0
	}
}
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct BackgammonMaterialEval {
	turn:Player
}

impl Evaluator for BackgammonMaterialEval {
	type G = Backgammon;

	fn evaluate_for(&self, state: &Backgammon, p: Player) -> Evaluation {
		if p == Player::PLAYER1 {
			state.outside[0] as Evaluation - state.outside[1] as Evaluation
		} else {
			state.outside[1] as Evaluation - state.outside[0] as Evaluation
		}
	}
}
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct BackgammonSimpleEval;
const CONSECUTIVE_BONUS: i16 = 19;
const ON_BAR_MALUS: i16 = 71;
const OUT_BONUS: i16 = 103;

impl Evaluator for BackgammonSimpleEval {
	type G = Backgammon;

	fn evaluate_for(&self, state: &Backgammon, p: Player) -> Evaluation {
		let mut scorep1 = 0 as Evaluation;
		let mut scorep2 = 0 as Evaluation;
		scorep1 += state.outside[0] as i16 * OUT_BONUS;
		scorep2 += state.outside[1] as i16 * OUT_BONUS;
		scorep1 -= state.on_bar[0] as i16 * ON_BAR_MALUS;
		scorep2 -= state.on_bar[1] as i16 * ON_BAR_MALUS;
		let mut consecutive: i16 = 0;
		for (i,p) in state.board.iter().enumerate() {
			if *p < 0 {
				if consecutive <= 0 {
					consecutive-=1;
				} else {
					apply_consecutive_bonus(&mut scorep1, &mut scorep2, consecutive);
					consecutive = -1;
				}
				if *p == -1 {
					scorep2 -= 25;
				} else {
					scorep2 += -(*p as i16) * (23 - i as i16);
				}
				if *p < -3 {
					scorep2 -= (-*p as i16) - 3;
				}
			} else if *p > 0 {
				if consecutive >= 0 {
					consecutive+=1;
				} else {
					apply_consecutive_bonus(&mut scorep1, &mut scorep2, consecutive);
					consecutive = 1;
				}
				if *p == 1 {
					scorep1 -= 25;
				} else {
					scorep1 += (*p as i16) * (i as i16);
				}
				if *p > 3 {
					scorep1 -= (*p as i16) - 3;
				}
			} else {
				apply_consecutive_bonus(&mut scorep1, &mut scorep2, consecutive);
				consecutive = 0;
			}
		}
		if p == Player::PLAYER1 {
			scorep1 - scorep2
		} else {
			scorep2 - scorep1
		}
	}
}

fn apply_consecutive_bonus(scorep1: &mut i16, scorep2: &mut i16, consecutive: i16) {
	if consecutive.abs() > 2 {
		if consecutive < 0 {
			*scorep2 += (-consecutive-2)*CONSECUTIVE_BONUS;
		} else {
			*scorep1 += (consecutive-2)*CONSECUTIVE_BONUS;
		}
	}
}