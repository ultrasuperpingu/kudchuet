use kudchuet::{
	Player,
	ai::move_search::{Game, astar::Heuristic},
	gui::{BoardMove, GUIGame, GUIMove},
};

use crate::{
	bitboard::SolitaireBoard,
	rules::{Move, Solitaire},
};

impl GUIMove<Solitaire> for Move {
	fn click_sequence(&self, _state: &Solitaire) -> Vec<<Solitaire as GUIGame>::Click> {
		self.click_sequence_board_move_default(_state)
	}
}
impl Game for Solitaire {
	type S = Self;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> kudchuet::GameOutcome {
		if state.board.count() == 1 {
			return kudchuet::GameOutcome::PLAYER1;
		}
		state.legal_moves_inplace(moves);
		if moves.is_empty() {
			kudchuet::GameOutcome::PLAYER2
		} else {
			kudchuet::GameOutcome::OnGoing
		}
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s = state.clone();
		s.play_unchecked(m);
		Some(s)
	}

	fn get_outcome(state: &Self::S) -> kudchuet::GameOutcome {
		if state.board.count() == 1 {
			if state.board == SolitaireBoard::CENTER {
				kudchuet::GameOutcome::PLAYER1
			} else {
				kudchuet::GameOutcome::Draw
			}
		} else {
			if state.legal_moves().is_empty() {
				kudchuet::GameOutcome::PLAYER2
			} else {
				kudchuet::GameOutcome::OnGoing
			}
		}
	}

	fn get_current_player(_state: &Self::S) -> kudchuet::Player {
		Player::PLAYER1
	}
	fn get_next_player(_state: &Self::S) -> Player {
		Player::PLAYER1
	}
	fn get_hash(state: &Self::S) -> u64 {
		//state.get_hash()
		state.compute_hash()
	}
}
impl Heuristic for Solitaire {
	type G = Solitaire;

	fn heuristic(&self, state: &Solitaire) -> u32 {
		state.board.count()
		//0
	}
}
