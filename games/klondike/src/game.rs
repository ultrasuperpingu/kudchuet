use kudchuet::{
	Player,
	ai::move_search::{Evaluator, Game},
};

use crate::rules::{Klondike, Move};

impl Game for Klondike {
	type S = Self;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> kudchuet::GameOutcome {
		*moves = state.legal_moves();
		if moves.len() == 0 {
			if state.foundations.iter().map(|f| f.len()).sum::<usize>() == 52 {
				kudchuet::GameOutcome::PLAYER1
			} else {
				kudchuet::GameOutcome::PLAYER2
			}
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
		let moves = state.legal_moves();
		if moves.len() == 0 {
			if state.foundations.iter().map(|f| f.len()).sum::<usize>() == 52 {
				kudchuet::GameOutcome::PLAYER1
			} else {
				kudchuet::GameOutcome::PLAYER2
			}
		} else {
			kudchuet::GameOutcome::OnGoing
		}
	}
	fn get_hash(state: &Self::S) -> u64 {
		//let mut hasher = DefaultHasher::new();
		//state.hash(&mut hasher);
		//hasher.finish()
		state.h
	}

	fn get_current_player(_state: &Self::S) -> kudchuet::Player {
		Player::PLAYER1
	}
	fn get_next_player(_state: &Self::S) -> Player {
		Player::PLAYER1
	}
}
#[derive(Eq, PartialEq, Debug, Default, Copy, Clone)]
pub struct KlondikeEval;
impl Evaluator for KlondikeEval {
	type G = Klondike;

	fn evaluate_for(
		&self,
		s: &<Self::G as Game>::S,
		_p: Player,
	) -> kudchuet::ai::move_search::Evaluation {
		let foundation_count: usize = s.foundations.iter().map(|c| c.len()).sum();

		let foundation_variability_sum = s
			.foundations
			.iter()
			.map(|c| {
				let d = c.len() as i16 * 4 - foundation_count as i16;
				d * d
			})
			.sum::<i16>();
		let hidden_cards: i16 = s.tableau.iter().map(|c| c.1 as i16).sum::<i16>();
		let empty_columns = s.tableau.iter().filter(|c| c.0.is_empty()).count() as i16;
		let stock_count = s.revealed_stock.len() as i16 + s.stock.len() as i16;

		foundation_count as i16 * 103 - (foundation_variability_sum * 7) as i16 - hidden_cards * 191
			+ empty_columns * 51
			- stock_count * 17
	}
}
