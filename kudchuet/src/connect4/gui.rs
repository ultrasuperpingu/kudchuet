use eframe::egui;
use egui::Color32;
use crate::common::new_move_searcher_vec;
use crate::common::{GameResult, Player, bitboards::Bitboard7x7Col};
 use crate::common::gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType, Shape};
 use crate::connect4::game::ConnectFourEval;

use crate::common::gui::board_app::GenericBoardApp;

use super::{Cell, Column, ConnectFour};

impl BoardMove<ConnectFour> for Column {

	fn to(&self) -> u16 {
		Bitboard7x7Col::index_from_coords(self.0, 0) as u16
	}
}
impl EGUIPieceType for Cell {
	fn shape(&self) -> Shape {
		match self {
			Cell::Empty => unreachable!(),
			Cell::PlayerOne => Shape::Circle{color:Color32::YELLOW, text: "".into(), size: 0.7, text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
			Cell::PlayerTwo => Shape::Circle{color:Color32::RED, text: "".into(), size: 0.7, text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
		}
	}
}

impl BoardGame for ConnectFour {
	type PieceType=Cell;

	fn width(&self) -> u8 {
		7
	}

	fn height(&self) -> u8 {
		6
	}

	fn legal_moves(&self) -> Vec<Self::M> {
		self.legal_moves()
	}
	fn play(&mut self, mv: Self::M) {
		self.play_unchecked(mv);
	}

	fn result(&self) -> GameResult {
		if !self.is_over() {
			GameResult::OnGoing
		} else if self.is_victory() {
			match self.player_turn() {
				Cell::Empty => unreachable!(),
				Cell::PlayerOne => GameResult::Player1,
				Cell::PlayerTwo => GameResult::Player2,
			}
		} else {
			//????
			match self.player_turn() {
				Cell::Empty => unreachable!(),
				Cell::PlayerOne => GameResult::Player2,
				Cell::PlayerTwo => GameResult::Player1,
			}
		}
	}

	fn current_player(&self) -> Player {
		
		match self.player_turn() {
			Cell::Empty => panic!(),
			Cell::PlayerOne => Player::PLAYER1,
			Cell::PlayerTwo => Player::PLAYER2,
		}
	}
	fn get_name(&self, p: crate::common::Player) -> String {
		match p {
			crate::common::Player::PLAYER1 => "Yellow".into(),
			crate::common::Player::PLAYER2 => "Red".into(),
			_ => unreachable!(),
		}
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
	fn default_style() -> BoardStyle {
		BoardStyle {
			dark_color: Color32::from_rgb(0, 0, 150),
			light_color: Color32::from_rgb(0, 0, 150),
			empty_cell_shape: Some(Shape::Circle { color: Color32::from_rgb(20, 20, 80), size: 0.7, text: "".into(), text_color: egui::Color32::BLACK, stroke_color: Some(egui::Color32::BLACK) }),
			show_coordinates_mod: crate::common::gui::CoordMod::NumbersAside,
			..Default::default()
		}
	}
}

pub fn create_board() -> GenericBoardApp<ConnectFour> {
	let board=GenericBoardApp::new(ConnectFour::new(), new_move_searcher_vec("Simple".into(), ConnectFourEval{}, 6));
	board
}