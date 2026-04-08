use eframe::egui;
use egui::{Color32, Rect};
use minimax::Game;
use crate::bagh_chal::game::BaghChalMaterialEval;
use crate::common::gui::board_app::GenericBoardApp;
use crate::common::gui::board_drawer::SquareDrawer;
use crate::common::gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType, Shape};
use crate::common::{Player, new_move_searcher_vec};

use super::*;


impl BoardMove<BaghChal> for Move {
	fn from(&self) -> Option<u16> { self.from.map(|f| f as u16) }
	fn to(&self) -> u16 { self.to as u16 }

}
#[derive(Copy, Clone)]
pub enum BaghChalPiece {
	Goat,
	Tiger,
}
impl EGUIPieceType for BaghChalPiece {
	fn shape(&self) -> Shape {
		match self {
			BaghChalPiece::Goat => Shape::Circle{color:Color32::WHITE, size: 0.7, text: "🐐".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
			BaghChalPiece::Tiger => Shape::Circle{color:Color32::ORANGE, size: 0.7, text: "🐅".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
		}
	}
}
impl BoardGame for BaghChal {
	type PieceType=BaghChalPiece;

	fn width(&self) -> u8 {
		5
	}

	fn height(&self) -> u8 {
		5
	}

	fn current_player(&self) -> crate::common::Player {
		
		match self.tigers_turn() {
			false => crate::common::Player::PLAYER1,
			true => crate::common::Player::PLAYER2,
		}
	}
	fn get_name(&self, p: crate::common::Player) -> String {
		match p {
			crate::common::Player::PLAYER1 => "Goats".into(),
			crate::common::Player::PLAYER2 => "Tigers".into(),
			_ => unreachable!(),
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		if self.tigers.get(x, y) {
			Some(BaghChalPiece::Tiger)
		} else if self.goats.get(x, y) {
			Some(BaghChalPiece::Goat)
		} else {
			None
		}
	}
	fn result(&self) -> GameResult {
		match <Self as Game>::get_winner(self) {
			Some(minimax::Winner::Draw) => GameResult::Draw,
			Some(minimax::Winner::PlayerJustMoved) => if self.current_player() == Player::PLAYER1 {GameResult::Player2} else {GameResult::Player1},
			Some(minimax::Winner::PlayerToMove) => if self.current_player() == Player::PLAYER2 {GameResult::Player2} else {GameResult::Player1},
			None => GameResult::OnGoing,
		}
	}
	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard5x5::index_from_coords(x, y) as u16
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		Bitboard5x5::coords_from_index(index as usize)
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			dark_color: egui::Color32::from_rgb(181, 136, 99),
			light_color: Color32::from_rgb(240, 217, 181),
			show_coordinates_mod: crate::common::gui::CoordMod::NumbersAside,
			..Default::default()
		}
	}
}
struct BaghChalSquareDrawer;
impl<G> SquareDrawer<G> for BaghChalSquareDrawer
	where G: BoardGame,
		G::M: BoardMove<G> {
	fn draw(&self, painter: &egui::Painter, style: &BoardStyle, _game: &G, square: &Rect, x_coord: u8, y_coord: u8) {
		let index = if style.mirrored {
			Bitboard5x5::index_from_coords(x_coord, 4-y_coord)
		} else {
			Bitboard5x5::index_from_coords(x_coord, y_coord)
		};
		let stroke = egui::Stroke::new(1.0, style.dark_color);

		painter.rect_filled(*square, 0.0, style.uniform_color);
		match index {
			0 => {
				painter.line(vec![square.center(), square.right_center()], stroke);
				painter.line(vec![square.center(), square.center_top()], stroke);
				painter.line(vec![square.center(), square.right_top()], stroke);
			}
			1 | 3 => {
				painter.line(vec![square.left_center(), square.right_center()], stroke);
				painter.line(vec![square.center(), square.center_top()], stroke);
			}
			2 => {
				painter.line(vec![square.left_center(), square.right_center()], stroke);
				painter.line(vec![square.center(), square.center_top()], stroke);
				painter.line(vec![square.center(), square.left_top()], stroke);
				painter.line(vec![square.center(), square.right_top()], stroke);
			}
			4 => {
				painter.line(vec![square.center(), square.left_center()], stroke);
				painter.line(vec![square.center(), square.center_top()], stroke);
				painter.line(vec![square.center(), square.left_top()], stroke);
			}
			5 | 15 => {
				painter.line(vec![square.center(), square.right_center()], stroke);
				painter.line(vec![square.center_bottom(), square.center_top()], stroke);
			}
			6 | 8 | 12 | 16| 18 => {
				painter.line(vec![square.left_bottom(), square.right_top()], stroke);
				painter.line(vec![square.right_bottom(), square.left_top()], stroke);
				painter.line(vec![square.left_center(), square.right_center()], stroke);
				painter.line(vec![square.center_bottom(), square.center_top()], stroke);
			}
			7 | 11 | 13 | 17 => {
				painter.line(vec![square.left_center(), square.right_center()], stroke);
				painter.line(vec![square.center_bottom(), square.center_top()], stroke);
			}
			9 | 19 => {
				painter.line(vec![square.center(), square.left_center()], stroke);
				painter.line(vec![square.center(), square.center_bottom()], stroke);
				painter.line(vec![square.center(), square.center_top()], stroke);
			}
			10 => {
				painter.line(vec![square.center(), square.right_center()], stroke);
				painter.line(vec![square.center(), square.center_bottom()], stroke);
				painter.line(vec![square.center(), square.center_top()], stroke);
				painter.line(vec![square.center(), square.right_bottom()], stroke);
				painter.line(vec![square.center(), square.right_top()], stroke);
			}
			14 => {
				painter.line(vec![square.center(), square.left_center()], stroke);
				painter.line(vec![square.center(), square.center_bottom()], stroke);
				painter.line(vec![square.center(), square.center_top()], stroke);
				painter.line(vec![square.center(), square.left_bottom()], stroke);
				painter.line(vec![square.center(), square.left_top()], stroke);
			}
			20 => {
				painter.line(vec![square.center(), square.right_center()], stroke);
				painter.line(vec![square.center(), square.center_bottom()], stroke);
				painter.line(vec![square.center(), square.right_bottom()], stroke);
			}
			21 | 23 => {
				painter.line(vec![square.left_center(), square.right_center()], stroke);
				painter.line(vec![square.center(), square.center_bottom()], stroke);
			}
			22 => {
				painter.line(vec![square.left_center(), square.right_center()], stroke);
				painter.line(vec![square.center(), square.right_bottom()], stroke);
				painter.line(vec![square.center(), square.center_bottom()], stroke);
				painter.line(vec![square.center(), square.left_bottom()], stroke);
			}
			24 => {
				painter.line(vec![square.center(), square.left_center()], stroke);
				painter.line(vec![square.center(), square.center_bottom()], stroke);
				painter.line(vec![square.center(), square.left_bottom()], stroke);
			}
			_ => {}
		}
	}
}
pub fn create_board() -> GenericBoardApp<BaghChal> {
	let mut board=GenericBoardApp::new(BaghChal::default(), new_move_searcher_vec("Material".into(), BaghChalMaterialEval{}, 8));
	//board.board_drawer.get_style_mut().dark_color=egui::Color32::from_rgb(181, 136, 99);
	//board.board_drawer.get_style_mut().light_color=Color32::from_rgb(240, 217, 181);
	//board.board_drawer.get_style_mut().show_coordinates_mod=crate::common::gui::CoordMod::NumbersAside;
	board.board_drawer.set_square_drawer(Box::new(BaghChalSquareDrawer{}));
	board.max_depth = 13;
	board.depth = 8;
	board
}
