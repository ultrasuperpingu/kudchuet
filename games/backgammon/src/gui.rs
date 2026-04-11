use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind};
use minimax::Game;

use kudchuet::ai::incomplete_info_searcher::ExpectiMinimaxBuilder;
use kudchuet::gui::board_drawer::SquareDrawer;
use kudchuet::gui::{BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType};
use kudchuet::{GameResult, Player};
use kudchuet::gui::board_app::GenericBoardApp;
use kudchuet::gui::shapes::{Shape, StrokeData, TextData};

use crate::rules::{P1_BAR, P1_OUT, P2_BAR, P2_OUT};
use crate::game::{BackgammonMaterialEval, BackgammonSimpleEval};

use super::rules::{Backgammon, Move};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
	Player1(u8),
	Player2(u8),
}

impl EGUIPieceType for Piece {
	fn shape(&self) -> Shape {
		match self {
			Piece::Player1(nb) => Shape::Circle {
				fill_color: Some(Color32::RED),
				text: if *nb > 1 { Some(TextData {text: nb.to_string(), size: 0.5, color: Color32::BLACK }) } else { None },
				size: 0.98,
				stroke: Some(StrokeData { stroke: Stroke::new(3.0, Color32::BLACK), kind: StrokeKind::Inside }),
			},
			Piece::Player2(nb) => Shape::Circle {
				fill_color: Some(Color32::BLACK),
				text: if *nb > 1 { Some(TextData {text: nb.to_string(), size: 0.5, color: Color32::WHITE }) } else { None },
				size: 0.98,
				stroke: None,
			},
		}
	}
}
impl BoardMove<Backgammon> for Move {
	fn click_sequence(&self, _state: &Backgammon) -> Vec<u16> {
		let mut seq = Vec::new();
		if self.is_random() {
			return seq;
		}
		let mv = self.clone().to_player_move().unwrap();

		for m in &mv.moves[..mv.len as usize] {
			seq.push(m.from as u16);
			seq.push(m.to as u16);
		}
		seq
	}
	fn compute_intermediate_state(&self, state: &Backgammon, clicks: &[u16]) -> Option<Backgammon> {
		if clicks.len() < 2 { return None; }

		let mut sim = state.clone();
		let mut i = 0;

		while i + 1 < clicks.len() {
			let from = clicks[i] as u8;
			let to = clicks[i+1] as u8;
			sim.apply_single_move(from, to);
			i += 2;
		}

		Some(sim)
	}
}

impl Backgammon {
	fn bar_piece(&self, y: u8) -> Option<Piece> {
		match y {
			4 => if self.on_bar[0] > 0 { Some(Piece::Player1(self.on_bar[0]))} else {None},
			5 => if self.on_bar[1] > 0 { Some(Piece::Player2(self.on_bar[1]))} else {None},
			_ => None
		}
	}
	fn out_piece(&self, y: u8) -> Option<Piece> {
		match y {
			4 => if self.outside[0] > 0 { Some(Piece::Player1(self.outside[0]))} else {None},
			5 => if self.outside[1] > 0 { Some(Piece::Player2(self.outside[1]))} else {None},
			_ => None
		}
	}

}
fn col(x: u8) -> u8 {
	if x > 6 { x - 1 } else { x }
}
impl BoardGame for Backgammon {
	type PieceType = Piece;

	fn width(&self) -> u8 {
		14
	}

	fn height(&self) -> u8 {
		10
	}

