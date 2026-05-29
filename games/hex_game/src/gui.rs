use bitboard::Bitboard;
use eframe::egui::{self, Painter, Pos2, Rect, Vec2};
use egui::{Color32, Stroke, StrokeKind};

use kudchuet::ai::move_search::{UniformRolloutPolicy, MCTS};
use kudchuet::ai::{AIBuilder, AIEngineProvider, MoveSearcherBuilder};
use kudchuet::gui::board_app::GenericBoardApp;
use kudchuet::gui::board_drawer::{BoardDrawer, DefaultBoardDrawer, GameDrawer, SquareDrawer};
use kudchuet::gui::shapes::{Shape, StrokeData};
use kudchuet::gui::{
	BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType, GUIGame,
};
use kudchuet::Player;

use crate::bitboard::HexBoard;
use crate::game::HexMaterialEval;

use super::rules::{Hex, Move};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
	Player1,
	Player2,
}

impl EGUIPieceType for Piece {
	fn shape(&self) -> Shape {
		match self {
			Piece::Player1 => Shape::RegularPolygon {
				fill_color: Some(Color32::BLUE),
				text: None,
				size: 1.0,
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
				sides: 6,
				angle: 1.0 / 6.0,
			},
			Piece::Player2 => Shape::RegularPolygon {
				fill_color: Some(Color32::RED),
				text: None,
				size: 1.0,
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
				sides: 6,
				angle: 1.0 / 6.0,
			},
		}
	}
}
impl BoardMove<Hex> for Move {
	fn to(&self) -> u16 {
		match self {
			Move::Place(i) => *i as u16,
			Move::Swap => u16::MAX,
		}
	}
}

impl GUIGame for Hex {
	type Settings = kudchuet::gui::DefaultSettings;
	type Style = kudchuet::gui::BoardStyle;
	fn get_name(&self, p: Player) -> String {
		match p {
			Player::PLAYER1 => "Black".into(),
			Player::PLAYER2 => "White".into(),
			_ => unreachable!(),
		}
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: CheckerBoardMod::None,
			uniform_color: Color32::from_rgb(230, 200, 205),
			dark_color: Color32::from_rgb(60, 45, 30),
			light_color: Color32::from_rgb(200, 175, 140),
			show_coordinates_mod: CoordMod::None,
			square_stroke_color: Some(Color32::BLACK),
			empty_cell_shape: Some(Shape::RegularPolygon {
				fill_color: Some(Color32::from_rgb(200, 200, 210)),
				size: 1.0,
				sides: 6,
				text: None,
				stroke: Some(StrokeData {
					stroke: Stroke {
						width: 1.0,
						color: Color32::BLACK,
					},
					kind: StrokeKind::Inside,
				}),
				angle: 1.0 / 6.0,
			}),
			..Default::default()
		}
	}
}
impl BoardGame for Hex {
	type PieceType = Piece;

	fn width(&self) -> u8 {
		HexBoard::WIDTH
	}

