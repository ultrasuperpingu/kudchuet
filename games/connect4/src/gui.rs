use crate::bitboard::Bitboard7x7Col;
use kudchuet::egui;
use egui::Color32;
use kudchuet::Player;
use kudchuet::ai::{AIEngineProvider, MoveSearcherBuilder};
use kudchuet::gui::shapes::{Shape, StrokeData};
use kudchuet::gui::{BoardGame, BoardMove, BoardStyle, CoordMod, EGUIPieceType, GUIGame, GUIMove};

use super::game::ConnectFourEval;

use kudchuet::gui::game_app::GenericBoardApp;

use super::rules::{Cell, Column, ConnectFour};

impl GUIMove<ConnectFour> for Column {
	fn click_sequence(&self, _state: &ConnectFour) -> Vec<<ConnectFour as GUIGame>::Click> {
		self.click_sequence_board_move_default(_state)
	}
}
impl BoardMove<ConnectFour> for Column {
	fn to(&self) -> u16 {
		Bitboard7x7Col::index_from_coords(self.0, 0) as u16
	}
}
impl EGUIPieceType for Cell {
	fn shape(&self) -> Shape {
		match self {
			Cell::Empty => unreachable!(),
			Cell::PlayerOne => Shape::Circle {
				fill_color: Some(Color32::YELLOW),
				size: 0.7,
				text: None,
				stroke: Some(StrokeData::default()),
			},
			Cell::PlayerTwo => Shape::Circle {
				fill_color: Some(Color32::RED),
				size: 0.7,
				text: None,
				stroke: Some(StrokeData::default()),
			},
		}
	}
}

impl GUIGame for ConnectFour {
	type Click = u16;
	type Settings = kudchuet::gui::DefaultSettings;
	type Style = kudchuet::gui::BoardStyle;
	fn get_name(&self, p: Player) -> String {
		match p {
			Player::PLAYER1 => "Yellow".into(),
			Player::PLAYER2 => "Red".into(),
			_ => unreachable!(),
		}
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			dark_color: Color32::from_rgb(0, 0, 150),
			light_color: Color32::from_rgb(0, 0, 150),
			empty_cell_shape: Some(Shape::Circle {
				fill_color: Some(Color32::from_rgb(20, 20, 80)),
				size: 0.7,
				text: None,
				stroke: Some(StrokeData::default()),
			}),
			show_coordinates_mod: CoordMod::NumbersAside,
			..Default::default()
		}
	}
}
impl BoardGame for ConnectFour {
	type PieceType = Cell;

	fn width(&self) -> u8 {
		7
	}

	fn height(&self) -> u8 {
		6
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		match self.cell(x, y) {
			Cell::Empty => None,
			Cell::PlayerOne => Some(Cell::PlayerOne),
			Cell::PlayerTwo => Some(Cell::PlayerTwo),
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard7x7Col::index_from_coords(x, y) as u16
	}
	fn coords_from_index(i: u16) -> (u8, u8) {
		Bitboard7x7Col::coords_from_index(i as usize)
	}
	
}

pub fn create_board() -> GenericBoardApp<ConnectFour> {
	let engines: Vec<Box<dyn AIEngineProvider<ConnectFour>>> = vec![Box::new(
		MoveSearcherBuilder::new("Simple", ConnectFourEval::new(), 9),
	)];
	GenericBoardApp::new(ConnectFour::new(), engines)
}
