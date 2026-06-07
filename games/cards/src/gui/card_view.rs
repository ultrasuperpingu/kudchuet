use core::f32;

use eframe::egui::{self, Color32, Pos2, Rect, include_image};
use egui::{Image, Vec2, pos2, vec2};
use kudchuet::{ai::move_search::Game, gui::board_drawer::GameDrawer};

use crate::{
	gui::{CardGame, CardMove},
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
		let rect = Rect::from_min_size(pos, size);
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
		let click = board.draw(ui, self, table_rect);
		if can_interact { click } else { None }
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
	Circle { start_angle: f32, len: usize },
	PlayersAround(u8),
}
#[derive(Debug, PartialEq)]
pub struct CardZone<Set: CardSet<Card = C>, C: PlayingCard> {
	pub id: u8,
	pub set: Set,
	pub layout: CardSetLayout,
	pub rotation: f32,
	pub origin: Pos2,
	pub rect: Rect,
	pub card_spacing: f32,
	pub face_up: bool,
	pub draw_empty: bool,
	pub zone_only: bool,
}
impl<Set: CardSet<Card = C>, C: DrawablePlayingCard> CardZone<Set, C> {
	pub fn draw<G: CardGame<Card = C>, Drawer: CardGameDrawer<G, C>>(
		&self,
		ui: &mut egui::Ui,
		drawer: &Drawer,
		board_rect: Rect,
		card_size: Vec2,
	) -> Option<CardGameClick<C>>
	where
		<G as Game>::M: CardMove<G>,
		Set::Card: PlayingCard,
		Set::Item: PlayingCard,
	{
		//println!("{:?}", self);
		let origin = pos2(
			board_rect.min.x + self.origin.x * board_rect.width(),
			board_rect.min.y + self.origin.y * board_rect.height(),
		);
		let zone_rect = Rect::from_min_max(
			pos2(
				board_rect.min.x + self.rect.min.x * board_rect.width(),
				board_rect.min.y + self.rect.min.y * board_rect.height(),
			),
			pos2(
				board_rect.min.x + self.rect.max.x * board_rect.width(),
				board_rect.min.y + self.rect.max.y * board_rect.height(),
			),
		);

		match self.layout {
			CardSetLayout::Stack => {
				if let Some(card) = self.set.iter().last() {
					let resp = if self.face_up {
						drawer.draw_card(ui, card, origin, card_size, self.rotation)
					} else {
						drawer.draw_back_cards(ui, origin, card_size, self.rotation)
					};

					if resp.clicked() {
						if self.zone_only {
							return Some(CardGameClick::CardZone(self.id));
						} else {
							return Some(CardGameClick::Card(card));
						}
					}
				} else if self.draw_empty {
					let rect = Rect::from_min_size(origin, card_size);

					let resp = ui.allocate_rect(rect, egui::Sense::click());

					ui.painter().rect_stroke(
						rect,
						4.0,
						egui::Stroke::new(2.0, Color32::GRAY),
						egui::StrokeKind::Middle,
					);

					if resp.clicked() {
						return Some(CardGameClick::CardZone(self.id));
					}
				}
				None
			}

			CardSetLayout::Horizontal => {
				let mut done = false;
				for (i, card) in self.set.iter().enumerate() {
					done = true;
					let pos = origin + vec2(i as f32 * self.card_spacing, 0.0);

					let resp = if self.face_up {
						drawer.draw_card(ui, card, pos, card_size, self.rotation)
					} else {
						drawer.draw_back_cards(ui, pos, card_size, self.rotation)
					};

					if resp.clicked() {
						if self.zone_only {
							return Some(CardGameClick::CardZone(self.id));
						} else {
							return Some(CardGameClick::Card(card));
						}
					}
				}
				if self.draw_empty && !done {
					let rect = Rect::from_min_size(origin, card_size);

					let resp = ui.allocate_rect(rect, egui::Sense::click());

					ui.painter().rect_stroke(
						rect,
						4.0,
						egui::Stroke::new(2.0, Color32::GRAY),
						egui::StrokeKind::Middle,
					);

					if resp.clicked() {
						return Some(CardGameClick::CardZone(self.id));
					}
				}
				None
			}

			CardSetLayout::Vertical => {
				let mut done = false;
				for (i, card) in self.set.iter().enumerate() {
					done = true;
					let pos = origin + vec2(0.0, i as f32 * self.card_spacing);

					let resp = if self.face_up {
						drawer.draw_card(ui, card, pos, card_size, self.rotation)
					} else {
						drawer.draw_back_cards(ui, pos, card_size, self.rotation)
					};

					if resp.clicked() {
						if self.zone_only {
							return Some(CardGameClick::CardZone(self.id));
						} else {
							return Some(CardGameClick::Card(card));
						}
					}
				}
				if self.draw_empty && !done {
					let rect = Rect::from_min_size(origin, card_size);

					let resp = ui.allocate_rect(rect, egui::Sense::click());

					ui.painter().rect_stroke(
						rect,
						4.0,
						egui::Stroke::new(2.0, Color32::GRAY),
						egui::StrokeKind::Middle,
					);

					if resp.clicked() {
						return Some(CardGameClick::CardZone(self.id));
					}
				}
				None
			}

			CardSetLayout::Circle { start_angle, len } => {
				let radius = 0.15 * board_rect.height();

				for (i, card) in self.set.iter().enumerate() {
					let angle = start_angle + i as f32 / len as f32 * f32::consts::TAU;

					let pos = origin + vec2(angle.cos() * radius, angle.sin() * radius);

					let resp = if self.face_up {
						drawer.draw_card(ui, card, pos, card_size, self.rotation)
					} else {
						drawer.draw_back_cards(ui, pos, card_size, self.rotation)
					};

					if resp.clicked() {
						if self.zone_only {
							return Some(CardGameClick::CardZone(self.id));
						} else {
							return Some(CardGameClick::Card(card));
						}
					}
				}

				None
			}

			CardSetLayout::PlayersAround(nb_players) => {
				let radius = 0.15 * board_rect.height();
				let n = nb_players as f32;

				for (i, card) in self.set.iter().enumerate() {
					let angle = i as f32 / n * f32::consts::TAU;

					let pos = origin + vec2(angle.cos() * radius, angle.sin() * radius);

					let resp = if self.face_up {
						drawer.draw_card(ui, card, pos, card_size, self.rotation)
					} else {
						drawer.draw_back_cards(ui, pos, card_size, self.rotation)
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
	pub fn draw<G, D>(&self, ui: &mut egui::Ui, drawer: &D, rect: Rect) -> Option<CardGameClick<C>>
	where
		G: CardGame<Card = C>,
		D: CardGameDrawer<G, C>,
		<G as Game>::M: CardMove<G>,
	{
		let card_size = self.best_card_size(rect);
		let mut click = None;
		for zone in &self.zones {
			let resp = zone.draw(ui, drawer, rect, card_size);
			if resp.is_some() {
				click = resp;
			}
		}
		click
	}
	pub fn best_card_size(&self, rect: Rect) -> Vec2 {
		//return Vec2::new(80.0, 120.0);
		let mut size = Vec2::new(f32::MAX, f32::MAX);

		for zone in &self.zones {
			let zone_rect = zone.compute_rect(rect);
			let zone_size = zone.max_card_size(zone_rect);
			//println!("zone_size {}", zone_size);

			size.x = size.x.min(zone_size.x);
			size.y = size.y.min(zone_size.y);
		}
		//println!("size {}", size);
		let ratio = C::aspect_ratio();

		let width_from_height = size.y * ratio;
		let height_from_width = size.x / ratio;

		if width_from_height <= size.x {
			size.x = width_from_height;
		} else {
			size.y = height_from_width;
		}
		//println!("final {}", size);
		size
	}
}
impl<Set: CardSet<Card = C>, C: PlayingCard> CardZone<Set, C> {
	pub fn compute_rect(&self, board_rect: Rect) -> Rect {
		Rect::from_min_max(
			pos2(
				board_rect.min.x + self.rect.min.x * board_rect.width(),
				board_rect.min.y + self.rect.min.y * board_rect.height(),
			),
			pos2(
				board_rect.min.x + self.rect.max.x * board_rect.width(),
				board_rect.min.y + self.rect.max.y * board_rect.height(),
			),
		)
	}
}
impl<Set, C> CardZone<Set, C>
where
	Set: CardSet<Card = C>,
	C: DrawablePlayingCard,
{
	fn max_card_size(&self, zone_rect: Rect) -> Vec2 {
		let ratio = C::aspect_ratio();

		let w = zone_rect.width();
		let h = zone_rect.height();

		match self.layout {
			CardSetLayout::Stack => vec2(w, h),

			CardSetLayout::Vertical => {
				let card_h = w / ratio;
				vec2(w, card_h)
			}

			CardSetLayout::Horizontal => {
				let card_w = h * ratio;
				vec2(card_w, h)
			}

			_ => vec2(80.0, 120.0),
		}
	}
}
