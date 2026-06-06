use core::f32;

use eframe::egui::{self, Color32, Pos2, Rect, include_image};
use egui::{Image, pos2, vec2};
use kudchuet::{ai::move_search::Game, gui::board_drawer::GameDrawer};

use crate::{
	gui::{CardGame, CardMove},
	ordered_card_set::OrderedCardSet,
	playing_cards::{CardSet, DrawablePlayingCard, PlayingCard},
};
pub trait CardGameDrawer<G: CardGame<Card = C>, C: PlayingCard>: GameDrawer<G>
where
	<G as Game>::M: CardMove<G>,
{
	/*fn draw(
		&mut self,
		ui: &mut egui::Ui,
		game: &G,
		input: &CardInputHandler<G>,
		can_interact: bool,
	) -> Option<G::M>;

	fn full_reset(&mut self);*/
	fn draw_back_cards(
		&self,
		ui: &mut egui::Ui,
		pos: Pos2,
		size: egui::Vec2,
		rotation: f32,
	) -> egui::Response;
	fn draw_card(
		&self,
		ui: &mut egui::Ui,
		card: C,
		pos: Pos2,
		size: egui::Vec2,
		rotation: f32,
	) -> egui::Response;
}
#[derive(Clone)]
pub struct DefaultCardGameDrawer<
	G: CardGame<Card = C, Click = CardGameClick<C>>,
	C: DrawablePlayingCard,
> where
	G::M: CardMove<G> + Copy,
{
	style: G::Style,
}
impl<G: CardGame<Card = C, Click = CardGameClick<C>>, C: DrawablePlayingCard> Default
	for DefaultCardGameDrawer<G, C>
where
	G::M: CardMove<G> + Copy,
{
	fn default() -> Self {
		Self {
			style: Default::default(),
		}
	}
}
impl<G: CardGame<Card = C, Click = CardGameClick<C>>, C: DrawablePlayingCard> GameDrawer<G>
	for DefaultCardGameDrawer<G, C>
where
	<G as Game>::M: CardMove<G>,
{
	fn draw(
		&mut self,
		ui: &mut egui::Ui,
		game: &G,
		//input: &Box<dyn InputHandler<G>>,
		can_interact: bool,
	) -> Option<G::Click> {
		self.draw_board(ui, game, can_interact)
	}

	fn full_reset(&mut self) {}

	fn get_style(&self) -> &<G as kudchuet::gui::GUIGame>::Style {
		&self.style
	}

	fn get_style_mut(&mut self) -> &mut <G as kudchuet::gui::GUIGame>::Style {
		&mut self.style
	}

	fn set_style(&mut self, style: <G as kudchuet::gui::GUIGame>::Style) {
		self.style = style;
	}
}
impl<G: CardGame<Card = C, Click = CardGameClick<C>>, C: DrawablePlayingCard> CardGameDrawer<G, C>
	for DefaultCardGameDrawer<G, C>
where
	<G as Game>::M: CardMove<G>,
{
	fn draw_back_cards(
		&self,
		ui: &mut egui::Ui,
		pos: Pos2,
		size: egui::Vec2,
		rotation: f32,
	) -> egui::Response {
		let rect = Rect::from_center_size(pos + size * 0.5, size);
		let response = ui.allocate_rect(rect, egui::Sense::click());

		let image = egui::Image::new(include_image!("../../cards/back.svg"))
			.maintain_aspect_ratio(true)
			.rotate(rotation, vec2(0.5, 0.5));

		image.paint_at(ui, rect);

		response
	}

	fn draw_card(
		&self,
		ui: &mut egui::Ui,
		card: C,
		pos: Pos2,
		size: egui::Vec2,
		rotation: f32,
	) -> egui::Response {
		let rect = Rect::from_min_size(pos, size);
		let response = ui.allocate_rect(rect, egui::Sense::click());
		let image = Image::new(card.card_texture())
			.maintain_aspect_ratio(true)
			.rotate(rotation, vec2(0.5, 0.5));
		image.paint_at(ui, rect);

		response
	}
}
impl<G: CardGame<Card = C, Click = CardGameClick<C>>, C: DrawablePlayingCard>
	DefaultCardGameDrawer<G, C>
