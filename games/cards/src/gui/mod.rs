use kudchuet::{
	Player,
	ai::move_search::Game,
	gui::{
		GUIGame,
		input_handler::{InputHandler, MoveResult},
	},
};
use std::fmt::Debug;

use crate::{playing_cards32::PlayingCard32, unordered_card_sets32::UnorderedCardSet32};
pub mod card_app;
pub mod card_view;
pub mod cards;

pub trait CardGame: GUIGame<S = Self> + Default + Clone
where
	Self::M: CardMove<Self> + Copy,
{
	fn player_ply_card(&self, p: Player) -> Option<PlayingCard32>;
	fn player_hand_cards(&self, p: Player) -> UnorderedCardSet32;
	fn revealed_cards(&self) -> UnorderedCardSet32;
	fn draw_revealed_cards(&self) -> bool;
}

pub trait CardMove<G: Game>: Debug + Sized + Copy {
	fn card(&self) -> Option<PlayingCard32>;
}

#[derive(Clone)]
pub struct CardInputHandler<G: CardGame>
where
	G::M: CardMove<G>,
{
	pending_moves: Option<Vec<G::M>>,
	matching_moves: Vec<G::M>,
}
impl<G: CardGame> Default for CardInputHandler<G>
where
	<G as Game>::M: CardMove<G>,
{
	fn default() -> Self {
		Self {
			pending_moves: Default::default(),
			matching_moves: Default::default(),
		}
	}
}
impl<G: CardGame> CardInputHandler<G>
where
	G::M: CardMove<G>,
{
	pub fn new() -> Self {
		Self {
			pending_moves: None,
			matching_moves: vec![],
		}
	}

	pub fn process_click(&mut self, card: PlayingCard32, game: &G) -> MoveResult<G> {
		let legals = game.legal_moves();

		let candidates: Vec<G::M> = legals
			.into_iter()
			.filter(|m| m.card() == Some(card))
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
impl<G: CardGame> InputHandler<G> for CardInputHandler<G> where G::M: CardMove<G> {
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