	fn current_player(&self) -> Player {
		self.current_player()
	}
	fn do_random(&mut self) {
		self.roll_dice();
	}
	fn result(&self) -> GameResult {
		match <Self as Game>::get_winner(self) {
			Some(minimax::Winner::Draw) => GameResult::Draw,
			Some(minimax::Winner::PlayerJustMoved) => if self.current_player() == Player::PLAYER1 {GameResult::Player2} else {GameResult::Player1},
			Some(minimax::Winner::PlayerToMove) => if self.current_player() == Player::PLAYER2 {GameResult::Player2} else {GameResult::Player1},
			None => GameResult::OnGoing,
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		if x == 6 {
			return self.bar_piece(y);
		}
		if x == 13 {
			return self.out_piece(y);
		}
		let index = match y {
			5..=9 => {
				if x == 6 {
					u8::MAX
				} else {
					12 + col(x)
				}
			},
			0..=4 => {
				if x == 6 {
					u8::MAX
				} else {
					11 - col(x)
				}
			},
			_ => return None,
		} as usize;
		if index == u8::MAX as usize {
			if y == 5 {
				let content = self.on_bar[1];
				if content > 0 {
					if self.current_player == Player::PLAYER1 {
						return Some(Piece::Player2(content));
					}
				}
			}
			if y == 4 {
				let content = self.on_bar[1];
				if content > 0 {
					if self.current_player == Player::PLAYER1 {
						return Some(Piece::Player1(content));
					}
				}
			}
			return None;
		}
		let content = self.board[index];

		if content > 0 {
			if y == 0 || y == 9 {
				if content > 5 {
					Some(Piece::Player1(content as u8 - 4))
				} else {
					Some(Piece::Player1(1))
				}
			} else if y < 5 && y < content as u8 || y >= 5 && 9 - y < content as u8{
				Some(Piece::Player1(1))
			} else {
				None
			}
		} else if content < 0 {
			if y == 0 || y == 9 {
				if -content > 5 {
					Some(Piece::Player2(-content as u8 - 4))
				} else {
					Some(Piece::Player2(1))
				}
			} else if y < 5 && y < (-content) as u8 || y >= 5 && 9 - y < -content as u8 {
				Some(Piece::Player2(1))
			} else {
				None
			}
			//Some(Piece::Player2((-content) as u8))
		} else {
			None
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		if x != 6 && x != 13 {
			return if y < 5 {
				11 - col(x) as u16
			} else {
				12 + col(x) as u16
			};
		}
		if x == 13 {
			if y >= 5 {
				P2_OUT as u16
			} else {
				P1_OUT as u16
			}
		} else if (x,y) == (6, 4) {
			P1_BAR as u16
		} else if (x,y) == (6, 5) {
			P2_BAR as u16
		} else {
			u8::MAX as u16
		}
	}

	fn coords_from_index(index: u16) -> (u8, u8) {
		if index == P1_BAR as u16 {
			return (6, 4);
		}
		if index == P2_BAR as u16 {
			return (6, 5);
		}
		if index == P1_OUT as u16 {
			return (13, 4);
		}
		if index == P2_OUT as u16 {
			return (13, 5);
		}
		let line = index as u8 / 12;
		let mut col = index as u8 % 12;
		
		if line == 0 {
			if col > 5 { col+=1; }
			(12-col, 0)
		} else {
			if col > 5 { col+=1; }
			(col, 9)
		}
	}
	fn play_random(&mut self) {
		self.roll_dice();
	}
	fn get_position_from_string(&self, _pos_str: &String) -> Result<Self, String> {
		Backgammon::from_position_notation(_pos_str)
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_position_notation())
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: CheckerBoardMod::OddDark,
			uniform_color: Color32::from_rgb(40, 70, 125),
			dark_color: Color32::from_rgb(60, 45, 30),
			light_color: Color32::from_rgb(200, 175, 140),
			show_coordinates_mod: CoordMod::None,
			square_stroke_color: None,
			..Default::default()
		}
	}
}
fn dice_string(dice: u8) -> String {
	match dice {
		1 => "🎲1".into(), //⚀
		2 => "🎲2".into(), //⚁
		3 => "🎲3".into(), //⚂
		4 => "🎲4".into(), //⚃
		5 => "🎲5".into(), //⚄
		6 => "🎲6".into(), //⚅
		_ => dice.to_string()
	}
}

