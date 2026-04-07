use eframe::egui;
use egui::{Color32, Stroke};
use crate::chinese_checkers::bitboard::ChineseCheckerBoard;
use crate::chinese_checkers::game::ChineseCheckersMaterialEval;
use crate::chinese_checkers::{ChineseCheckers, ChineseCheckersPlayer, Move};
use crate::common::ai::incomplete_info_searcher::ExpectiMinimaxBuilder;
use crate::common::gui::board_app::GenericBoardApp;
use crate::common::gui::board_drawer::{BoardDrawer, DefaultBoardDrawer, SquareDrawer};
use crate::common::gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType, Shape};


impl BoardMove<ChineseCheckers> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.from as u16)
	}

	fn to(&self) -> u16 {
		self.to as u16
	}
}
impl EGUIPieceType for ChineseCheckersPlayer {
	fn shape(&self) -> Shape {
		match self {
			ChineseCheckersPlayer::Red => Shape::Circle { color: Color32::RED, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
			ChineseCheckersPlayer::Blue => Shape::Circle { color: Color32::BLUE, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
			ChineseCheckersPlayer::Green => Shape::Circle { color: Color32::GREEN, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
			ChineseCheckersPlayer::Yellow => Shape::Circle { color: Color32::YELLOW, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
			ChineseCheckersPlayer::Black => Shape::Circle { color: Color32::BLACK, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
			ChineseCheckersPlayer::White => Shape::Circle { color: Color32::WHITE, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
		}
	}
}
impl BoardGame for ChineseCheckers {

	type PieceType=ChineseCheckersPlayer;

	fn width(&self) -> u8 {
		13
	}

	fn height(&self) -> u8 {
		17
	}

	fn play(&mut self, mv: Self::M) {
		let _ = self.play(mv);
	}
	#[inline(always)]
	fn result(&self) -> crate::common::GameResult {
		match self.winner() {
			Some(p) => crate::common::GameResult::Player(p.idx()),
			None => crate::common::GameResult::OnGoing,
		}
	}

	fn current_player(&self) -> crate::common::Player {
		match self.turn {
			ChineseCheckersPlayer::Red => crate::common::Player::Player(0),
			ChineseCheckersPlayer::Blue => crate::common::Player::Player(1),
			ChineseCheckersPlayer::Green => crate::common::Player::Player(2),
			ChineseCheckersPlayer::Yellow => crate::common::Player::Player(3),
			ChineseCheckersPlayer::Black => crate::common::Player::Player(4),
			ChineseCheckersPlayer::White => crate::common::Player::Player(5),
		}
	}
	fn nb_players(&self) -> u8 {
		self.nb_players
	}
	fn get_name(&self, p: crate::common::Player) -> String {
		match p {
			crate::common::Player::Player1 => "White".into(),
			crate::common::Player::Player2 => "Black".into(),
			crate::common::Player::Player(idx) => {
				let players = Self::active_players(self.nb_players);
				if idx < players.len() as u8 {
					return match players[idx as usize] {
						ChineseCheckersPlayer::Red => "Red",
						ChineseCheckersPlayer::Blue => "Blue",
						ChineseCheckersPlayer::Green => "Green",
						ChineseCheckersPlayer::Yellow => "Yellow",
						ChineseCheckersPlayer::Black => "Black",
						ChineseCheckersPlayer::White => "White",
					}.into()
				}
				"".into()
			}
			crate::common::Player::RandomMove => unreachable!(),
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		if self.black.get(x, y) {
			return  Some(ChineseCheckersPlayer::Black);
		}
		if self.white.get(x, y) {
			return  Some(ChineseCheckersPlayer::White);
		}
		if self.yellow.get(x, y) {
			return  Some(ChineseCheckersPlayer::Yellow);
		}
		if self.green.get(x, y) {
			return  Some(ChineseCheckersPlayer::Green);
		}
		if self.blue.get(x, y) {
			return  Some(ChineseCheckersPlayer::Blue);
		}
		if self.red.get(x, y) {
			return  Some(ChineseCheckersPlayer::Red);
		}
		None
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		ChineseCheckerBoard::index_from_coords(x, y) as u16
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		ChineseCheckerBoard::coords_from_index(index as usize)
	}
	fn default_style() -> BoardStyle {
		let mut style = BoardStyle::default();
		style.uniform_color=egui::Color32::from_rgb(200, 190, 125);
		style.show_coordinates_mod=crate::common::gui::CoordMod::None;
		style.played_highlights_shape=Shape::Rect { color: Color32::from_rgba_unmultiplied(120, 120, 120, 128), size: 1.0, text: "".into(), text_color: Color32::BLACK, stroke_color: None };
		style
	}
	fn get_position_from_string(&self, fen: &String) -> Result<Self, String> {
		Self::from_fen(fen)
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
}
#[derive(Default)]
struct ChineseCheckersBoardDrawer(DefaultBoardDrawer<ChineseCheckers>);
impl BoardDrawer<ChineseCheckers> for ChineseCheckersBoardDrawer
{
	fn draw_board(&self, ui: &mut egui::Ui, game: &ChineseCheckers, can_interact: bool) -> Option<(u8, u8)> {
		let mut click = None;
		let w = game.width();
		let h = game.height();
		
		let avail_w = ui.available_width().max(10.0)* if self.get_style().show_coordinates_mod.is_aside() {0.90} else {1.0};
		let avail_h = ui.available_height().max(10.0)* if self.get_style().show_coordinates_mod.is_aside() {0.90} else {1.0};
		let cell_size = (avail_w / (w as f32+0.5)).min(avail_h / h as f32);

		let board_width  = cell_size * (w as f32 + 0.5);
		let board_height  = cell_size * h as f32;

		let left_margin = if self.get_style().show_coordinates_mod.is_aside() {cell_size * 0.6} else {0.0};
		let bottom_margin = if self.get_style().show_coordinates_mod.is_aside() {cell_size * 0.6} else {0.0};

		let x_offset = (avail_w - board_width) / 2.0;
		//let y_offset = (avail_h - board_height) / 2.0;

		//let total_w = board_width + left_margin;
		let total_h = board_height + bottom_margin;
		

		let (outer_rect, response) =
				ui.allocate_exact_size(egui::vec2(avail_w, total_h), egui::Sense::click());
		let painter = ui.painter_at(outer_rect);

		let board_rect = egui::Rect::from_min_size(
			egui::pos2(outer_rect.left() + x_offset + left_margin, outer_rect.top()),
			egui::vec2(board_width, board_height),
		);
		if let Some(c) = &self.get_style().clear_color {
			painter.rect_filled(board_rect, 0.0, *c);
		}
		if let Some(pos) = response.interact_pointer_pos() {
			if can_interact && board_rect.contains(pos) {
				let (x_coord,y_coord) = self.pixel_to_coords(&board_rect, cell_size, pos, w, h).unwrap();

				if x_coord < w && y_coord < h {
					if response.clicked() {
						println!("click coords: {} {}", x_coord, y_coord);
						let index = ChineseCheckers::index_from_coords(x_coord, y_coord);
						//println!("index_from_coords: {}", index);
						println!("coords_from_index: {} -> {:?}", index, ChineseCheckers::coords_from_index(index));
						click = Some((x_coord, y_coord));
					}
				}
			}
		}
		for y_coord in 0..h {
			for x_coord in 0..w {
				let pos = Self::coords_to_pixel(&self, &board_rect, cell_size, x_coord, y_coord, h);

				let square = egui::Rect::from_min_size(
					pos,
					egui::vec2(cell_size, cell_size),
				);

				// Background
				self.get_square_drawer().draw(&painter, &self.get_style(), &game, &square, x_coord, y_coord);
			}
		}
		// Overlay
		self.get_square_drawer().draw_overlay(&painter, &self.get_style(), &game, &board_rect, cell_size);
		for y_coord in 0..h {
			for x_coord in 0..w {
				let pos = Self::coords_to_pixel(&self, &board_rect, cell_size, x_coord, y_coord, h);

				let square = egui::Rect::from_min_size(
					pos,
					egui::vec2(cell_size, cell_size),
				);

				// played highlights
				if self.get_played_highlights().contains(&ChineseCheckers::index_from_coords(x_coord, y_coord)) {
					self.get_style().played_highlights_shape.draw(ui.painter(), square.center(), cell_size);
					//painter.rect_filled(square, 0.0, Color32::from_rgba_unmultiplied(150, 150, 250, 80));
				}
				// Pieces
				if let Some(piece) = game.piece_at(x_coord, y_coord) {
					piece.draw(ui, square.center(), cell_size);
				} else if let Some(shape) = self.get_style().empty_cell_shape.as_ref() {
					shape.draw(ui.painter(), square.center(), cell_size);
				}
			}
		}
		// selected square
		if let Some(sindex) = self.get_selected() {
			let (sx, sy) = ChineseCheckers::coords_from_index(sindex);
			//let x = board_rect.left() + sx as f32 * cell_size;
			//let y = board_rect.top() + (h - 1 - sy) as f32 * cell_size;
			let pos = Self::coords_to_pixel(&self, &board_rect, cell_size, sx, sy, h);
			let x = pos.x;
			let y = pos.y;
			let square = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_size, cell_size));
			//painter.rect_stroke(square, 0.0, egui::Stroke::new(3.0, egui::Color32::YELLOW), egui::StrokeKind::Outside);
			self.get_style().selected_highlights_shape.draw(ui.painter(), square.center(), cell_size);
		}
		// legal moves highlights
		for &index in self.get_legal_highlights() {
			let (tx, ty) = ChineseCheckers::coords_from_index(index);
			//let x = board_rect.left() + tx as f32 * cell_size;
			//let y = board_rect.top() + (h - 1 - ty) as f32 * cell_size;
			let pos = Self::coords_to_pixel(&self, &board_rect, cell_size, tx, ty, h);
			let x = pos.x;
			let y = pos.y;
			let square = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_size, cell_size));
			
			self.get_style().legal_highlights_shape.draw(ui.painter(), square.center(), cell_size);
		}

		self.draw_coordinates_aside(&painter, board_rect, cell_size, w, h);
		click
		//self.0.draw_board(ui, game, can_interact)
	}
	fn coords_to_pixel(&self, board_rect: &egui::Rect, cell_size: f32, x_coord: u8, y_coord: u8, h: u8) -> egui::Pos2 {
		let (x_visual, y_visual) = if self.get_style().mirrored {
			(x_coord, h - 1 - y_coord)
		} else {
			(x_coord, y_coord)
		};

		let x = board_rect.left() + x_visual as f32 * cell_size + if y_coord % 2 == 0 {0.5*cell_size} else {0.0};
		let y = board_rect.top() + (h - 1 - y_visual) as f32 * cell_size;

		egui::Pos2::new(x, y)
	}

	fn pixel_to_coords(&self, board_rect: &egui::Rect, cell_size: f32, pos: egui::Pos2, w: u8, h: u8) -> Option<(u8, u8)> {
		if !board_rect.contains(pos) { return None; }

		let x_off = pos.x - board_rect.left();
		let y_off = pos.y - board_rect.top();

		let y_visual = (h - 1) - (y_off / cell_size).floor() as u8;
		let x_visual = ((x_off - if y_visual % 2 == 0 {0.5*cell_size} else {0.0}) / cell_size).floor() as u8;
		
		let (x_coord, y_coord) = if self.get_style().mirrored {
			(x_visual, h - 1 - y_visual)
		} else {
			(x_visual, y_visual)
		};

		if x_coord < w && y_coord < h {
			Some((x_coord, y_coord))
		} else {
			None
		}
	}
	fn get_square_drawer(&self) -> &Box<dyn SquareDrawer<ChineseCheckers>> {
		self.0.get_square_drawer()
	}

	fn set_square_drawer(&mut self, sq_drawer: Box<dyn SquareDrawer<ChineseCheckers>>) {
		self.0.set_square_drawer(sq_drawer)
	}

	fn get_style(&self) -> &BoardStyle {
		self.0.get_style()
	}

	fn get_style_mut(&mut self) -> &mut BoardStyle {
		self.0.get_style_mut()
	}

	fn set_style(&mut self, style: BoardStyle) {
		self.0.set_style(style)
	}

	fn get_selected(&self) -> Option<u16> {
		self.0.get_selected()
	}

	fn set_selected(&mut self, selected: Option<u16>) {
		self.0.set_selected(selected)
	}

	fn clear_selection(&mut self) {
		self.0.clear_selection()
	}

	fn get_legal_highlights(&self) -> &Vec<u16> {
		self.0.get_legal_highlights()
	}

	fn set_legal_highlights(&mut self, legal_highlights: Vec<u16>) {
		self.0.set_legal_highlights(legal_highlights)
	}

	fn get_played_highlights(&self) -> &Vec<u16> {
		self.0.get_played_highlights()
	}

	fn set_played_highlights(&mut self, played_highlights: Vec<u16>) {
		self.0.set_played_highlights(played_highlights)
	}

	fn full_reset(&mut self)  {
		self.0.full_reset()
	}
}
struct ChineseCheckerSquareDrawer{}
impl SquareDrawer<ChineseCheckers> for ChineseCheckerSquareDrawer {
	fn draw(&self, painter: &egui::Painter, style: &BoardStyle, _game: &ChineseCheckers, square: &egui::Rect, x_coord: u8, y_coord: u8)
	{
		painter.rect_filled(*square, 0.0, style.uniform_color);
		if ChineseCheckerBoard::is_playable(x_coord, y_coord) {
			let actives = ChineseCheckers::active_players(_game.nb_players);
			painter.circle_stroke(square.center(), square.width()*0.7/2.0, Stroke::new(3.0, style.dark_color));
			if actives.contains(&ChineseCheckersPlayer::Red) && ChineseCheckerBoard::initial_blue().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::RED);
			}
			else if actives.contains(&ChineseCheckersPlayer::Blue) && ChineseCheckerBoard::initial_red().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::BLUE);
			}
			else if actives.contains(&ChineseCheckersPlayer::Yellow) && ChineseCheckerBoard::initial_green().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::YELLOW);
			}
			else if actives.contains(&ChineseCheckersPlayer::Green) && ChineseCheckerBoard::initial_yellow().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::GREEN);
			}
			else if actives.contains(&ChineseCheckersPlayer::Black) && ChineseCheckerBoard::initial_white().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::BLACK);
			}
			else if actives.contains(&ChineseCheckersPlayer::White) && ChineseCheckerBoard::initial_black().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::WHITE);
			}
		}
	}

	fn draw_overlay(&self, _painter: &egui::Painter, _style: &BoardStyle, _game: &ChineseCheckers, _board_rect: &egui::Rect, _cell_size: f32) {
	}
}
pub fn create_board() -> GenericBoardApp<ChineseCheckers> {
	//let engines: Vec<Box<dyn AIEngineProvider<ChineseCheckers, Engine=Box<dyn AIEngine<ChineseCheckers>>>>> = vec![
	//	Box::new(ExpectiMinimaxBuilder::new("Material".into(), ChineseCheckersMaterialEval::new(ChineseCheckersPlayer::Red), 5)),
	//];
	let ai_provider = ExpectiMinimaxBuilder::new("Material".into(), ChineseCheckersMaterialEval::default(), 4);
	let mut board = GenericBoardApp::new(ChineseCheckers::new(6), vec![Box::new(ai_provider)]);
	//let mut board=GenericBoardApp::new(ChineseCheckers::new(6), engines);
	board.board_drawer=Box::new(ChineseCheckersBoardDrawer::default());
	board.board_drawer.set_square_drawer(Box::new(ChineseCheckerSquareDrawer{}));
	board
}