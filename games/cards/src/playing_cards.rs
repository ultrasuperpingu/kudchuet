use std::fmt::{Debug, Display};

use egui::ImageSource;

pub trait PlayingCard:
	Copy + Clone + Eq + std::hash::Hash + TryFrom<u8, Error = ()> + Debug + Display
where
	Self: 'static,
{
	const CARD_COUNT: u8;
	const ALL: &'static [Self];
	type Color: Copy + Debug + PartialEq;

	fn index(self) -> u8;
	fn from_index(i: u8) -> Option<Self>;
	fn color(self) -> Self::Color;
}
pub trait DrawablePlayingCard: PlayingCard
where
	Self: 'static,
{
	fn card_texture(&self) -> ImageSource<'_>;
	fn aspect_ratio() -> f32 {
		2.0 / 3.0
	}
}
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum CardSuit {
	Spades,
	Hearts,
	Diamonds,
	Clubs,
	Joker,
}

pub trait CardSet: Sized + IntoIterator<Item = Self::Card> + Debug {
	type Card: PlayingCard;

	const EMPTY: Self;

	fn contains(&self, card: Self::Card) -> bool;
	fn insert(&mut self, card: Self::Card) -> bool;
	fn remove(&mut self, card: Self::Card) -> bool;

	fn len(&self) -> usize;

	fn is_empty(&self) -> bool;

	fn draw_random(nb: u8, from: &mut Self) -> Result<Self, String>;
	fn all() -> Self;
	fn iter(&self) -> impl Iterator<Item = Self::Card>;
}
impl<Card: PlayingCard> CardSet for Option<Card> {
	type Card = Card;

	const EMPTY: Self = None;

	fn contains(&self, card: Self::Card) -> bool {
		self == &Some(card)
	}

	fn insert(&mut self, card: Self::Card) -> bool {
		*self = Some(card);
		true
	}

	fn remove(&mut self, card: Self::Card) -> bool {
		if self == &Some(card) {
			*self = None;
			true
		} else {
			false
		}
	}

	fn len(&self) -> usize {
		match self {
			Some(_) => 1,
			None => 0,
		}
	}

	fn is_empty(&self) -> bool {
		self.is_none()
	}

	fn draw_random(nb: u8, from: &mut Self) -> Result<Self, String> {
		if nb > 1 || nb == 1 && from.is_none() {
			Err("Not enough cards in the set".into())
		} else if nb == 0 {
			Ok(None)
		} else {
			Ok(from.take())
		}
	}

	fn all() -> Self {
		unimplemented!()
	}

	fn iter(&self) -> impl Iterator<Item = Self::Card> {
		self.iter().cloned()
	}
}