struct BackgammonSquareDrawer;
impl SquareDrawer<Backgammon> for BackgammonSquareDrawer
{
	fn draw(&self, _painter: &egui::Painter, _style: &BoardStyle, _game: &Backgammon, _square: &Rect, _x_coord:u8, _y_coord:u8) {
	}
	fn draw_overlay(&self, painter: &egui::Painter, style: &BoardStyle, game: &Backgammon, board_rect: &Rect, cell_size: f32) {
		painter.rect_filled(*board_rect, 0, style.uniform_color);
		let center = board_rect.center();
		let left_bottom = board_rect.left_bottom();
		let left_top = board_rect.left_top();
		for i in 0..6 {
			let colors = if style.checkerboard_mod == CheckerBoardMod::EvenDark {
				if i % 2 == 0 { (style.dark_color, style.light_color) } else {(style.light_color, style.dark_color)}
			} else {
				if i % 2 == 1 { (style.dark_color, style.light_color) } else {(style.light_color, style.dark_color)}
			};
			let points = vec![
				Pos2::new(left_bottom.x + i as f32 * cell_size, left_bottom.y),
				Pos2::new(left_bottom.x + (i as f32 + 0.5) * cell_size, center.y),
				Pos2::new(left_bottom.x + (i+1) as f32 * cell_size, left_bottom.y),
			];
			painter.add(egui::Shape::convex_polygon(
				points,
				colors.1,
				Stroke::new(0.0, Color32::BLACK),
			));
			let points = vec![
				Pos2::new(left_top.x + i as f32 * cell_size, left_top.y),
				Pos2::new(left_top.x + (i as f32 + 0.5) * cell_size, center.y),
				Pos2::new(left_top.x + (i+1) as f32 * cell_size, left_top.y),
			];
			painter.add(egui::Shape::convex_polygon(
				points,
				colors.0,
				Stroke::new(0.0, Color32::BLACK),
			));
			let points = vec![
				Pos2::new(left_bottom.x + (i+7) as f32 * cell_size, left_bottom.y),
				Pos2::new(left_bottom.x + ((i+7) as f32 + 0.5) * cell_size, center.y),
				Pos2::new(left_bottom.x + (i+8) as f32 * cell_size, left_bottom.y),
			];
			painter.add(egui::Shape::convex_polygon(
				points,
				colors.1,
				Stroke::new(0.0, Color32::BLACK),
			));
			let points = vec![
				Pos2::new(left_top.x + (i+7) as f32 * cell_size, left_top.y),
				Pos2::new(left_top.x + ((i+7) as f32 + 0.5) * cell_size, center.y),
				Pos2::new(left_top.x + (i+8) as f32 * cell_size, left_top.y),
			];
			painter.add(egui::Shape::convex_polygon(
				points,
				colors.0,
				Stroke::new(0.0, Color32::BLACK),
			));
		}
		painter.rect_filled(
			Rect {
				min: Pos2 { x: center.x - cell_size, y: board_rect.top() },
				max: Pos2 { x: center.x, y: board_rect.bottom() },
			},
			0.0,
			style.dark_color
		);
		painter.rect_filled(
			Rect {
				min: Pos2 { x: board_rect.right() - cell_size, y: board_rect.top() },
				max: Pos2 { x: board_rect.right(), y: board_rect.bottom() },
			},
			0.0,
			style.dark_color
		);
		painter.line_segment(
			[
				Pos2{x:board_rect.center_top().x - cell_size*0.5, y: board_rect.center_top().y},
			 	Pos2{x:board_rect.center_bottom().x - cell_size*0.5, y: board_rect.center_bottom().y},
			],
			Stroke::new(3.0, Color32::BLACK)
		);
		if game.dice.len() >= 2 {
			painter.text(
				Pos2{x:center.x - 0.5 * cell_size, y: board_rect.top()},
				Align2::CENTER_TOP,
				dice_string(game.dice[0])+"\n"+dice_string(game.dice[1]).as_str(),
				FontId::monospace(cell_size*0.5),
				Color32::WHITE
			);
		}
	}
}
pub fn create_board() -> GenericBoardApp<Backgammon> {
	let ai_provider = ExpectiMinimaxBuilder::new("Material".into(), BackgammonMaterialEval::default(), 4);
	let ai_provider2 = ExpectiMinimaxBuilder::new("Simple".into(), BackgammonSimpleEval::default(), 4);
	let mut board = GenericBoardApp::new(Backgammon::new(), vec![Box::new(ai_provider), Box::new(ai_provider2)]);
	board.board_drawer.set_square_drawer(Box::new(BackgammonSquareDrawer{}));
	board
}