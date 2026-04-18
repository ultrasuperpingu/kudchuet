use kudchuet::ai::minimax::{BEST_EVAL, Evaluation, WORST_EVAL};
use shakmaty::{KnownOutcome, Position};



pub fn evaluate_materials(state: &dyn Position) -> Evaluation {
	if let Some(known) = state.outcome().known() {
		if known == KnownOutcome::Draw {
			0 as Evaluation
		} else if known.winner() == Some(state.turn()) {
			BEST_EVAL
		} else {
			WORST_EVAL
		}
	} else {
		let mypieces = state.board().material_side(state.turn());
		let theirpieces = state.board().material_side(state.turn().other());
		let mut eval = mypieces.pawn as Evaluation * 100 + mypieces.knight  as Evaluation * 300 + mypieces.bishop as Evaluation * 320 + mypieces.rook as Evaluation * 500 + mypieces.queen as Evaluation * 900;
		eval -= theirpieces.pawn as Evaluation * 100 + theirpieces.knight as Evaluation * 300 + theirpieces.bishop as Evaluation * 320 + theirpieces.rook as Evaluation * 500 + theirpieces.queen as Evaluation * 900;

		eval
	}
}

pub fn evaluate_attacks(state: &dyn Position) -> Evaluation {
	let mut eval = 0;		
	for (sq, _p) in state.board() {
		eval+= state.board().attacks_to(sq, state.turn(), state.board().by_color(!state.turn())).count() as i16;
		eval-= state.board().attacks_to(sq, !state.turn(), state.board().by_color(state.turn())).count() as i16;
	}
	eval
}

pub fn evaluate_kings_to_center(_state: &dyn Position) -> Evaluation {
	let eval = 0;		
	//TODO
	#[allow(clippy::let_and_return)]
	eval
}