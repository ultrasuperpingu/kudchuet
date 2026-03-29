use eframe::egui;
use egui::{Align2, Color32, FontId, Rect, Vec2};
use crate::common::bitboards::Bitboard9x13;
use crate::common::gui::board_drawer::{DefaultSquareDrawer, SquareDrawer};
use crate::common::{Player, new_move_searcher_vec};
use crate::common::gui::{BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType, Shape};

use crate::common::gui::board_app::GenericBoardApp;
use crate::footboard::Action;

use super::game::FootboardEvalDumb;

use super::{Cell, Move, FootBoard};

impl BoardMove<FootBoard> for Move {

	fn click_sequence(&self, _state: &FootBoard) -> Vec<u16> {
		let mut seq = Vec::new();
		for action in self.0.iter().flatten() {
			match action {
				Action::Move { from, to } | Action::Shoot { from, to } => {
					seq.push(*from as u16);
					seq.push(*to as u16);
				}
			}
		}
		seq
	}
	/*fn compute_intermediate_state(state: &FootBoard, _legals:&[Move], clicks: &[u8]) -> Option<FootBoard> {
		if clicks.len() < 2 { return None; }

		let mut sim = state.clone();
		let player = state.current_player();
		let mut i = 0;

		while i + 1 < clicks.len() {
			let from = clicks[i];
			let to = clicks[i+1];

			if player == Player::Player1 {
				if sim.player1.get_at_index(from as usize) {
					sim.player1.reset_at_index(from as usize);
					sim.player1.set_at_index(to as usize);
					
					if from == sim.ball {
						sim.ball = to;
					}
				}
			} else {
				if sim.player2.get_at_index(from as usize) {
					sim.player2.reset_at_index(from as usize);
					sim.player2.set_at_index(to as usize);
					
					if from == sim.ball {
						sim.ball = to;
					}
				}
			}
			i += 2;
		}

		Some(sim)
	}*/
	fn compute_intermediate_state(&self, state: &FootBoard, clicks: &[u16]) -> Option<FootBoard> {
		if clicks.len() < 2 { return None; }

		let mut sim = state.clone();
		let player = state.current_player();
		let mut i = 0;

		while i + 1 < clicks.len() {
			//let from = clicks[i];
			//let to = clicks[i+1];

			if player == Player::Player1 {
				if let Some(a) = self.0[i/2] {
					FootBoard::play_action(&a, &mut sim.player1, &mut sim.ball);
				} 
			} else {
				if let Some(a) = self.0[i/2] {
					FootBoard::play_action(&a, &mut sim.player2, &mut sim.ball);
				} 
			}
			i += 2;
		}

		Some(sim)
	}
}
impl EGUIPieceType for Cell {
	fn shape(&self) -> Shape {
		match self {
			Cell::Empty => unreachable!(),
			Cell::White => Shape::Circle{color:Color32::WHITE, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
			Cell::Black => Shape::Circle{color:Color32::BLACK, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
			Cell::WhiteWithBall => Shape::Circle{color:Color32::WHITE, size: 0.7, text: "⚽".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
			Cell::BlackWithBall => Shape::Circle{color:Color32::BLACK, size: 0.7, text: "⚽".into(), text_color: Color32::WHITE, stroke_color: Some(Color32::BLACK)},
			Cell::Ball => Shape::String { text: "⚽".into(), color: Color32::BLACK },
		}
	}
	fn draw(&self, ui: &mut egui::Ui, center: egui::Pos2, cell_size: f32) {
		match self {
			Cell::Empty => unreachable!(),
			Cell::White => {
				Shape::Circle{color:Color32::WHITE, text: "".into(), size: 0.7, text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)}.draw(ui, center, cell_size);
			},
			Cell::Black => {
				Shape::Circle{color:Color32::BLACK, text: "".into(), size: 0.7, text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)}.draw(ui, center, cell_size);
			},
			Cell::WhiteWithBall => {
				Shape::Circle{color:Color32::WHITE, text: "⚽".into(), size: 0.7, text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)}.draw(ui, center, cell_size)
			},
			Cell::BlackWithBall => {
				Shape::Circle{color:Color32::BLACK, text: "⚽".into(), size: 0.7, text_color: Color32::WHITE, stroke_color: Some(Color32::BLACK)}.draw(ui, center, cell_size);
			},
			Cell::Ball => {
				ui.painter().circle_filled(center, cell_size * 0.26, Color32::WHITE);
					ui.painter().text(
						center,
						egui::Align2::CENTER_CENTER,
							"⚽",
							egui::FontId::monospace(cell_size * 0.6),
							Color32::BLACK
						);
				//Shape::String { text: "⚽".into(), color: Color32::BLACK }.draw(ui, center, cell_size);
			},
		}
	}
}

