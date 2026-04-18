use crate::ai::minimax::interface::Game;

use crate::gui::{BoardGame, BoardMove};
use crate::gui::board_drawer::BoardDrawer;
use crate::gui::game_state_manager::GameStateManager;



pub enum MoveResult<G: Game> {
	Invalid,
	Incomplete {
		selected : Option<u16>,
		highlights: Vec<u16>,
		matching_moves: Vec<G::M>,
		//intermediate_state: Option<G::S>
	},
	Created {
		mv: G::M,
		highlights_played: Vec<u16>,
	},
	ChoiceRequired {
		candidates: Vec<G::M>,
	},
}


pub struct InputHandler<G: Game> {
	clicks: Vec<u16>,
	pending_moves: Option<Vec<G::M>>,
	matching_moves: Vec<G::M>,
	intermediate_state: Option<G>
}
impl<G: Game> Default for InputHandler<G> {
	fn default() -> Self {
		Self::new()
	}
}
impl<G: Game> InputHandler<G> {
	pub fn new() -> Self {
		Self { clicks: Vec::new(), pending_moves:None, matching_moves: vec![], intermediate_state:None }
	}

	pub fn current_clicks(&self) -> &Vec<u16> {
		&self.clicks
	}

	pub fn pending_moves(&self) -> Option<&Vec<G::M>> {
		self.pending_moves.as_ref()
	}
	pub fn set_pending_moves(&mut self, moves: Vec<G::M>) {
		self.pending_moves = Some(moves);
	}

	pub fn clear_pending_moves(&mut self) {
		self.pending_moves = None
	}

	pub fn matching_moves(&self) -> &Vec<G::M> {
		&self.matching_moves
	}

	pub fn intermediate_state(&self) -> &Option<G> {
		&self.intermediate_state
	}
	pub fn set_intermediate_state(&mut self, is: Option<G>) {
		self.intermediate_state = is;
	}
	pub fn process(
		&mut self,
		click_pos: (u8, u8),
		game_manager: &GameStateManager<G>,
		drawer: &mut Box<dyn BoardDrawer<G>>,
	) -> MoveResult<G>
	where
		G: BoardGame,
		G::M: BoardMove<G>,
	{
		let (x, y) = click_pos;
		let index = G::index_from_coords(x, y);
		println!("index: {}", index);

		self.clicks.push(index);

		let custom_result = G::M::handle_clicks_interaction(
			game_manager.game(),
			game_manager.legal_moves(),
			&self.clicks,
		);

		match &custom_result {
			MoveResult::Created { highlights_played, .. } => {
				self.reset(drawer, game_manager);
				drawer.set_played_highlights(highlights_played.clone());
				custom_result
			}
			MoveResult::Incomplete { selected, highlights, matching_moves/*, intermediate_state*/ } => {
				drawer.set_selected(selected.clone());
				drawer.set_legal_highlights(highlights.clone());
				self.matching_moves = matching_moves.clone();
				if !self.matching_moves.is_empty() {
					self.intermediate_state = self.matching_moves[0].compute_intermediate_state(game_manager.game(), &self.clicks).clone();
				}
				custom_result
			}
			MoveResult::Invalid => {
				self.reset(drawer, game_manager);
				MoveResult::Invalid
			},
			MoveResult::ChoiceRequired { candidates } => {
				self.pending_moves = Some(candidates.clone());
				custom_result
			}
		}
	}

	pub fn reset(&mut self, drawer: &mut Box<dyn BoardDrawer<G>>, game_manager: &GameStateManager<G>,)
	where
		G: BoardGame,
		G::M: BoardMove<G>,
	{
		self.clicks.clear();
		self.pending_moves = None;
		self.intermediate_state=None;
		self.matching_moves = game_manager.legal_moves().iter()
					.filter(|m| m.click_sequence(game_manager.game()) == self.clicks).copied().collect();
		drawer.clear_selection();
	}
}