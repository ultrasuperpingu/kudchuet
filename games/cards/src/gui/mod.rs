use egui::ImageSource;
use kudchuet::gui::card_view::{CardBoard, CardGameClick};
use kudchuet::gui::{GUIGame, GUIMove};


use kudchuet::cards::playing_cards::{CardSet, PlayingCard};

pub trait CardGame: GUIGame<S = Self, Click = CardGameClick<Self::Card>> + Default + Clone
where
	Self::M: GUIMove<Self> + Copy,
{
	type Card: PlayingCard;
	fn build_board(&self) -> CardBoard<impl CardSet<Card = Self::Card>, Self::Card>;
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