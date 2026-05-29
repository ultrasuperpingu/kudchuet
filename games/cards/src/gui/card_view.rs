use eframe::egui::{self, Color32, Pos2, Rect, include_image, pos2};
use egui::vec2;
use kudchuet::{Player, ai::move_search::Game, gui::board_drawer::GameDrawer};

use crate::{
	gui::{CardGame, CardMove},
	playing_cards32::PlayingCard32,
	unordered_card_sets32::UnorderedCardSet32,
};
fn paint_rotated_image(
	painter: &egui::Painter,
	texture_id: egui::TextureId,
	rect: Rect,
	angle: f32,
) {
	let center = rect.center();

	let mut mesh = egui::Mesh::with_texture(texture_id);

	let corners = [
		rect.left_top(),
		rect.right_top(),
		rect.right_bottom(),
		rect.left_bottom(),
	];

	let uvs = [
		pos2(0.0, 0.0),
		pos2(1.0, 0.0),
		pos2(1.0, 1.0),
		pos2(0.0, 1.0),
	];

	let rotated: Vec<Pos2> = corners
		.iter()
		.map(|p| {
			let dx = p.x - center.x;
			let dy = p.y - center.y;

			pos2(
				center.x + dx * angle.cos() - dy * angle.sin(),
				center.y + dx * angle.sin() + dy * angle.cos(),
			)
		})
		.collect();

	let idx = mesh.vertices.len() as u32;

	for i in 0..4 {
		mesh.vertices.push(egui::epaint::Vertex {
			pos: rotated[i],
			uv: uvs[i],
			color: Color32::WHITE,
		});
	}

	mesh.indices
		.extend_from_slice(&[idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);

	painter.add(egui::Shape::mesh(mesh));
}
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
#[derive(Debug, Clone, Copy, Default)]
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
				layout.player_areas[i as usize],
				false,
				can_interact,
			);
		}
		self.draw_player(
			ui,
			game.player_hand_cards(game.current_player()),
			layout.player_areas[game.current_player().0 as usize],
			true,
			can_interact,
		)
	}

	fn draw_player(
		&mut self,
		ui: &mut egui::Ui,
		cards: UnorderedCardSet32,
		area: Rect,
		face_up: bool,
		interactive: bool,
	) -> Option<PlayingCard32> {
		let mut clicked = None;

		let card_size = vec2(80.0, 120.0);
		let spacing = 45.0;
		let total_width = (cards.len().saturating_sub(1) as f32) * spacing + card_size.x;
		let start_x = area.center().x - total_width * 0.5;
		let y = area.center().y - card_size.y * 0.5;

		for (i, card) in cards.iter().enumerate() {
			let x = start_x + i as f32 * spacing;

			let pos = pos2(x, y);

			let response = if face_up {
				self.draw_card(ui, card, pos, card_size)
			} else {
				self.draw_back_cards(ui, pos, card_size)
			};

			if interactive && response.clicked() {
				clicked = Some(card);
			}
		}

		clicked
	}
	pub fn draw_back_cards(
		&self,
		ui: &mut egui::Ui,
		pos: Pos2,
		size: egui::Vec2,
	) -> egui::Response {
		let rect = Rect::from_min_size(pos, size);

		let response = ui.allocate_rect(rect, egui::Sense::click());

		let image = egui::Image::new(include_image!("../../cards/back.svg"));

		image.paint_at(ui, rect);

		response
	}
	pub fn draw_card(
		&self,
		ui: &mut egui::Ui,
		card: PlayingCard32,
		pos: Pos2,
		size: egui::Vec2,
	) -> egui::Response {
		let rect = Rect::from_min_size(pos, size);
		let response = ui.allocate_rect(rect, egui::Sense::click());
		let image = card.card_texture();
		image.paint_at(ui, rect);

		response
	}

	fn draw_center(&self, ui: &mut egui::Ui, game: &G, table: Rect) {
		let center = table.center();
		let card_size = egui::vec2(50.0, 75.0);
		let spacing = 70.0;
		let nb_players = G::nb_players(game) as f32;

		for i in 0..G::nb_players(game) {
			if let Some(card) = game.player_ply_card(Player(i)) {
				let x = center.x + (i as f32 - (nb_players - 1.0) / 2.0) * spacing;
				let pos = pos2(x - card_size.x * 0.5, center.y - card_size.y * 0.5);
				self.draw_card(ui, card, pos, card_size);
			}
		}
	}
}
pub struct TableLayout {
	pub player_areas: Vec<Rect>,
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
			let rect = Rect::from_center_size(pos2(x, y), hand_size);

			player_areas.push(rect);
		}

		Self { player_areas }
	}
}
