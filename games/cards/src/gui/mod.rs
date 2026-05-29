use kudchuet::{
	Player, ai::move_search::Game, gui::{GUIGame, input_handler::InputHandler}
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
}

pub trait CardMove<G: Game>: Debug + Sized + Copy {
}

trait CardInputHandler<G: CardGame>: InputHandler<G>
where
	<G as Game>::M: CardMove<G>,
{
}
#[derive(Default, Clone, Debug)]
pub struct DefaultCardInputHandler<G: CardGame>
where
	<G as Game>::M: CardMove<G>,
{
	pending_moves: Vec<G::M>,
}
impl<G: CardGame> InputHandler<G> for DefaultCardInputHandler<G>
where
	<G as Game>::M: CardMove<G>,
{
	fn pending_moves(&self) -> Option<&Vec<<G>::M>> {
		Some(&self.pending_moves)
	}

	fn set_pending_moves(&mut self, moves: Vec<<G>::M>) {
		self.pending_moves = moves;
	}

	fn clear_pending_moves(&mut self) {
		self.pending_moves.clear();
	}

	fn matching_moves(&self) -> &Vec<<G>::M> {
		todo!()
	}
}
impl<G: CardGame> CardInputHandler<G> for DefaultCardInputHandler<G> where <G as Game>::M: CardMove<G> {}
