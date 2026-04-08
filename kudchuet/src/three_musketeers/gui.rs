
use eframe::egui;
use egui::Color32;

use crate::common::{bitboards::Bitboard5x5, gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType, Shape}, new_move_searcher_vec};
use crate::three_musketeers::{Move, Player, ThreeMusketeers, game::ThreeMusketeersEvalSimple};

use crate::common::gui::board_app::GenericBoardApp;
//use super::{game::ThreeMusketeersEvalAdvance};

impl BoardMove<ThreeMusketeers> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.from as u16)
	}

	fn to(&self) -> u16 {
		self.to as u16
	}
	//fn ty() -> MoveType { MoveType::FromToOrDrop }
}
#[derive(Copy, Clone)]
pub enum ThreeMPiece {
	Musketeer,
	Guard,
}
impl EGUIPieceType for ThreeMPiece {
	fn shape(&self) -> Shape {
		match self {
			//✠    ✥♱♰✢✣
			ThreeMPiece::Musketeer => Shape::Circle{color:Color32::from_rgb(50, 100, 200), size: 0.7, text: "⚜".into(), text_color: Color32::WHITE, stroke_color: None},
			//ThreeMPiece::Musketeer => Shape::Circle{color:Color32::from_rgb(50, 100, 200), text: "✠".into(), text_color: Color32::WHITE, stroke_color: None},
			//✚ 
			ThreeMPiece::Guard => Shape::Circle{color:Color32::from_rgb(200, 50, 50), size: 0.7, text: "✝".into(), text_color: Color32::WHITE, stroke_color: None},
		}
	}
}

impl BoardGame for ThreeMusketeers {
	type PieceType=ThreeMPiece;

	fn width(&self) -> u8 {
		5
	}

	fn height(&self) -> u8 {
		5
	}

	fn legal_moves(&self) -> Vec<Self::M> {
		let mut moves = vec![];
		self.legal_moves_inplace(&mut moves);
		moves
	}
	fn play(&mut self, mv: Self::M) {
		self.play_unchecked(mv);
	}

	fn result(&self) -> crate::common::GameResult {
		self.result()
	}

	fn current_player(&self) -> Player {
		if self.turn == 0 {Player::PLAYER1} else {Player::PLAYER2}
	}
	fn get_name(&self, p: crate::common::Player) -> String {
		match p {
			crate::common::Player::PLAYER1 => "Musketeers".into(),
			crate::common::Player::PLAYER2 => "Guards".into(),
			_ => unreachable!()
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		match self.get_cell(x, y) {
			Some(Player::PLAYER1) => Some(ThreeMPiece::Musketeer),
			Some(Player::PLAYER2) => Some(ThreeMPiece::Guard),
			_ => None,
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard5x5::index_from_coords(x, y) as u16
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		Bitboard5x5::coords_from_index(index as usize)
	}
	fn default_style() -> BoardStyle {
		let mut style = BoardStyle::default();
		style.checkerboard_mod=crate::common::gui::CheckerBoardMod::EvenDark;
		//style.uniform_color=Color32::from_rgb(235, 230, 220);
		style.light_color=Color32::from_rgb(200, 200, 250);
		style.dark_color=Color32::from_rgb(40, 40, 100);
		style.show_coordinates_mod=crate::common::gui::CoordMod::FileRankOnSquare;
		style.played_highlights_shape=Shape::Rect {
			color: Color32::from_rgba_unmultiplied(200,200,90,120),
			size: 1.0,
			text: "".into(),
			text_color: Color32::BLACK,
			stroke_color: None
		};
		//style.square_stroke_color=Some(egui::Color32::BLACK);
		style
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
	fn get_position_from_string(&self, pos_str: &String) -> Result<Self, String> {
		Self::from_fen(pos_str)
	}
	
}

pub fn create_board() -> GenericBoardApp<ThreeMusketeers> {
	let board=GenericBoardApp::new(ThreeMusketeers::default(), new_move_searcher_vec("Simple".into(), ThreeMusketeersEvalSimple{}, 5));
	board
}