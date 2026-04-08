use eframe::egui;
use egui::{Color32, Rect};
use crate::common::gui::board_drawer::SquareDrawer;
use crate::common::new_move_searcher_vec;
use crate::common::gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType, Shape};

use crate::common::gui::board_app::GenericBoardApp;
use super::Board;
use super::game::HareAndHoundsEval;
//use super::game::HareAndHoundsEvalDumb;

use super::{Cell, Move, HareAndHounds};

impl BoardMove<HareAndHounds> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.from as u16)
	}

	fn to(&self) -> u16 {
		self.to as u16
	}
	//fn ty() -> MoveType { MoveType::FromToOrDrop }
}
impl EGUIPieceType for Cell {
	fn shape(&self) -> Shape {
		match self {
			Cell::Empty => unreachable!(),
			Cell::Hound => Shape::Circle{color:Color32::ORANGE, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
			Cell::Hare => Shape::Circle{color:Color32::LIGHT_GRAY, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
		}
	}
}

impl BoardGame for HareAndHounds {
	type PieceType=Cell;

	fn width(&self) -> u8 {
		5
	}

	fn height(&self) -> u8 {
		3
	}

	fn legal_moves(&self) -> Vec<Self::M> {
		let mut len=0;
		let mut moves=[Move::default();HareAndHounds::MAX_MOVES];
		self.legal_moves(&mut moves, &mut len);
		let mut mvs=vec![];
		mvs.extend_from_slice(&moves[0..len]);
		mvs
	}
	fn play(&mut self, mv: Self::M) {
		self.play_unchecked(mv);
	}

	fn result(&self) -> crate::common::GameResult {
		self.result()
	}

	fn current_player(&self) -> crate::common::Player {
		
		match self.turn() {
			true => crate::common::Player::PLAYER2,
			false => crate::common::Player::PLAYER1,
		}
	}
	fn get_name(&self, p: crate::common::Player) -> String {
		match p {
			crate::common::Player::PLAYER1 => "Hounds".into(),
			crate::common::Player::PLAYER2 => "Har".into(),
			_ => unreachable!()
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		match self.cell(x, y) {
			Cell::Empty => None,
			Cell::Hare => Some(Cell::Hare),
			Cell::Hound => Some(Cell::Hound),
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Board::index_from_coords(x, y) as u16
	}
	fn coords_from_index(i: u16) -> (u8, u8) {
		Board::coords_from_index(i as usize)
	}
	fn default_style() -> BoardStyle {
		let mut style = BoardStyle::default();
		style.show_coordinates_mod=crate::common::gui::CoordMod::NumbersAside;
		//style.checkerboard_mod=crate::common::gui::CheckerBoardMod::None;
		style
	}
}

struct HareAndHoundsSquareDrawer {
}
impl<G> SquareDrawer<G> for HareAndHoundsSquareDrawer
	where G: BoardGame,
		G::M: BoardMove<G> {
	fn draw(&self, painter: &egui::Painter, style: &BoardStyle, _game: &G, square: &Rect, x_coord:u8,y_coord:u8) {
		let index = if style.mirrored {
			Board::index_from_coords(x_coord, 2-y_coord)
		} else {
			Board::index_from_coords(x_coord, y_coord)
		};
		painter.rect_filled(*square, 0.0, style.uniform_color);
		let stroke = egui::Stroke::new(1.0, style.dark_color);
		match index {
			0|2|12|14 => {
				
			}
			1 => {
				let points=vec![square.center(), square.right_top()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.right_bottom()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.right_center()];
				painter.line(points, stroke);
			}
			5 => {
				let points=vec![square.left_bottom(), square.center()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.right_bottom()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.right_center()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.center_bottom()];
				painter.line(points, stroke);
			}
			3 => {
				let points=vec![square.center(), square.right_top()];
				painter.line(points, stroke);
				let points=vec![square.left_top(), square.center()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.right_center()];
				painter.line(points, stroke);
				let points=vec![square.center_top(), square.center()];
				painter.line(points, stroke);
			}
			7 => {
				let points=vec![square.left_bottom(), square.right_top()];
				painter.line(points, stroke);
				let points=vec![square.left_top(), square.right_bottom()];
				painter.line(points, stroke);
				let points=vec![square.left_center(), square.right_center()];
				painter.line(points, stroke);
				let points=vec![square.center_top(), square.center_bottom()];
				painter.line(points, stroke);
			}
			11 => {
				let points=vec![square.left_bottom(), square.center()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.right_bottom()];
				painter.line(points, stroke);
				let points=vec![square.left_center(), square.center()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.center_bottom()];
				painter.line(points, stroke);
			}
			9 => {
				let points=vec![square.center(), square.right_top()];
				painter.line(points, stroke);
				let points=vec![square.left_top(), square.center()];
				painter.line(points, stroke);
				let points=vec![square.left_center(), square.center()];
				painter.line(points, stroke);
				let points=vec![square.center_top(), square.center()];
				painter.line(points, stroke);
			}
			4|10 => {
				let points=vec![square.left_center(), square.right_center()];
				painter.line(points, stroke);
				let points=vec![square.center_top(), square.center_bottom()];
				painter.line(points, stroke);
			}
			8 => {
				let points=vec![square.left_center(), square.right_center()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.center_bottom()];
				painter.line(points, stroke);
			}
			6 => {
				let points=vec![square.left_center(), square.right_center()];
				painter.line(points, stroke);
				let points=vec![square.center_top(), square.center()];
				painter.line(points, stroke);
			}
			13 => {
				let points=vec![square.center(), square.left_top()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.left_bottom()];
				painter.line(points, stroke);
				let points=vec![square.center(), square.left_center()];
				painter.line(points, stroke);
			}
			_ => {}
		}
	}
}
pub fn create_board() -> GenericBoardApp<HareAndHounds> {
	let mut board=GenericBoardApp::new(HareAndHounds::default(), new_move_searcher_vec("Simple".into(), HareAndHoundsEval{}, 12));
	board.board_drawer.set_square_drawer(Box::new(HareAndHoundsSquareDrawer{}));
	board.depth = 12;
	board.max_depth = 25;
	board
}