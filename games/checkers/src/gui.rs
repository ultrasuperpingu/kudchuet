use eframe::egui;
use egui::{Color32, Stroke, StrokeKind};
use kudchuet::gui::board_app::GenericBoardApp;
use kudchuet::gui::shapes::Shape;
use kudchuet::{
	Player,
	gui::{
		BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType,
		shapes::{StrokeData, TextData},
	},
	new_move_searcher_vec,
};

use crate::game::CheckersEval;
use crate::rules::{Cell, Checkers10, Move};

impl BoardMove<Checkers10> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.from() as u16)
	}

	fn to(&self) -> u16 {
		self.to() as u16
	}
	fn to_uci(&self) -> Option<String> {
		Some(self.to_string())
	}
}
impl EGUIPieceType for Cell {
	fn shape(&self) -> Shape {
		match self {
			Cell::Empty => unreachable!(),
			Cell::WhitePawn => Shape::Circle {
				fill_color: Some(Color32::WHITE),
				size: 0.7,
				text: None,
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
			},
			Cell::BlackPawn => Shape::Circle {
				fill_color: Some(Color32::BLACK),
				size: 0.7,
				text: None,
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
			},
			Cell::WhiteQueen => Shape::Circle {
				fill_color: Some(Color32::WHITE),
				size: 0.7,
				text: Some(TextData {
					text: "Q".into(),
					color: Color32::BLACK,
					size: 0.5,
				}),
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
			},
			Cell::BlackQueen => Shape::Circle {
				fill_color: Some(Color32::BLACK),
				size: 0.7,
				text: Some(TextData {
					text: "Q".into(),
					color: Color32::WHITE,
					size: 0.5,
				}),
				stroke: None,
			},
		}
	}
}

impl BoardGame for Checkers10 {
	type PieceType = Cell;
	type Settings = kudchuet::gui::DefaultSettings;

	fn width(&self) -> u8 {
		10
	}

	fn height(&self) -> u8 {
		10
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		match self.cell_from_coords(x, y) {
			Cell::Empty => None,
			Cell::WhitePawn => Some(Cell::WhitePawn),
			Cell::BlackPawn => Some(Cell::BlackPawn),
			Cell::WhiteQueen => Some(Cell::WhiteQueen),
			Cell::BlackQueen => Some(Cell::BlackQueen),
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		if let Some(index) = Checkers10::coords_to_index(x, y) {
			index as u16
		} else {
			100
		}
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		Checkers10::index_to_coords(index as u8)
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
	fn get_position_from_string(&self, pos_str: &str) -> Result<Self, String> {
		Checkers10::from_fen(pos_str)
	}
	fn move_to_string(&self, m: &Move) -> Option<String> {
		if m.is_simple() {
			return Some(format!("{}-{}", m.from() + 1, m.to() + 1));
		}

		let candidates: Vec<_> = self
			.legal_moves()
			.into_iter()
			.filter(|mv| {
				mv.from() == m.from()
					&& mv.to() == m.to()
					&& mv.pawn_takes == m.pawn_takes
					&& mv.queen_takes == m.queen_takes
			})
			.collect();

		if candidates.is_empty() {
			return None;
		}

		if candidates.len() > 1 {
			return Some(format!("{}x{}", m.from() + 1, m.to() + 1));
		}

		Some(format!("{}x{}", m.from() + 1, m.to() + 1))
	}

	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: CheckerBoardMod::OddDark,
			uniform_color: Color32::from_rgb(235, 230, 220),
			show_coordinates_mod: CoordMod::NumbersOnSquare,
			square_stroke_color: Some(egui::Color32::BLACK),
			mirrored: true,
			..Default::default()
		}
	}

	fn nb_players(&self) -> u8 {
		2
	}

	fn get_name(&self, p: Player) -> String {
		p.to_string()
	}

	fn move_from_string(&self, m_str: &str) -> Result<Self::M, String> {
		Self::M::from_uci(m_str)
	}

	fn play_random(&mut self) {}
}

pub fn create_board() -> GenericBoardApp<Checkers10> {
	let board = GenericBoardApp::new(
		Checkers10::default(),
		new_move_searcher_vec("Material".into(), CheckersEval::new(), 5),
	);
	board
}
