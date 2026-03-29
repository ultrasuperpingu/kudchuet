use shakmaty::{KnownOutcome, Position};



pub fn evaluate_materials(state: &dyn Position) -> minimax::Evaluation {
	if let Some(known) = state.outcome().known() {
		if known == KnownOutcome::Draw {
			0 as minimax::Evaluation
		} else if known.winner() == Some(state.turn()) {
			minimax::BEST_EVAL
		} else {
			minimax::WORST_EVAL
		}
	} else {
		let mypieces = state.board().material_side(state.turn());
		let theirpieces = state.board().material_side(state.turn().other());
		let mut eval = mypieces.pawn as minimax::Evaluation * 100 + mypieces.knight  as minimax::Evaluation * 300 + mypieces.bishop as minimax::Evaluation * 320 + mypieces.rook as minimax::Evaluation * 500 + mypieces.queen as minimax::Evaluation * 900;
		eval -= theirpieces.pawn as minimax::Evaluation * 100 + theirpieces.knight as minimax::Evaluation * 300 + theirpieces.bishop as minimax::Evaluation * 320 + theirpieces.rook as minimax::Evaluation * 500 + theirpieces.queen as minimax::Evaluation * 900;

		eval
	}
}

pub fn evaluate_attacks(state: &dyn Position) -> minimax::Evaluation {
	let mut eval = 0;		
	for (sq, _p) in state.board() {
		eval+= state.board().attacks_to(sq, state.turn(), state.board().by_color(!state.turn())).count() as i16;
		eval-= state.board().attacks_to(sq, !state.turn(), state.board().by_color(state.turn())).count() as i16;
	}
	eval
}

pub fn evaluate_kings_to_center(_state: &dyn Position) -> minimax::Evaluation {
	let eval = 0;		
	//TODO
	#[allow(clippy::let_and_return)]
	eval
}