	fn height(&self) -> u8 {
		HexBoard::HEIGHT
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		if self.blue.get(x, y) {
			Some(Piece::Player1)
		} else if self.red.get(x, y) {
			Some(Piece::Player2)
		} else {
			None
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		HexBoard::index_from_coords(x, y) as u16
	}

	fn coords_from_index(index: u16) -> (u8, u8) {
		HexBoard::coords_from_index(index as usize)
	}
}

#[derive(Default)]
pub struct HexSquareDrawer {}
impl<G> SquareDrawer<G> for HexSquareDrawer
where
	G: BoardGame,
	G::M: BoardMove<G>,
{
	fn draw(
		&self,
		painter: &Painter,
		style: &BoardStyle,
		_game: &G,
		square: &Rect,
		x_coord: u8,
		y_coord: u8,
	) {
		let (bg_color, _txt_color) = match style.checkerboard_mod {
			CheckerBoardMod::None => (style.uniform_color, Color32::BLACK),
			CheckerBoardMod::EvenDark => {
				if (x_coord + y_coord) % 2 == 1 {
					(style.light_color, Color32::BLACK)
				} else {
					(style.dark_color, Color32::WHITE)
				}
			}
			CheckerBoardMod::OddDark => {
				if (x_coord + y_coord).is_multiple_of(2) {
					(style.light_color, Color32::BLACK)
				} else {
					(style.dark_color, Color32::WHITE)
				}
			}
		};
		let mut points = Vec::with_capacity(6);
		let center = square.center();
		let radius = square.width() / 2.0;

		for i in 0..6 {
			let theta = i as f32 / 6.0 * std::f32::consts::TAU + std::f32::consts::FRAC_PI_6;
			let p = center + egui::Vec2::new(theta.cos(), theta.sin()) * radius;
			points.push(p);
		}
		painter.add(egui::Shape::convex_polygon(
			points.clone(),
			bg_color,
			egui::Stroke::NONE,
		));

		if let Some(color) = style.square_stroke_color {
			painter.line(points, egui::Stroke::new(1.0, color));
		}
	}
	fn draw_overlay(
		&self,
		painter: &egui::Painter,
		_style: &BoardStyle,
		_game: &G,
		board_rect: &Rect,
		_cell_size: f32,
	) {
		let mut points = Vec::with_capacity(4);
		points.push(board_rect.left_top());
		points.push(
			board_rect.left_top()
				+ Vec2 {
					x: 0.0,
					y: (_game.height() + 1) as f32 * _cell_size * 0.75,
				},
		);
		points.push(
			board_rect.right_top()
				+ Vec2 {
					x: 0.0,
					y: (_game.height() + 1) as f32 * _cell_size * 0.75,
				},
		);
		points.push(board_rect.right_top());
		painter.add(egui::Shape::convex_polygon(
			points.clone(),
			Color32::RED,
			egui::Stroke::NONE,
		));

		points.clear();
		points.push(board_rect.left_top());
		points.push(
			board_rect.left_top()
				+ Vec2 {
					x: 0.0,
					y: _game.height() as f32 * _cell_size * 0.75,
				},
		);
		points.push(
			board_rect.left_top()
				+ Vec2 {
					x: _cell_size,
					y: _game.height() as f32 * _cell_size * 0.75 - _cell_size,
				},
		);
		points.push(
			board_rect.left_top()
				+ Vec2 {
					x: _game.width() as f32 * _cell_size * SQRT_3 / 4.0,
					y: 0.0,
				},
		);
		painter.add(egui::Shape::convex_polygon(
			points.clone(),
			Color32::BLUE,
			egui::Stroke::NONE,
		));

		points.clear();
		points.push(board_rect.right_top());
		points.push(
			board_rect.right_top()
				+ Vec2 {
					x: -_cell_size * 0.75,
					y: 0.0,
				},
		);
		points.push(
			board_rect.right_top()
				+ Vec2 {
					x: -(_game.width() as f32 * _cell_size * SQRT_3 / 4.0) - _cell_size * 0.75,
					y: (_game.height() + 1) as f32 * _cell_size * 0.75 - _cell_size,
				},
		);
		points.push(
			board_rect.right_top()
				+ Vec2 {
					x: 0.0,
					y: (_game.height() + 1) as f32 * _cell_size * 0.75,
				},
		);
		painter.add(egui::Shape::convex_polygon(
			points.clone(),
			Color32::BLUE,
			egui::Stroke::NONE,
		));
	}
}
const SQRT_3: f32 = 1.7320508;

#[derive(Default)]
struct HexBoardDrawer(DefaultBoardDrawer<Hex>);
impl GameDrawer<Hex> for HexBoardDrawer {
	type Click = (u8, u8);

	fn get_style(&self) -> &BoardStyle {
		self.0.get_style()
	}

	fn get_style_mut(&mut self) -> &mut BoardStyle {
		self.0.get_style_mut()
	}

	fn set_style(&mut self, style: BoardStyle) {
		self.0.set_style(style);
	}
	fn full_reset(&mut self) {
		self.0.full_reset();
	}

	fn draw(
		&mut self,
		ui: &mut egui::Ui,
		game: &Hex,
		//input: &Box<dyn InputHandler<G>>,
		can_interact: bool,
	) -> Option<Self::Click> {
		self.draw_board(ui, game, can_interact)
	}
}
impl BoardDrawer<Hex> for HexBoardDrawer {
	fn get_square_drawer(&self) -> &dyn kudchuet::gui::board_drawer::SquareDrawer<Hex> {
		self.0.get_square_drawer()
	}

	fn set_square_drawer(
		&mut self,
		sq_drawer: Box<dyn kudchuet::gui::board_drawer::SquareDrawer<Hex>>,
	) {
		self.0.set_square_drawer(sq_drawer);
	}

