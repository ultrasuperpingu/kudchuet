
use eframe::egui;
use egui::Color32;

use crate::common::{bitboards::Bitboard8x8, gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType, Shape}, new_move_searcher_vec};

use crate::common::gui::board_app::GenericBoardApp;
use super::{game::ReversiEval};
use super::{Cell, Reversi};

impl BoardMove<Reversi> for (u8,u8) {
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
			Cell::White => Shape::Circle{color:Color32::WHITE, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: None},
			Cell::Black => Shape::Circle{color:Color32::BLACK, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: None},
		}
	}
}

impl BoardGame for Reversi {

	type PieceType=Cell;

	fn width(&self) -> u8 {
		8
	}

	fn height(&self) -> u8 {
		8
	}

	fn legal_moves(&self) -> Vec<Self::M> {
		let mut moves = vec![];
		self.legal_moves(&mut moves);
		moves
	}
	fn play(&mut self, mv: Self::M) {
		self.play_unchecked(mv.0, mv.1);
	}

	fn result(&self) -> crate::common::GameResult {
		if !self.is_over() {
			crate::common::GameResult::OnGoing
		} else if self.is_draw() {
			crate::common::GameResult::Draw
		} else {
			if self.winner() == Some(Cell::Black) {
				crate::common::GameResult::Player1
			} else {
				crate::common::GameResult::Player2
			}
		}
	}

	fn current_player(&self) -> crate::common::Player {
		
		match self.turn() {
			Cell::Empty => unreachable!(),
			Cell::Black => crate::common::Player::PLAYER1,
			Cell::White => crate::common::Player::PLAYER2,
		}
	}
	fn get_name(&self, p: crate::common::Player) -> String {
		match p {
			crate::common::Player::PLAYER1 => "Black".into(),
			crate::common::Player::PLAYER2 => "White".into(),
			_ => unreachable!()
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
			checkerboard_mod: crate::common::gui::CheckerBoardMod::None,
			uniform_color: Color32::from_rgb(0, 120, 0),
			show_coordinates_mod: crate::common::gui::CoordMod::FileRankOnSquare,
			square_stroke_color: Some(egui::Color32::BLACK),
			..Default::default()
		}
	}
}

pub fn create_board() -> GenericBoardApp<Reversi> {
	let board=GenericBoardApp::new(Reversi::default(), new_move_searcher_vec("Dumb".into(), ReversiEval{}, 4));
	board
}