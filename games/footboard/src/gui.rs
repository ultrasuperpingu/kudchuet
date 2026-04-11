use eframe::egui;
use egui::{Align2, Color32, FontId, Rect, Stroke, Vec2};
use egui_field_editor::EguiInspect;
use crate::bitboard::Bitboard9x13;
use kudchuet::gui::board_drawer::{DefaultSquareDrawer, PieceDrawer, SquareDrawer};
use kudchuet::{GameResult, Player, new_move_searcher_vec};
use kudchuet::gui::{BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType};
use kudchuet::gui::shapes::{Shape, StrokeData, TextData};

use kudchuet::gui::board_app::GenericBoardApp;
use crate::rules::Action;

use super::game::FootboardEvalDumb;

use super::rules::{Cell, Move, FootBoard};

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

			if player == Player::PLAYER1 {
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
			Cell::White => Shape::Circle { fill_color: Some(Color32::WHITE), size: 0.7, text: None, stroke: Some(StrokeData::default()) },
			Cell::Black => Shape::Circle { fill_color: Some(Color32::BLACK), size: 0.7, text: None, stroke: Some(StrokeData::default()) },
			Cell::WhiteWithBall => Shape::Circle {
				fill_color: Some(Color32::YELLOW),
				size: 0.7,
				text: Some(TextData {
					text: "⚽".into(),
					color: Color32::BLACK,
					size: 0.5,
				}),
				stroke: Some(StrokeData::default())
			},
			Cell::BlackWithBall => Shape::Circle {
				fill_color: Some(Color32::YELLOW),
				size: 0.7,
				text: Some(TextData {
					text: "⚽".into(),
					color: Color32::WHITE,
					size: 0.5,
				}),
				stroke: Some(StrokeData::default())
			},
			Cell::Ball => Shape::Circle {
				fill_color: Some(Color32::WHITE),
				size: 0.52,
				text: Some(TextData {
					text: "⚽".into(),
					color: Color32::WHITE,
					size: 0.5,
				}),
				stroke: None
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

	fn result(&self) -> GameResult {
		self.result()
	}

	fn current_player(&self) -> Player {
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
		BoardStyle {
			checkerboard_mod: CheckerBoardMod::None,
			uniform_color: Color32::DARK_GREEN,
			dark_color: Color32::BLACK,
			light_color: Color32::WHITE,
			square_stroke_color: Some(Color32::BLACK),
			show_coordinates_mod: CoordMod::FileRankAside,
			..Default::default()
		}
	}
	
	fn do_random(&mut self) {
			}
	
	fn nb_players(&self) -> u8 {
				2
			}
	
	fn get_name(&self, p: Player) -> String {
				p.to_string()
			}
	
	fn position_to_string(&self) -> Option<String> {
				None
			}
	
	fn game_to_string(&self, _mvs: &[Self::M]) -> Option<String> {
				None
			}
	
	fn game_from_string(&self, _game_str: &String) -> Result<Vec<Self::M>, String> {
				Err("Not Supported".into())
			}
	
	fn get_position_from_string(&self, _pos_str: &String) -> Result<Self, String> {
				Err("Not Supported".into())
			}
	
	fn move_from_string(&self, m_str: &String) -> Result<Self::M, String> {
				Self::M::from_uci(m_str)
			}
	
	fn move_to_string(&self, m: &Self::M) -> Option<String> {
				Self::M::to_uci(m)
			}
	
	fn play_random(&mut self) {}
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
		painter.text(board_rect.right_bottom(), Align2::RIGHT_BOTTOM, (90.0 - (_game.turn as f32 / 30.0) * 90.0).round().to_string() +":00", FontId::monospace(cell_size*0.8), Color32::WHITE);
		painter.line(points, stroke);
	}
}
#[derive(EguiInspect, Debug)]
struct FootboardPieceDrawer {
	#[inspect(slider(min=0.0, max=1.0))]
	size: f32,
	p1_color1: Color32,
	p1_color2: Option<Color32>,
	p1_color3: Option<Color32>,
	#[inspect(slider(min=0.0, max=1.0))]
	p1_angle: f32,
	#[inspect(slider(min=0.0, max=1.0))]
	p1_width: f32,
	p2_color1: Color32,
	p2_color2: Option<Color32>,
	p2_color3: Option<Color32>,
	#[inspect(slider(min=0.0, max=1.0))]
	p2_angle: f32,
	#[inspect(slider(min=0.0, max=1.0))]
	p2_width: f32,
	p1_stroke: Option<Stroke>,
	p2_stroke: Option<Stroke>,
}
impl FootboardPieceDrawer {
	fn new() -> Self {
		Self{
			size:1.0,
			p1_color1: Color32::from_rgb(250, 250, 250),
			p1_color2: Some(Color32::from_rgb(0, 157, 222)),
			p1_color3: None,//Some(Color32::from_rgb(250, 250, 250)),
			p1_angle: 0.38,
			p1_width: 0.5,
			p2_color1: Color32::from_rgb(5, 45, 177),
			p2_color2: Some(Color32::from_rgb(244, 0, 14)),
			p2_color3: Some(Color32::from_rgb(5, 45, 177)),
			p2_angle: 0.25,
			p2_width: 0.5,
			p1_stroke: None,
			p2_stroke: None,
		}
	}
}

