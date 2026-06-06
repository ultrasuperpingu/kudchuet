use kudchuet::{
	Player,
	ai::move_search::Game,
	gui::{
		GUIGame,
		input_handler::{InputHandler, MoveResult},
	},
};
use std::{fmt::Debug, marker::PhantomData};

use crate::{
	gui::card_view::{CardGameClick, CardZone}, playing_cards::{CardSet, PlayingCard}
};
pub mod card_app;
pub mod card_view;
pub mod cards;

pub trait CardGame: GUIGame<S = Self> + Default + Clone
where
	Self::M: CardMove<Self> + Copy,
{
	type Card: PlayingCard;
	fn build_board(&self) -> Vec<CardZone<impl CardSet<Card = Self::Card>, Self::Card>>;
}

pub trait CardMove<G: CardGame>: Debug + Sized + Copy
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
	pending_moves: Option<Vec<G::M>>,
	matching_moves: Vec<G::M>,
	_dummy: PhantomData<CardGameClick<G::Card>>,
}
impl<G: CardGame> Default for CardInputHandler<G>
where
	<G as Game>::M: CardMove<G>,
{
	fn default() -> Self {
		Self {
			pending_moves: Default::default(),
			matching_moves: Default::default(),
			_dummy: PhantomData,
		}
	}
}
impl<G: CardGame<Card = Card>, Card: PlayingCard> CardInputHandler<G>
where
	G::M: CardMove<G>,
{
	pub fn new() -> Self {
		Self {
			pending_moves: None,
			matching_moves: vec![],
			_dummy: PhantomData,
		}
	}

	pub fn process_click(&mut self, click: CardGameClick<Card>, game: &G) -> MoveResult<G, CardGameClick<G::Card>> {
		let legals = game.legal_moves();

		let candidates: Vec<G::M> = legals
			.into_iter()
			.filter(|m| m.click() == Some(click))
			.collect();

		match candidates.len() {
			0 => MoveResult::Invalid,

			1 => MoveResult::Created {
				mv: candidates[0],
				highlights_played: vec![],
			},

			_ => {
				self.pending_moves = Some(candidates.clone());
				MoveResult::ChoiceRequired { candidates }
			}
		}
	}

	pub fn reset(&mut self) {
		self.pending_moves = None;
		self.matching_moves.clear();
	}
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
