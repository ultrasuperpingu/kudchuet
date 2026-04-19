use eframe::egui;
use egui::Color32;

use kudchuet::Player;
use kudchuet::gui::shapes::Shape;
use kudchuet::gui::{CheckerBoardMod, CoordMod};
use kudchuet::{
	gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType},
	new_move_searcher_vec,
};

use super::game::ReversiEval;
use crate::bitboard::Bitboard8x8;
use crate::rules::{Cell, Reversi};
use kudchuet::gui::board_app::GenericBoardApp;

impl BoardMove<Reversi> for (u8, u8) {
	fn from(&self) -> Option<u16> {
		None
	}

	fn to(&self) -> u16 {
		Bitboard8x8::index_from_coords(self.0, self.1) as u16
	}
	//fn ty() -> MoveType { MoveType::FromToOrDrop }
}
impl EGUIPieceType for Cell {
	fn shape(&self) -> Shape {
		match self {
			Cell::Empty => unreachable!(),
			Cell::White => Shape::Circle {
				fill_color: Some(Color32::WHITE),
				size: 0.7,
				text: None,
				stroke: None,
			},
			Cell::Black => Shape::Circle {
				fill_color: Some(Color32::BLACK),
				size: 0.7,
				text: None,
				stroke: None,
			},
		}
	}
}

impl BoardGame for Reversi {
	type PieceType = Cell;
	type Settings = kudchuet::gui::DefaultSettings;

	fn width(&self) -> u8 {
		8
	}

	fn height(&self) -> u8 {
		8
	}

	fn get_name(&self, p: Player) -> String {
		match p {
			Player::PLAYER1 => "Black".into(),
			Player::PLAYER2 => "White".into(),
			_ => unreachable!(),
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		match self.cell_from_coords(x, y) {
			Cell::Empty => None,
			Cell::Black => Some(Cell::Black),
			Cell::White => Some(Cell::White),
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard8x8::index_from_coords(x, y) as u16
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		Bitboard8x8::coords_from_index(index as usize)
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: CheckerBoardMod::None,
			uniform_color: Color32::from_rgb(0, 120, 0),
			show_coordinates_mod: CoordMod::FileRankOnSquare,
			square_stroke_color: Some(egui::Color32::BLACK),
			..Default::default()
		}
	}
}

pub fn create_board() -> GenericBoardApp<Reversi> {
	let board = GenericBoardApp::new(
		Reversi::default(),
		new_move_searcher_vec("Dumb".into(), ReversiEval::new(), 4),
	);
	board
}
