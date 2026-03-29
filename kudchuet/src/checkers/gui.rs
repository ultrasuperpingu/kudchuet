
use eframe::egui;
use egui::Color32;
use crate::common::{Player, gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType, Shape}, new_move_searcher_vec};
use crate::checkers::rules::{Cell, Checkers10, Move};
use crate::checkers::game::CheckersEval;
use crate::common::gui::board_app::GenericBoardApp;

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
			Cell::WhitePawn => Shape::Circle{color:Color32::WHITE, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
			Cell::BlackPawn => Shape::Circle{color:Color32::BLACK, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: None},
			Cell::WhiteQueen =>Shape::Circle{color:Color32::WHITE, size: 0.7, text: "Q".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
			Cell::BlackQueen => Shape::Circle{color:Color32::BLACK, size: 0.7, text: "Q".into(), text_color: Color32::WHITE, stroke_color: None},
		}
	}
}

impl BoardGame for Checkers10 {
	type PieceType=Cell;

	fn width(&self) -> u8 {
		10
	}

	fn height(&self) -> u8 {
		10
	}

	fn legal_moves(&self) -> Vec<Self::M> {
		self.legal_moves()
	}
	fn play(&mut self, mv: Self::M) {
		self.play_unchecked(&mv);
	}

	fn result(&self) -> crate::common::GameResult {
		if self.is_over() {
			if self.player_turn() == Player::Player1 {
				crate::common::GameResult::Player1
			} else {
				crate::common::GameResult::Player2
			}
		} else {
			crate::common::GameResult::OnGoing
		}
	}

	fn current_player(&self) -> crate::common::Player {
		self.player_turn()
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
		if let Some(index) = Checkers10::coords_to_index(x as u8, y as u8) {
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
	fn get_position_from_string(&self, pos_str: &String) -> Result<Self, String> {
		Ok(Checkers10::from_fen(pos_str)?)
	}
	fn move_to_string(&self, m: &Move) -> Option<String> {
		if m.is_simple() {
			return Some(format!("{}-{}", m.from()+1, m.to()+1));
		}

		let candidates: Vec<_> = self.legal_moves()
			.into_iter()
			.filter(|mv| {
				mv.from() == m.from() &&
				mv.to() == m.to() &&
				mv.pawn_takes == m.pawn_takes &&
				mv.queen_takes == m.queen_takes
			})
			.collect();

		if candidates.is_empty() {
			return None;
		}

		if candidates.len() > 1 {
			return Some(format!("{}x{}", m.from()+1, m.to()+1));
		}

		Some(format!("{}x{}", m.from()+1, m.to()+1))
	}
	
	fn default_style() -> BoardStyle {
		let mut style = BoardStyle::default();
		style.checkerboard_mod=crate::common::gui::CheckerBoardMod::OddDark;
		style.uniform_color=Color32::from_rgb(235, 230, 220);
		style.show_coordinates_mod=crate::common::gui::CoordMod::NumbersOnSquare;
		style.square_stroke_color=Some(egui::Color32::BLACK);
		style.mirrored = true;
		style
	}
}

pub fn create_board() -> GenericBoardApp<Checkers10> {
	let board=GenericBoardApp::new(Checkers10::default(), new_move_searcher_vec("Material".into(), CheckersEval{}, 5));
	board
}