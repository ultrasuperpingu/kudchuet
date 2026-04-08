
use eframe::egui;
use egui::{Color32, Stroke};

use crate::common::{bitboards::Goban, gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType, Shape, board_drawer::SquareDrawer}, new_move_searcher_vec};
use crate::gomoku::{Cell, Gomoku, Move, game::GomokuEvalSimple};

use crate::common::gui::board_app::GenericBoardApp;
//use super::{game::ThreeMusketeersEvalAdvance};

impl BoardMove<Gomoku> for Move {
	fn from(&self) -> Option<u16> {
		None
	}

	fn to(&self) -> u16 {
		self.to
	}
}
impl EGUIPieceType for Cell {
	fn shape(&self) -> Shape {
		match self {
			Cell::White => Shape::Circle{color:Color32::WHITE, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None},
			Cell::Black => Shape::Circle{color:Color32::BLACK, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: None},
			Cell::Empty => unreachable!(),
		}
	}
}

impl BoardGame for Gomoku {
	type PieceType=Cell;

	fn width(&self) -> u8 {
		19
	}

	fn height(&self) -> u8 {
		19
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

	fn current_player(&self) -> crate::common::Player {
		self.turn
	}
	fn get_name(&self, p: crate::common::Player) -> String {
		match p {
			crate::common::Player::PLAYER1 => "Black".into(),
			crate::common::Player::PLAYER2 => "White".into(),
			_ => unreachable!(),
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		match self.cell(x, y) {
			Cell::White => Some(Cell::White),
			Cell::Black => Some(Cell::Black),
			Cell::Empty => None,
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Goban::index_from_coords(x, y) as u16
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		Goban::coords_from_index(index as usize)
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: crate::common::gui::CheckerBoardMod::None,
			uniform_color: Color32::from_rgb(215, 180, 140),
			light_color: Color32::from_rgb(200, 200, 250),
			dark_color: Color32::from_rgb(40, 40, 40),
			show_coordinates_mod: crate::common::gui::CoordMod::NumbersAside,
			..Default::default()
		}
	}
}
pub struct GobanSquareDrawer;
impl SquareDrawer<Gomoku> for GobanSquareDrawer {
	fn draw(&self, painter: &egui::Painter, style: &crate::common::gui::BoardStyle, _game: &Gomoku, square: &egui::Rect, x_coord:u8,y_coord:u8) {
		painter.rect_filled(*square, 0.0, style.uniform_color);
		if x_coord == 0 {
			if y_coord == 0 {
				let lines=vec![square.center_top(), square.center()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
				let lines=vec![square.center(), square.right_center()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
			} else if y_coord == 18 {
				let lines=vec![square.center(), square.center_bottom()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
				let lines=vec![square.center(), square.right_center()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
			} else {
				let lines=vec![square.center_top(), square.center_bottom()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
				let lines=vec![square.center(), square.right_center()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
			}
		} else if x_coord == 18 {
			if y_coord == 0 {
				let lines=vec![square.center_top(), square.center()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
				let lines=vec![square.center(), square.left_center()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
			} else if y_coord == 18 {
				let lines=vec![square.center(), square.center_bottom()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
				let lines=vec![square.center(), square.left_center()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
			} else {
				let lines=vec![square.center_top(), square.center_bottom()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
				let lines=vec![square.center(), square.left_center()];
				painter.line(lines, Stroke::new(2.0, style.dark_color));
			}
		} else if y_coord == 0 {
			let lines=vec![square.center_top(), square.center()];
			painter.line(lines, Stroke::new(2.0, style.dark_color));
			let lines=vec![square.right_center(), square.left_center()];
			painter.line(lines, Stroke::new(2.0, style.dark_color));
		} else if y_coord == 18 {
			let lines=vec![square.center_bottom(), square.center()];
			painter.line(lines, Stroke::new(2.0, style.dark_color));
			let lines=vec![square.right_center(), square.left_center()];
			painter.line(lines, Stroke::new(2.0, style.dark_color));
		} else {
			let lines=vec![square.center_top(), square.center_bottom()];
			painter.line(lines, Stroke::new(2.0, style.dark_color));
			let lines=vec![square.right_center(), square.left_center()];
			painter.line(lines, Stroke::new(2.0, style.dark_color));
			if x_coord == 9 && y_coord == 9 ||
				x_coord == 3 && y_coord == 9 || x_coord == 9 && y_coord == 3 ||
				x_coord == 15 && y_coord == 9|| x_coord == 9 && y_coord == 15 ||
				x_coord == 3 && y_coord == 3|| x_coord == 3 && y_coord == 15 ||
				x_coord == 15 && y_coord == 3|| x_coord == 15 && y_coord == 15 {
				painter.circle_filled(square.center(), square.width()*0.15, style.dark_color);
			}
		}
	}
}
pub fn create_board() -> GenericBoardApp<Gomoku> {
	let mut board=GenericBoardApp::new(Gomoku::default(), new_move_searcher_vec("Simple".into(), GomokuEvalSimple{}, 4));
	board.board_drawer.set_square_drawer(Box::new(GobanSquareDrawer{}));
	board
}