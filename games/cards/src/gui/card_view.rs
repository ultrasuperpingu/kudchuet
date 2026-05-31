use eframe::egui::{self, Color32, Pos2, Rect, include_image, pos2};
use egui::{Image, vec2};
use kudchuet::{Player, ai::move_search::Game, gui::board_drawer::GameDrawer};

use crate::{
	gui::{CardGame, CardMove},
	playing_cards32::PlayingCard32,
	unordered_card_sets32::UnorderedCardSet32,
};
pub trait CardGameDrawer<G: CardGame>: GameDrawer<G>
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
}
pub struct CardVisual {
	pub card: PlayingCard32,
	pub pos: Pos2,
	pub rotation: f32,
	pub scale: f32,
}
pub struct PlayerLayout {
	pub rect: Rect,
	pub rotation: f32,
}
#[derive(Clone, Default)]
pub struct DefaultCardGameDrawer<G: CardGame>
where
	G::M: CardMove<G> + Copy,
{
	style: G::Style,
}

impl<G: CardGame> GameDrawer<G> for DefaultCardGameDrawer<G>
where
	<G as Game>::M: CardMove<G>,
{
	type Click = PlayingCard32;

	fn draw(
		&mut self,
		ui: &mut egui::Ui,
		game: &G,
		//input: &Box<dyn InputHandler<G>>,
		can_interact: bool,
	) -> Option<Self::Click> {
		//todo!()
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
impl<G: CardGame> CardGameDrawer<G> for DefaultCardGameDrawer<G> where <G as Game>::M: CardMove<G> {}
impl<G: CardGame> DefaultCardGameDrawer<G>
where
	<G as Game>::M: CardMove<G>,
{
	pub fn draw_board(
		&mut self,
		ui: &mut egui::Ui,
		game: &G,
		can_interact: bool,
	) -> Option<PlayingCard32> {
		let available = ui.available_rect_before_wrap();

		let table_rect = available.shrink(20.0);

		ui.painter()
			.rect_filled(table_rect, 16.0, Color32::from_rgb(20, 90, 20));

		self.draw_center(ui, game, table_rect);
		let layout = TableLayout::new(table_rect, G::nb_players(game) as usize);

		//self.draw_hand(ui, game, can_interact, table_rect)
		for i in 0..G::nb_players(game) {
			if i == game.current_player().0 {
				continue;
			}
			self.draw_player(
				ui,
				game.player_hand_cards(Player(i)),
				layout.player_areas[i as usize].rect,
				layout.player_areas[i as usize].rotation,
				false,
				can_interact,
			);
		}
		let p = &layout.player_areas[game.current_player().0 as usize];

		self.draw_player(
			ui,
			game.player_hand_cards(game.current_player()),
			p.rect,
			p.rotation,
			true,
			can_interact,
		)
	}

	fn draw_player(
		&mut self,
		ui: &mut egui::Ui,
		cards: UnorderedCardSet32,
		area: Rect,
		rotation: f32,
		face_up: bool,
		interactive: bool,
	) -> Option<PlayingCard32> {
		let mut clicked = None;

		let card_size = vec2(80.0, 120.0);
		let spacing = 45.0;
		//let total_width = (cards.len().saturating_sub(1) as f32) * spacing + card_size.x;
		let spread = vec2(rotation.cos(), rotation.sin()) * spacing;
		let start = area.center() - spread * ((cards.len().saturating_sub(1) as f32) * 0.5);

		for (i, card) in cards.iter().enumerate() {
			let center = start + spread * i as f32;

			let pos = center - card_size * 0.5;

			let response = if face_up {
				self.draw_card(ui, card, pos, card_size, rotation)
			} else {
				self.draw_back_cards(ui, pos, card_size, rotation)
			};

			if interactive && response.clicked() {
				clicked = Some(card);
			}
		}

		clicked
	}
	pub fn draw_back_cards(
		&mut self,
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

	pub fn draw_card(
		&self,
		ui: &mut egui::Ui,
		card: PlayingCard32,
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

	fn draw_center(&self, ui: &mut egui::Ui, game: &G, table: Rect) {
		let center = table.center();
		let card_size = egui::vec2(50.0, 75.0);
		let spacing = 70.0;
		let nb_players = G::nb_players(game) as f32;

		if game.draw_revealed_cards() {
			for (i, card) in game.revealed_cards().into_iter().enumerate() {
				let x = center.x + (i as f32 - (nb_players - 1.0) / 2.0) * spacing;
				let pos = pos2(x - card_size.x * 0.5, center.y - card_size.y * 0.5);
				self.draw_card(ui, card, pos, card_size, 0.0);
			}
		}

		for i in 0..G::nb_players(game) {
			if let Some(card) = game.player_ply_card(Player(i)) {
				let x = center.x + (i as f32 - (nb_players - 1.0) / 2.0) * spacing;
				let pos = pos2(x - card_size.x * 0.5, center.y - card_size.y * 0.5);
				self.draw_card(ui, card, pos, card_size, 0.0);
			}
		}
	}
}
pub struct TableLayout {
	pub player_areas: Vec<PlayerLayout>,
}
impl TableLayout {
	pub fn new(table: Rect, nb_players: usize) -> Self {
		let mut player_areas = Vec::new();

		let center = table.center();

		let radius_x = table.width() * 0.40;
		let radius_y = table.height() * 0.40;

		let hand_size = vec2(400.0, 160.0);

		for i in 0..nb_players {
			let t = i as f32 / nb_players as f32;
			let angle = std::f32::consts::TAU * t + std::f32::consts::FRAC_PI_2;

			let x = center.x + angle.cos() * radius_x;
			let y = center.y + angle.sin() * radius_y;

			// rotation vers le centre
			let dir = (center - pos2(x, y)).angle() + std::f32::consts::FRAC_PI_2;

			let rect = Rect::from_center_size(pos2(x, y), hand_size);

			player_areas.push(PlayerLayout {
				rect,
				rotation: dir,
			});
		}

		Self { player_areas }
	}
}
