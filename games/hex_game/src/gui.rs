use bitboard::Bitboard;
use eframe::egui::{self, Pos2};
use egui::{Color32, Stroke, StrokeKind};

use kudchuet::ai::{AIEngineProvider, MoveSearcherBuilder};
use kudchuet::gui::board_app::GenericBoardApp;
use kudchuet::gui::board_drawer::{BoardDrawer, DefaultBoardDrawer, HexSquareDrawer};
use kudchuet::gui::shapes::{Shape, StrokeData};
use kudchuet::gui::{BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType};

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
			Piece::Player1 => Shape::Circle {
				fill_color: Some(Color32::BLUE),
				text: None,
				size: 0.8,
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
			},
			Piece::Player2 => Shape::Circle {
				fill_color: Some(Color32::RED),
				text: None,
				size: 0.8,
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
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

impl BoardGame for Hex {
	type PieceType = Piece;
	type Settings = kudchuet::gui::DefaultSettings;

	fn width(&self) -> u8 {
		7
	}

	fn height(&self) -> u8 {
		7
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

	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: CheckerBoardMod::OddDark,
			uniform_color: Color32::from_rgb(40, 70, 125),
			dark_color: Color32::from_rgb(60, 45, 30),
			light_color: Color32::from_rgb(200, 175, 140),
			show_coordinates_mod: CoordMod::None,
			square_stroke_color: None,
			..Default::default()
		}
	}
}

const SQRT_3: f32 = 1.7320508;

#[derive(Default)]
struct HexBoardDrawer(DefaultBoardDrawer<Hex>);
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

	fn get_style(&self) -> &BoardStyle {
		self.0.get_style()
	}

	fn get_style_mut(&mut self) -> &mut BoardStyle {
		self.0.get_style_mut()
	}

	fn set_style(&mut self, style: BoardStyle) {
		self.0.set_style(style);
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

	fn full_reset(&mut self) {
		self.0.full_reset();
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
		//if !board_rect.contains(pos) {
		//	return None;
		//}

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
		let board_height = cell_size as f32 / 0.75 * h as f32;
		(cell_size, board_width, board_height)
	}
}
pub fn create_board() -> GenericBoardApp<Hex> {
	let ai_provider: Vec<Box<dyn AIEngineProvider<Hex>>> = vec![Box::new(
		MoveSearcherBuilder::new("Material".into(), HexMaterialEval::default(), 4),
	)];
	let mut board = GenericBoardApp::new(Hex::new(), ai_provider);
	board.board_drawer = Box::new(HexBoardDrawer::default());
	board
		.board_drawer
		.set_square_drawer(Box::new(HexSquareDrawer::default()));
	board
}