impl FootboardPieceDrawer {
	fn draw_weighted_stripped_circle(
		painter: &egui::Painter,
		center: egui::Pos2,
		radius: f32,
		colors: &[Color32],
		weights: &[f32], // sum = 1.0
		angle: f32,      // 0..1
	) {
		let steps = 64;
		let epsilon = 2.0;

		let dir = egui::Vec2::angled(angle * std::f32::consts::TAU);
		let normal = egui::vec2(-dir.y, dir.x);

		let total_width = radius * 2.0;

		let mut offset = -total_width / 2.0;

		for (color, weight) in colors.iter().zip(weights.iter()) {
			let band_width = weight * total_width;

			let min = offset;
			let max = offset + band_width;

			let mut points = Vec::new();

			for i in 0..=steps {
				let theta = i as f32 / steps as f32 * std::f32::consts::TAU;
				let p = center + egui::Vec2::new(theta.cos(), theta.sin()) * radius;

				let proj = (p - center).dot(normal);

				if proj >= min - epsilon && proj <= max + epsilon {
					points.push(p);
				}
			}

			if points.len() >= 3 {
				painter.add(egui::Shape::convex_polygon(
					points,
					*color,
					egui::Stroke::NONE,
				));
			}

			offset += band_width;
		}
	}
}
impl PieceDrawer<FootBoard> for FootboardPieceDrawer
{
	fn draw(&self, painter: &egui::Painter, _style: &BoardStyle, _game: &FootBoard, piece: <FootBoard as BoardGame>::PieceType, square: &Rect, _x_coord: u8, _y_coord: u8)
	{
		//piece.shape().draw(painter, square.center(), square.width());
		match piece {
			Cell::White => {
				if let (Some(c2), Some(c3)) = (self.p1_color2,self.p1_color3) {
					Self::draw_weighted_stripped_circle(painter, square.center(), self.size * square.width()/2.0, &[self.p1_color1,c2, c3], &[(1.0-self.p1_width)/2.0,self.p1_width, (1.0-self.p1_width)/2.0], self.p1_angle);
					
				}
				else if let Some(c2) = self.p1_color2 {
					Self::draw_weighted_stripped_circle(painter, square.center(), self.size * square.width()/2.0, &[self.p1_color1,c2], &[self.p1_width,1.0-self.p1_width],self.p1_angle);
				}
				else {
					painter.circle_filled(square.center(), square.width()/2.0 * self.size, self.p1_color1);
				}
			},
			Cell::Black => {
				if let (Some(c2), Some(c3)) = (self.p2_color2,self.p2_color3) {
					Self::draw_weighted_stripped_circle(painter, square.center(), self.size * square.width()/2.0, &[self.p2_color1,c2, c3], &[(1.0-self.p2_width)/2.0,self.p2_width, (1.0-self.p2_width)/2.0], self.p2_angle);
				}
				else if let Some(c2) = self.p2_color2 {
					Self::draw_weighted_stripped_circle(painter, square.center(), self.size * square.width()/2.0, &[self.p2_color1,c2], &[self.p2_width,1.0-self.p2_width],self.p2_angle);
				}
				else {
					painter.circle_filled(square.center(), square.width()/2.0 * self.size, self.p2_color1);
				}
			},
			Cell::WhiteWithBall => {
				if let (Some(c2), Some(c3)) = (self.p1_color2,self.p1_color3) {
					Self::draw_weighted_stripped_circle(painter, square.center(), self.size * square.width()/2.0, &[self.p1_color1,c2, c3], &[(1.0-self.p1_width)/2.0,self.p1_width, (1.0-self.p1_width)/2.0], self.p1_angle);
				}
				else if let Some(c2) = self.p1_color2 {
					Self::draw_weighted_stripped_circle(painter, square.center(), self.size * square.width()/2.0, &[self.p1_color1,c2], &[self.p1_width,1.0-self.p1_width],self.p1_angle);
				}
				else {
					painter.circle_filled(square.center(), square.width()/2.0 * self.size, self.p1_color1);
				}
				painter.text(square.center(), Align2::CENTER_CENTER, "⚽", FontId::monospace(square.width() * 0.8), Color32::BLACK);
			},
			Cell::BlackWithBall => {
				if let (Some(c2), Some(c3)) = (self.p2_color2,self.p2_color3) {
					Self::draw_weighted_stripped_circle(painter, square.center(), self.size * square.width()/2.0, &[self.p2_color1,c2, c3], &[(1.0-self.p2_width)/2.0,self.p2_width, (1.0-self.p2_width)/2.0], self.p2_angle);
				}
				else if let Some(c2) = self.p2_color2 {
					Self::draw_weighted_stripped_circle(painter, square.center(), self.size * square.width()/2.0, &[self.p2_color1,c2], &[self.p2_width,1.0-self.p2_width],self.p2_angle);
				}
				else {
					painter.circle_filled(square.center(), square.width()/2.0 * self.size, self.p2_color1);
				}
				painter.text(square.center(), Align2::CENTER_CENTER, "⚽", FontId::monospace(square.width() * 0.8), Color32::BLACK);
			},
			Cell::Ball => {
				piece.shape().draw(painter, square.center(), square.width());
			},
			Cell::Empty => {
			},
		}
	}

	fn has_custom_properties(&self) -> bool {
		true
	}

	fn set_default(&mut self) {
		*self=Self::new();
	}
}
pub fn create_board() -> GenericBoardApp<FootBoard> {
	let mut board=GenericBoardApp::new(FootBoard::default(), new_move_searcher_vec("Dumb".into(), FootboardEvalDumb{}, 3));
	board.depth = 3;
	board.max_depth = 6;
	board.board_drawer.set_square_drawer(Box::new(FootboardSquareDrawer::new()));
	board.board_drawer.set_piece_drawer(Box::new(FootboardPieceDrawer::new()));
	board
}