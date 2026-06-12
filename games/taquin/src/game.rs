use kudchuet::{
	Player,
	ai::move_search::{Game, astar::Heuristic},
	gui::{BoardMove, GUIGame, GUIMove},
};

use crate::rules::{Move, Taquin};

impl<const W: usize, const H: usize, const NB: usize> GUIMove<Taquin<W, H, NB>> for Move {
	fn click_sequence(
		&self,
		_state: &Taquin<W, H, NB>,
	) -> Vec<<Taquin<W, H, NB> as GUIGame>::Click> {
		self.click_sequence_board_move_default(_state)
	}
}
impl<const W: usize, const H: usize, const NB: usize> Game for Taquin<W, H, NB> {
	type S = Self;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> kudchuet::GameOutcome {
		*moves = state.legal_moves();
		Self::get_outcome(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s = state.clone();
		s.play_unchecked(m);
		Some(s)
	}

	fn get_outcome(state: &Self::S) -> kudchuet::GameOutcome {
		if *state == Self::SOLVED {
			kudchuet::GameOutcome::PLAYER1
		} else {
			kudchuet::GameOutcome::OnGoing
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
impl<const W: usize, const H: usize, const NB: usize> Heuristic for Taquin<W, H, NB> {
	type G = Taquin<W, H, NB>;

	fn heuristic(&self, state: &<Self::G as Game>::S) -> u32 {
		state.manhattan_with_linear_conflict() as u32
	}
}
pub struct TaquinSettings {
	pub width: u8,
	pub height: u8,
}