impl BoardGame for FootBoard {
	type PieceType=Cell;

	fn width(&self) -> u8 {
		9
	}

	fn height(&self) -> u8 {
		13
	}

	fn legal_moves(&self) -> Vec<Self::M> {
		let mut moves=vec![];
		self.legal_moves(&mut moves);
		moves
	}
	fn play(&mut self, mv: Self::M) {
		self.play_unchecked(&mv);
	}

	fn result(&self) -> crate::common::GameResult {
		self.result()
	}

	fn current_player(&self) -> crate::common::Player {
		self.turn()
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		match self.cell(x, y) {
			Cell::Empty => None,
			e => Some(e),
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard9x13::index_from_coords(x, y) as u16
	}
	fn coords_from_index(i: u16) -> (u8, u8) {
		Bitboard9x13::coords_from_index(i as usize)
	}
	fn default_style() -> BoardStyle {
		let mut style = BoardStyle::default();
		style.checkerboard_mod=CheckerBoardMod::None;
		style.uniform_color=Color32::DARK_GREEN;
		style.dark_color=Color32::BLACK;
		style.light_color=Color32::WHITE;
		style.square_stroke_color=Some(Color32::BLACK);
		style.show_coordinates_mod=CoordMod::FileRankAside;
		style
	}
}

struct FootboardSquareDrawer {
	default: DefaultSquareDrawer
}
impl FootboardSquareDrawer {
	fn new() -> Self {
		Self{
			default:DefaultSquareDrawer{}
		}
	}
}
impl SquareDrawer<FootBoard> for FootboardSquareDrawer
//where G: BoardGame,
//		G::M: BoardMove<G> 
{


	fn draw(&self, painter: &egui::Painter, style: &BoardStyle, game: &FootBoard, square: &Rect, x_coord:u8,y_coord:u8) {
		let index = Bitboard9x13::index_from_coords(x_coord, y_coord);
		match index {
			0|1|2|6|7|8 => {
				painter.rect_filled(*square, 0.0, style.dark_color);
			}
			116|115|114|110|109|108 => {
				painter.rect_filled(*square, 0.0, style.dark_color);
			}
			_ => {
				self.default.draw(painter, style, game, square, x_coord, y_coord);
			}
		}
	}
	fn draw_overlay(&self, painter: &egui::Painter, style: &BoardStyle, _game: &FootBoard, board_rect: &Rect, cell_size: f32) {
		let mut field_rect = *board_rect;
		*field_rect.bottom_mut()-=cell_size;
		*field_rect.top_mut()+=cell_size;
		let stroke = egui::Stroke::new(5.0, style.light_color);
		painter.rect_stroke(field_rect, 0.0, stroke, egui::StrokeKind::Middle);
		painter.circle_filled(field_rect.center(), cell_size*0.1, style.light_color);
		painter.circle_stroke(field_rect.center(), cell_size*1.4, stroke);
		let points=vec![field_rect.left_center(), field_rect.right_center()];
		painter.line(points, stroke);
		let points=vec![
			field_rect.center_bottom()+Vec2{x:-1.5*cell_size,y:0.0},
			field_rect.center_bottom()+Vec2{x:-1.5*cell_size,y:-2.0*cell_size},
			field_rect.center_bottom()+Vec2{x:1.5*cell_size,y:-2.0*cell_size},
			field_rect.center_bottom()+Vec2{x:1.5*cell_size,y:0.0},
			
		];
		painter.line(points, stroke);
		let points=vec![
			field_rect.center_top()+Vec2{x:-1.5*cell_size,y:0.0},
			field_rect.center_top()+Vec2{x:-1.5*cell_size,y:2.0*cell_size},
			field_rect.center_top()+Vec2{x:1.5*cell_size,y:2.0*cell_size},
			field_rect.center_top()+Vec2{x:1.5*cell_size,y:0.0},
			
		];
		painter.text(board_rect.left_bottom(), Align2::LEFT_BOTTOM, _game.score1.to_string()+" - "+_game.score2.to_string().as_str(), FontId::monospace(cell_size*0.8), Color32::WHITE);
		painter.text(board_rect.right_bottom(), Align2::RIGHT_BOTTOM, (90 as f32 - (_game.turn as f32 / 30.0) * 90.0).round().to_string() +":00", FontId::monospace(cell_size*0.8), Color32::WHITE);
		painter.line(points, stroke);
	}
}
pub fn create_board() -> GenericBoardApp<FootBoard> {
	let mut board=GenericBoardApp::new(FootBoard::default(), new_move_searcher_vec("Dumb".into(), FootboardEvalDumb{}, 3));
	board.depth = 3;
	board.max_depth = 6;
	board.board_drawer.set_square_drawer(Box::new(FootboardSquareDrawer::new()));
	board
}