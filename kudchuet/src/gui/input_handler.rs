use crate::ai::move_search::interface::Game;

use crate::gui::board_drawer::{BoardDrawer, GameDrawer};
use crate::gui::game_state_manager::GameStateManager;
use crate::gui::{BoardGame, BoardMove, GUIGame};
use crate::gui::GUIMove;

pub trait InputHandler<G: GUIGame> {
	fn pending_moves(&self) -> Option<&Vec<G::M>>;
	fn set_pending_moves(&mut self, moves: Vec<G::M>);

	fn clear_pending_moves(&mut self);

	fn matching_moves(&self) -> &Vec<G::M>;

	fn intermediate_state(&self) -> &Option<G> {
		&None
	}
	fn set_intermediate_state(&mut self, _is: Option<G>) {}
	/*fn process<Drawer: GameDrawer<G>>(
		&mut self,
		click_pos: Drawer::Click,
		game_manager: &GameStateManager<G>,
		drawer: &mut Box<Drawer>,
	) -> MoveResult<G>
	where
		G: BoardGame,
		G::M: BoardMove<G>;

	fn reset<Drawer: GameDrawer<G>>(
		&mut self,
		drawer: &mut Box<dyn BoardDrawer<G, Click = (u8, u8)>>,
		game_manager: &GameStateManager<G>,
	) where
		G: BoardGame,
		G::M: BoardMove<G>;*/
}

pub enum MoveResult<G: Game, Click> {
	Invalid,
	Incomplete {
		selected: Option<Click>,
		highlights: Vec<Click>,
		matching_moves: Vec<G::M>,
		//intermediate_state: Option<G::S>
	},
	Created {
		mv: G::M,
		highlights_played: Vec<Click>,
	},
	ChoiceRequired {
		candidates: Vec<G::M>,
	},
}

pub struct BoardInputHandler<G: BoardGame>
where
	G::M: BoardMove<G>,
{
	clicks: Vec<G::Click>,
	pending_moves: Option<Vec<G::M>>,
	matching_moves: Vec<G::M>,
	intermediate_state: Option<G>,
}
impl<G: BoardGame> Default for BoardInputHandler<G>
where
	G::M: BoardMove<G>,
{
	fn default() -> Self {
		Self::new()
	}
}
impl<G: BoardGame> InputHandler<G> for BoardInputHandler<G>
where
	G::M: BoardMove<G>,
{
	fn pending_moves(&self) -> Option<&Vec<G::M>> {
		self.pending_moves.as_ref()
	}
	fn set_pending_moves(&mut self, moves: Vec<G::M>) {
		self.pending_moves = Some(moves);
	}

	fn clear_pending_moves(&mut self) {
		self.pending_moves = None
	}

	fn matching_moves(&self) -> &Vec<G::M> {
		&self.matching_moves
	}

	fn intermediate_state(&self) -> &Option<G> {
		&self.intermediate_state
	}
	fn set_intermediate_state(&mut self, is: Option<G>) {
		self.intermediate_state = is;
	}

}
impl<G: BoardGame> BoardInputHandler<G>
where
	G::M: BoardMove<G>,
{
	pub fn new() -> Self {
		Self {
			clicks: Vec::new(),
			pending_moves: None,
			matching_moves: vec![],
			intermediate_state: None,
		}
	}

	pub fn current_clicks(&self) -> &Vec<u16> {
		&self.clicks
	}

	pub fn process(
		&mut self,
		click_pos: u16,
		game_manager: &GameStateManager<G>,
		drawer: &mut Box<dyn BoardDrawer<G, Click = u16>>,
	) -> MoveResult<G, u16>
	where
		G: BoardGame,
		G::M: BoardMove<G>,
	{
		let index = click_pos;
		println!("index: {}", index);

		self.clicks.push(index);

		let custom_result = G::M::handle_clicks_interaction(
			game_manager.game(),
			game_manager.legal_moves(),
			&self.clicks,
		);

		match &custom_result {
			MoveResult::Created {
				highlights_played, ..
			} => {
				self.reset(drawer, game_manager);
				drawer.set_played_highlights(highlights_played.clone());
				custom_result
			}
			MoveResult::Incomplete {
				selected,
				highlights,
				matching_moves, /*, intermediate_state*/
			} => {
				drawer.set_selected(selected.clone());
				drawer.set_legal_highlights(highlights.clone());
				self.matching_moves = matching_moves.clone();
				if !self.matching_moves.is_empty() {
					self.intermediate_state = self.matching_moves[0]
						.compute_intermediate_state(game_manager.game(), &self.clicks)
						.clone();
				}
				custom_result
			}
			MoveResult::Invalid => {
				self.reset(drawer, game_manager);
				MoveResult::Invalid
			}
			MoveResult::ChoiceRequired { candidates } => {
				self.pending_moves = Some(candidates.clone());
				custom_result
			}
		}
	}

	pub fn reset(
		&mut self,
		drawer: &mut Box<dyn BoardDrawer<G, Click = u16>>,
		game_manager: &GameStateManager<G>,
	) where
		G: BoardGame,
		G::M: BoardMove<G>,
	{
		self.clicks.clear();
		self.pending_moves = None;
		self.intermediate_state = None;
		self.matching_moves = game_manager
			.legal_moves()
			.iter()
			.filter(|m| m.click_sequence(game_manager.game()) == self.clicks)
			.copied()
			.collect();
		drawer.clear_selection();
	}
}