	fn get_piece_drawer(&self) -> &dyn kudchuet::gui::board_drawer::PieceDrawer<Hex> {
		self.0.get_piece_drawer()
	}

	fn get_piece_drawer_mut(&mut self) -> &mut dyn kudchuet::gui::board_drawer::PieceDrawer<Hex> {
		self.0.get_piece_drawer_mut()
	}

	fn set_piece_drawer(
		&mut self,
		sq_drawer: Box<dyn kudchuet::gui::board_drawer::PieceDrawer<Hex>>,
	) {
		self.0.set_piece_drawer(sq_drawer);
	}

	fn get_selected(&self) -> Option<u16> {
		self.0.get_selected()
	}

	fn set_selected(&mut self, selected: Option<u16>) {
		self.0.set_selected(selected);
	}

	fn clear_selection(&mut self) {
		self.0.clear_selection();
	}

	fn get_legal_highlights(&self) -> &Vec<u16> {
		self.0.get_legal_highlights()
	}

	fn set_legal_highlights(&mut self, legal_highlights: Vec<u16>) {
		self.0.set_legal_highlights(legal_highlights);
	}

	fn get_played_highlights(&self) -> &Vec<u16> {
		self.0.get_played_highlights()
	}

	fn set_played_highlights(&mut self, played_highlights: Vec<u16>) {
		self.0.set_played_highlights(played_highlights);
	}

	fn coords_to_pixel(
		&self,
		board_rect: &egui::Rect,
		cell_size: f32,
		x_coord: u8,
		y_coord: u8,
		h: u8,
	) -> egui::Pos2 {
		let (x_visual, y_visual) = if self.get_style().mirrored {
			(x_coord, h - 1 - y_coord)
		} else {
			(x_coord, y_coord)
		};
		let x = board_rect.left()
			+ cell_size * SQRT_3 / 2.0 * (x_visual as f32 + y_visual as f32 * 0.5);

		let y = board_rect.top() + cell_size * 0.75 * (h - 1 - y_visual) as f32;
		Pos2::new(x, y)
	}
	fn pixel_to_coords(
		&self,
		board_rect: &egui::Rect,
		cell_size: f32,
		pos: egui::Pos2,
		w: u8,
		h: u8,
	) -> Option<(u8, u8)> {
		let x_off = pos.x - board_rect.left();
		let y_off = pos.y - board_rect.top();

		let y_visual = (h - 1) - (y_off / cell_size / 0.75).floor() as u8;
		let x_visual = (x_off / cell_size / SQRT_3 * 2.0 - y_visual as f32 / 2.0).floor() as u8;

		let (x_coord, y_coord) = if self.get_style().mirrored {
			(x_visual, h - 1 - y_visual)
		} else {
			(x_visual, y_visual)
		};

		if x_coord < w && y_coord < h {
			Some((x_coord, y_coord))
		} else {
			None
		}
	}
	/// Returns (cell size, board width, board height)
	///
	/// w: board width (in cell)
	/// h: board height (in cell)
	/// style: the board style
	/// avail_w: available width
	/// avail_h: available height
	fn get_cell_and_board_size(
		&self,
		w: u8,
		h: u8,
		_style: &BoardStyle,
		avail_w: f32,
		avail_h: f32,
	) -> (f32, f32, f32) {
		let cell_size = (avail_w / w as f32).min(avail_h / h as f32);
		let board_width = cell_size * (w as f32 + h as f32 / 2.0) * SQRT_3 / 2.0;
		//let board_width  = cell_size * w as f32;
		let board_height = cell_size / 0.75 * h as f32;
		(cell_size, board_width, board_height)
	}
}
pub fn create_board() -> GenericBoardApp<Hex> {
	let ai_provider: Vec<Box<dyn AIEngineProvider<Hex>>> = vec![
		Box::new(MoveSearcherBuilder::new("Material", HexMaterialEval, 4)),
		Box::new(AIBuilder::<Hex, MCTS<Hex, UniformRolloutPolicy<Hex>>>::new(
			"MCTS",
		)),
	];
	let mut board = GenericBoardApp::new(Hex::new(), ai_provider);
	board.game_drawer = Box::new(HexBoardDrawer::default());
	board
		.game_drawer
		.set_square_drawer(Box::new(HexSquareDrawer::default()));
	board
}
