use kudchuet::{
	Player,
	ai::move_search::{Evaluator, Game},
	gui::GUIGame,
};

use crate::rules::{Freecell, Move, max_ordered_suffix, rank};

impl Game for Freecell {
	type S = Self;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> kudchuet::GameOutcome {
		state.legal_moves_inplace(moves);
		if moves.len() == 0 {
			if state
				.tableau
				.iter()
				.all(|a| a.is_empty() && state.free_cells.iter().all(|c| c.is_none()))
			{
				kudchuet::GameOutcome::PLAYER1
			} else {
				kudchuet::GameOutcome::PLAYER2
			}
		} else {
			kudchuet::GameOutcome::OnGoing
		}
	}
	fn filter_moves(_state: &Self::S, moves: &mut Vec<Self::M>) {
		// todo: retain only one move to empty column
		let mut seen = [false; 8];

		moves.retain(|m| match m {
			Move::TableauToFreeCell { from, .. } => {
				if seen[*from] {
					false
				} else {
					seen[*from] = true;
					true
				}
			}
			_ => true,
		});
	}
	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		state.play_unchecked(m);
		None
	}
	fn undo(state: &mut Self::S, m: Self::M) {
		state.undo_unchecked(m);
	}

	fn get_outcome(state: &Self::S) -> kudchuet::GameOutcome {
		let moves = state.legal_moves();
		if moves.len() == 0 {
			if state
				.tableau
				.iter()
				.all(|a| a.is_empty() && state.free_cells.iter().all(|c| c.is_none()))
			{
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
	fn notation(_state: &Self::S, _move: Self::M) -> Option<String> {
		match _move {
			Move::TableauToTableau { from, to, count } => {
				Some(format!("{}-{} ({})", from, to, count))
			}
			Move::TableauToFreeCell { from, cell } => Some(format!("{}-free({})", from, cell)),
			Move::FreeCellToTableau { cell, to } => Some(format!("free({})-{}", cell, to)),
			Move::TableauToFoundation {
				from,
				foundation: _,
			} => Some(format!("{}-f", from)),
			Move::FreeCellToFoundation {
				cell,
				foundation: _,
			} => Some(format!("free({})-f", cell)),
			Move::Finish => Some("end".into()),
		}
	}

	fn generate_and_filter_moves(
		state: &Self::S,
		moves: &mut Vec<Self::M>,
	) -> kudchuet::GameOutcome {
		let res = Self::generate_moves(state, moves);
		Self::filter_moves(state, moves);
		res
	}

	fn null_move(_state: &Self::S) -> Option<Self::M> {
		None
	}

	fn table_index(_: Self::M) -> u16 {
		0
	}

	fn max_table_index() -> u16 {
		0
	}

	fn is_random_move(_state: &Self::S) -> bool {
		false
	}

	fn get_probability(_state: &Self::S, _mv: Self::M) -> f32 {
		0.0
	}
}

#[derive(Eq, PartialEq, Debug, Default, Copy, Clone)]
pub struct FreecellEval;
impl Evaluator for FreecellEval {
	type G = Freecell;

	fn evaluate_for(
		&self,
		s: &<Self::G as Game>::S,
		_p: Player,
	) -> kudchuet::ai::move_search::Evaluation {
		let foundation_count: usize = s.foundations.iter().map(|c| c.len()).sum();
		let foundation_mean = foundation_count as f32 / 4.0;

		let mut foundation_variability: f32 = s
			.foundations
			.iter()
			.map(|c| {
				let l = c.len() as f32;
				(l - foundation_mean) * (l - foundation_mean)
			})
			.sum();
		foundation_variability /= 4.0;
		let max_movable = s.max_movables_cards();
		let mut misorder: i16 = 0;
		let mut ordered_bonus: i16 = 0;
		for column in &s.tableau {
			if column.is_empty() {
				continue;
			}
			let ordered = max_ordered_suffix(column);
			let unordered_count = column.len() - ordered;
			misorder += unordered_count as i16 * 11;
			ordered_bonus += ordered as i16 * 5;
			if unordered_count == 0 {
				let rank = rank(column.0[0]);
				ordered_bonus += rank as i16 * 13;
			}
			let unordered = &column.0[0..unordered_count];
			for (i, c) in unordered.iter().enumerate() {
				let rank = rank(*c);
				if rank < 6 {
					misorder += (ordered + i) as i16;
				}
			}
		}
		if misorder == 0 {
			ordered_bonus = 0;
		}
		max_movable.min(12) as i16 * 71 + foundation_count as i16 * 191
			- (foundation_variability * 23.0) as i16
			- misorder as i16 * 3
			+ ordered_bonus
	}
}