where
	<G as Game>::M: CardMove<G>,
{
	pub fn draw_board(
		&mut self,
		ui: &mut egui::Ui,
		game: &G,
		can_interact: bool,
	) -> Option<G::Click> {
		let available = ui.available_rect_before_wrap();

		let table_rect = available.shrink(20.0);

		ui.painter()
			.rect_filled(table_rect, 16.0, Color32::from_rgb(20, 90, 20));

		let board = game.build_board();
		let mut click = None;
		for l in board {
			//println!("{:?}", l);
			let resp = l.draw(ui, self, table_rect);
			if can_interact && resp.is_some() {
				click = resp;
			}
		}
		click
	}
}
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum CardGameClick<C: PlayingCard> {
	Card(C),
	CardZone(u8),
}
#[derive(Debug, PartialEq)]
pub enum CardSetLayout {
	Stack,
	Vertical,
	Horizontal,
	Circle { start_angle: f32 },
	PlayersAround(u8),
}
#[derive(Debug, PartialEq)]
pub struct CardZone<Set: CardSet<Card = C>, C: PlayingCard> {
	pub id: u8,
	pub set: Set,
	pub layout: CardSetLayout,
	pub rotation: f32,
	pub origin: Pos2,
	pub card_spacing: f32,
	pub face_up: bool,
	pub draw_empty: bool,
}
impl<Set: CardSet<Card = C>, C: DrawablePlayingCard> CardZone<Set, C> {
	pub fn draw<G: CardGame<Card = C>, Drawer: CardGameDrawer<G, C>>(
		&self,
		ui: &mut egui::Ui,
		drawer: &Drawer,
		rect: Rect,
	) -> Option<CardGameClick<C>>
	where
		<G as Game>::M: CardMove<G>,
		Set::Card: PlayingCard,
		Set::Item: PlayingCard,
	{
		//println!("{:?}", self);
		let size = vec2(80.0, 120.0);
		let origin = pos2(
			rect.min.x + self.origin.x * rect.width(),
			rect.min.y + self.origin.y * rect.height(),
		);

		match self.layout {
			CardSetLayout::Stack => {
				if let Some(card) = self.set.iter().last() {
					let resp = if self.face_up {
						drawer.draw_card(ui, card, origin, size, self.rotation)
					} else {
						drawer.draw_back_cards(ui, origin, size, self.rotation)
					};

					if resp.clicked() {
						return Some(CardGameClick::Card(card));
					}
				} else if self.draw_empty {
					let rect = Rect::from_center_size(origin, size);

					let resp = ui.allocate_rect(rect, egui::Sense::click());

					ui.painter()
						.rect_stroke(rect, 4.0, egui::Stroke::new(2.0, Color32::GRAY), egui::StrokeKind::Middle);

					if resp.clicked() {
						return Some(CardGameClick::CardZone(self.id));
					}
				}
				None
			}

			CardSetLayout::Horizontal => {
				for (i, card) in self.set.iter().enumerate() {
					let pos = origin + vec2(i as f32 * self.card_spacing, 0.0);

					let resp = if self.face_up {
						drawer.draw_card(ui, card, pos, size, self.rotation)
					} else {
						drawer.draw_back_cards(ui, pos, size, self.rotation)
					};

					if resp.clicked() {
						return Some(CardGameClick::Card(card));
					}
				}

				None
			}

			CardSetLayout::Vertical => {
				for (i, card) in self.set.iter().enumerate() {
					let pos = origin + vec2(0.0, i as f32 * self.card_spacing);

					let resp = if self.face_up {
						drawer.draw_card(ui, card, pos, size, self.rotation)
					} else {
						drawer.draw_back_cards(ui, pos, size, self.rotation)
					};

					if resp.clicked() {
						return Some(CardGameClick::Card(card));
					}
				}

				None
			}

			CardSetLayout::Circle { start_angle } => {
				let len = self.set.len();
				let radius = 0.15 * rect.height();

				for (i, card) in self.set.iter().enumerate() {
					let angle = start_angle + i as f32 / len as f32 * f32::consts::TAU;

					let pos = origin + vec2(angle.cos() * radius, angle.sin() * radius);

					let resp = if self.face_up {
						drawer.draw_card(ui, card, pos, size, self.rotation)
					} else {
						drawer.draw_back_cards(ui, pos, size, self.rotation)
					};

					if resp.clicked() {
						return Some(CardGameClick::Card(card));
					}
				}

				None
			}

			CardSetLayout::PlayersAround(nb_players) => {
				let radius = 0.15 * rect.height();
				let n = nb_players as f32;

				for (i, card) in self.set.iter().enumerate() {
					let angle = i as f32 / n * f32::consts::TAU;

					let pos = origin + vec2(angle.cos() * radius, angle.sin() * radius);

					let resp = if self.face_up {
						drawer.draw_card(ui, card, pos, size, self.rotation)
					} else {
						drawer.draw_back_cards(ui, pos, size, self.rotation)
					};

					if resp.clicked() {
						return Some(CardGameClick::Card(card));
					}
				}

				None
			}
		}
	}
}
pub struct CardBoard<Set, C>
where
	Set: CardSet<Card = C>,
	C: PlayingCard,
{
	pub zones: Vec<CardZone<Set, C>>,
}
impl<Set, C> CardBoard<Set, C>
where
	Set: CardSet<Card = C>,
	C: DrawablePlayingCard,
{
	pub fn draw<G, D>(&self, ui: &mut egui::Ui, drawer: &D, rect: Rect)
	where
		G: CardGame<Card = C>,
		D: CardGameDrawer<G, C>,
		<G as Game>::M: CardMove<G>,
	{
		for zone in &self.zones {
			zone.draw(ui, drawer, rect);
		}
	}
}
