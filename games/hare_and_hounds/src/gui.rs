use kudchuet::egui;
use egui::{Color32, Rect};
use kudchuet::ai::{AIBuilder, AIEngineProvider, MoveSearcherBuilder};
use kudchuet::ai::move_search::perfect_solver::PerfectSolver;
use kudchuet::gui::board_drawer::SquareDrawer;
use kudchuet::{GameOutcome, Player};
use kudchuet::gui::{BoardGame, BoardMove, BoardStyle, CoordMod, EGUIPieceType, GUIGame, GUIMove};
use kudchuet::gui::shapes::{Shape, StrokeData};

use kudchuet::gui::game_app::GenericBoardApp;
use crate::rules::Board;

use super::game::HareAndHoundsEval;
//use super::game::HareAndHoundsEvalDumb;

use super::rules::{Cell, Move, HareAndHounds};

impl GUIMove<HareAndHounds> for Move {
	fn click_sequence(&self, _state: &HareAndHounds) -> Vec<<HareAndHounds as GUIGame>::Click> {
		self.click_sequence_board_move_default(_state)
	}
}
impl BoardMove<HareAndHounds> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.from as u16)
	}

	fn to(&self) -> u16 {
		self.to as u16
	}
}
impl EGUIPieceType for Cell {
	fn shape(&self) -> Shape {
		match self {
			Cell::Empty => unreachable!(),
			Cell::Hound => Shape::Circle { fill_color: Some(Color32::ORANGE), size: 0.7, text: None, stroke: Some(StrokeData::default()) },
			Cell::Hare => Shape::Circle { fill_color: Some(Color32::LIGHT_GRAY), size: 0.7, text: None, stroke: Some(StrokeData::default()) },
		}
	}
}

impl GUIGame for HareAndHounds {
	type Click = u16;
	type Settings = kudchuet::gui::DefaultSettings;
	type Style = kudchuet::gui::BoardStyle;
	
	fn get_name(&self, p: Player) -> String {
		match p {
			Player::PLAYER1 => "Hounds".into(),
			Player::PLAYER2 => "Har".into(),
			_ => unreachable!()
		}
	}
	fn play(&mut self, mv: Self::M) {
		self.play_unchecked(mv);
	}

	fn result(&self) -> GameOutcome {
		self.result()
	}

	fn current_player(&self) -> Player {
		
		match self.turn() {
			true => Player::PLAYER2,
			false => Player::PLAYER1,
		}
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			show_coordinates_mod: CoordMod::NumbersAside,
			//checkerboard_mod: CheckerBoardMod::None,
			..Default::default()
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
	use kudchuet::gui::board_drawer::BoardDrawer;
	let engines: Vec<Box<dyn AIEngineProvider<HareAndHounds>>> = vec![
		Box::new(MoveSearcherBuilder::new("Simple", HareAndHoundsEval::new(), 4)),
		Box::new(AIBuilder::<HareAndHounds, PerfectSolver<HareAndHounds>>::new("Perfect")),
	];
	let mut board=GenericBoardApp::new(HareAndHounds::default(), engines);
	board.game_drawer.set_square_drawer(Box::new(HareAndHoundsSquareDrawer{}));
	board.depth = 12;
	board.max_depth = 25;
	board
}