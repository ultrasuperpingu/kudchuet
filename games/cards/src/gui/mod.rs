use kudchuet::{
	ai::move_search::Game,
	gui::{
		GUIGame, GUIMove,
		input_handler::{InputHandler, MoveResult},
	},
};
use std::fmt::Debug;

use crate::{
	gui::card_view::{CardGameClick, CardZone},
	playing_cards::{CardSet, PlayingCard},
};
pub mod card_app;
pub mod card_view;
pub mod cards;

pub trait CardGame: GUIGame<S = Self, Click = CardGameClick<Self::Card>> + Default + Clone
where
	Self::M: CardMove<Self> + Copy,
{
	type Card: PlayingCard;
	fn build_board(&self) -> Vec<CardZone<impl CardSet<Card = Self::Card>, Self::Card>>;
}

pub trait CardMove<G: CardGame>: GUIMove<G> + Debug + Sized + Copy
where
	G::M: CardMove<G>,
{
	fn click(&self) -> Option<CardGameClick<G::Card>>;
}

#[derive(Clone)]
pub struct CardInputHandler<G: CardGame>
where
	G::M: CardMove<G>,
{
	clicks: Vec<G::Click>,
	pending_moves: Option<Vec<G::M>>,
	matching_moves: Vec<G::M>,
	//_dummy: PhantomData<CardGameClick<G::Card>>,
}
impl<G: CardGame> Default for CardInputHandler<G>
where
	<G as Game>::M: CardMove<G>,
{
	fn default() -> Self {
		Self {
			clicks: Default::default(),
			pending_moves: Default::default(),
			matching_moves: Default::default(),
			//_dummy: PhantomData,
		}
	}
}
impl<G: CardGame<Card = Card>, Card: PlayingCard> CardInputHandler<G>
where
	G::M: CardMove<G>,
{
	pub fn new() -> Self {
		Self {
			clicks: vec![],
			pending_moves: None,
			matching_moves: vec![],
			//_dummy: PhantomData,
		}
	}
	pub fn current_clicks(&self) -> &Vec<G::Click> {
		&self.clicks
	}
	pub fn process_click(&mut self, click: G::Click, game: &G) -> MoveResult<G, G::Click> {
		let legals = game.legal_moves();
		self.clicks.push(click);

		let custom_result = G::M::handle_clicks_interaction(game, legals.as_slice(), &self.clicks);
		println!("{:?}: {:?}: {:?}",custom_result, legals, self.clicks);
		match &custom_result {
			MoveResult::Created {
				highlights_played, ..
			} => {
				self.reset();
				//self.reset(drawer, game_manager);
				//drawer.set_played_highlights(highlights_played.clone());
				custom_result
			}
			MoveResult::Incomplete {
				selected,
				highlights,
				matching_moves, /*, intermediate_state*/
			} => {
				//drawer.set_selected(selected.clone());
				//drawer.set_legal_highlights(highlights.clone());
				self.matching_moves = matching_moves.clone();
				if !self.matching_moves.is_empty() {
					//	self.intermediate_state = self.matching_moves[0]
					//		.compute_intermediate_state(game_manager.game(), &self.clicks)
					//		.clone();
				}
				custom_result
			}
			MoveResult::Invalid => {
				//self.reset(drawer, game_manager);
				self.reset();
				MoveResult::Invalid
			}
			MoveResult::ChoiceRequired { candidates } => {
				self.pending_moves = Some(candidates.clone());
				custom_result
			}
		}

	}

	pub fn reset(&mut self) {
		self.clicks.clear();
		self.pending_moves = None;
		self.matching_moves.clear();
	}
	/*pub fn reset(
		&mut self,
		drawer: &mut Box<dyn BoardDrawer<G>>,
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
	}*/
}
impl<G: CardGame> InputHandler<G> for CardInputHandler<G>
where
	G::M: CardMove<G>,
{
	fn pending_moves(&self) -> Option<&Vec<G::M>> {
		self.pending_moves.as_ref()
	}

	fn set_pending_moves(&mut self, moves: Vec<G::M>) {
		self.pending_moves = Some(moves);
	}

	fn clear_pending_moves(&mut self) {
		self.pending_moves = None;
	}

	fn matching_moves(&self) -> &Vec<G::M> {
		&self.matching_moves
	}
}
