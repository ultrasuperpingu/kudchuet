
use bitboard::Bitboard;
use eframe::egui;
use egui::{Color32, Rect, Stroke, Vec2};
use minimax::Game;
use crate::bitboard::BitboardAbalone;
use crate::game::AbaloneMaterialEval;
use crate::rules::{Abalone, Cell, HEXES, Hex, Move, idx};
use kudchuet::common::gui::board_app::GenericBoardApp;
use kudchuet::common::gui::board_drawer::{BoardDrawer, DefaultBoardDrawer, PieceDrawer, SquareDrawer};
use kudchuet::common::gui::{BoardGame, BoardMove, BoardStyle, CheckerBoardMod, EGUIPieceType};
use kudchuet::common::gui::shapes::{Shape, StrokeData};
use kudchuet::common::{GameResult, Player, new_move_searcher_vec};



impl BoardMove<Abalone> for Move {
	fn click_sequence(&self, _state: &Abalone) -> Vec<u16> {
		let mut clicks = vec![];
		match self {
			Move::Simple { from, dir } => {
				clicks.push(idx(*from).unwrap() as u16);
				clicks.push(idx(from.add(Hex::DIRS[*dir as usize])).unwrap() as u16);
			},
			Move::Side { start, line_dir, side_dir, len } => {
				clicks.push(idx(*start).unwrap() as u16);
				let mut extrimity = *start;
				for _ in 0..*len-1 {
					extrimity = extrimity.add(Hex::DIRS[*line_dir as usize]);
				}
				clicks.push(idx(extrimity).unwrap() as u16);
				clicks.push(idx(start.add(Hex::DIRS[*side_dir as usize])).unwrap() as u16);
			},
			Move::Push { start, dir, len } => {
				clicks.push(idx(*start).unwrap() as u16);
				let mut extrimity = *start;
				for _ in 0..*len {
					extrimity=extrimity.add(Hex::DIRS[*dir as usize]);
				}
				clicks.push(idx(extrimity).unwrap() as u16);
			},
		}
		clicks
	}
	fn to_uci(&self) -> Option<String> {
		Some(self.to_string())
	}
}

impl EGUIPieceType for Cell {
	fn shape(&self) -> Shape {
		match self {
			Cell::White => Shape::Circle{
				fill_color: Some(Color32::WHITE),
				size: 0.7,
				text: None,
				stroke: Some(StrokeData { stroke: Stroke::new(3.0, Color32::BLACK), kind: egui::StrokeKind::Inside })
			},
			Cell::Black => Shape::Circle{
				fill_color:Some(Color32::BLACK),
				size: 0.7,
				text: None,
				stroke: None
			},
			Cell::Empty => unreachable!(),
		}
	}
}
impl BoardGame for Abalone {
	type PieceType=Cell;

	fn width(&self) -> u8 {
		8
	}

	fn height(&self) -> u8 {
		8
	}

	fn current_player(&self) -> Player {
		self.turn
	}
	fn get_name(&self, p: Player) -> String {
		match p {
			Player::PLAYER1 => "Black".into(),
			Player::PLAYER2 => "White".into(),
			_ => unreachable!(),
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		if self.white.get(x, y) {
			Some(Cell::White)
		} else if self.black.get(x, y) {
			Some(Cell::Black)
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
		BitboardAbalone::index_from_coords(x, y) as u16
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		BitboardAbalone::coords_from_index(index as usize)
	}

	fn get_position_from_string(&self, pos_str: &String) -> Result<Self, String> {
		Self::from_fen(pos_str)
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
	fn move_to_string(&self, m: &Self::M) -> Option<String> {
		m.to_uci()
	}
	fn move_from_string(&self, m_str: &String) -> Result<Self::M, String> {
		Move::from_uci(m_str)
	}
	fn default_style() -> BoardStyle {
		let mut style = BoardStyle::default();
		style.checkerboard_mod=CheckerBoardMod::None;
		style.uniform_color=Color32::from_rgb(40, 17, 51);
		style.selected_highlights_shape=Shape::Circle {
			fill_color: None,
			size: 0.8,
			stroke: Some(StrokeData {stroke: Stroke::new(5.0, Color32::from_rgb(240, 220, 80)), kind: egui::StrokeKind::Inside}),
			text: None };
		let board_color = Color32::from_rgb(0xaf, 0x69 ,0x49);
		let outline_color = Color32::from_rgb(0x70, 0x20, 30);
		style.empty_cell_shape=Some(
			Shape::Circle {
				fill_color: Some(board_color),
				size: 0.95,
				stroke: Some(StrokeData {stroke: egui::Stroke::new(3.0, outline_color), kind: egui::StrokeKind::Inside}),
				text: None,
			});
		style
	}
}

struct AbaloneBoardDrawer<G>(DefaultBoardDrawer<G>);
impl BoardDrawer<Abalone> for AbaloneBoardDrawer<Abalone>
{
	fn draw_board(&self, ui: &mut egui::Ui, game: &Abalone, can_interact: bool) -> Option<(u8, u8)> {
		let available = ui.available_rect_before_wrap();
		let painter = ui.painter_at(available);

		//let center = available.center();
		let cell_radius = 22.0;   // taille des billes
		let size = 28.0;          // taille de la cellule hex


		//let board_color = Color32::from_rgb(0xaf, 0x69 ,0x49);
		//let outline_color = Color32::from_rgb(0x70, 0x20, 30);
		//let selection_color = Color32::from_rgb(240, 220, 80);

		// click this frame
		let mut clicked_hex: Option<Hex> = None;
		

		for (index, hex) in HEXES.iter().enumerate() {
			let hex = *hex;
			let cell = game.cell(hex);
			if cell.is_none() {
				continue;
			}
			let cell = cell.unwrap();
			let (tx, ty) = Abalone::coords_from_index(index as u16);
			let pos = self.coords_to_pixel(&available, size, tx, ty, game.height());

			// background
			//painter.circle_filled(pos, cell_radius, board_color);
			//painter.circle_stroke(pos, cell_radius, egui::Stroke::new(1.5, outline_color));
			if let Some(s) = &self.get_style().empty_cell_shape {
				s.draw(ui.painter(), pos, cell_radius * 2.0);
			}

			// marble
			if let Some(c) = match cell {
				Cell::Empty => None,
				Cell::Black => Some(Color32::BLACK),
				Cell::White => Some(Color32::WHITE),
			} {
				painter.circle_filled(pos, cell_radius * 0.8, c);
			}
			if let Some(sel) = self.get_selected() {
				if Some(sel as usize) == idx(hex) {
					self.get_style().selected_highlights_shape.draw(ui.painter(), pos, cell_radius*2.0);
					/*painter.circle_stroke(
						pos,
						cell_radius * 0.9,
						Stroke::new(3.0, selection_color),
					);*/
				}
			}
			if can_interact {
				// interaction
				let rect = Rect::from_center_size(pos, Vec2::splat(cell_radius * 2.0));
				let response = ui.interact(rect, ui.id().with((hex.q, hex.r)), egui::Sense::click());
				if response.clicked() {
					clicked_hex = Some(hex);
				}
			}
		}
		// legal moves highlights
		for &index in self.get_legal_highlights() {
			let (tx, ty) = Abalone::coords_from_index(index);
			let pos = self.coords_to_pixel(&available, size, tx, ty, game.height());
			let square = egui::Rect::from_center_size(pos, egui::vec2(size, size));
			
			self.get_style().legal_highlights_shape.draw(ui.painter(), square.center(), size);
		}
		if let Some(h) = clicked_hex {
			idx(h).map(BitboardAbalone::coords_from_index)
		} else {
			None
		}
	}
	fn coords_to_pixel(&self, board_rect: &egui::Rect, cell_size: f32, x_coord: u8, y_coord: u8, _h: u8) -> egui::Pos2 {
		let index = BitboardAbalone::index_from_coords(x_coord, y_coord);
		if index > 60 {
			return egui::Pos2{x:0.0,y:0.0};
		}
		let hex = HEXES[index];
		let x = (hex.q as f32 * 1.732 * cell_size) + (hex.r as f32 * 0.866 * cell_size);
		let y = hex.r as f32 * 1.5 * cell_size;

		egui::Pos2::new(board_rect.center().x + x, board_rect.center().y + y)
	}

	fn get_square_drawer(&self) -> &dyn SquareDrawer<Abalone> {
		self.0.get_square_drawer()
	}

	fn set_square_drawer(&mut self, sq_drawer: Box<dyn SquareDrawer<Abalone>>) {
		self.0.set_square_drawer(sq_drawer)
	}

	fn get_piece_drawer(&self) -> &dyn PieceDrawer<Abalone> {
		self.0.get_piece_drawer()
	}

	fn get_piece_drawer_mut(&mut self) -> &mut dyn PieceDrawer<Abalone> {
		self.0.get_piece_drawer_mut()
	}

	fn set_piece_drawer(&mut self, sq_drawer: Box<dyn PieceDrawer<Abalone>>) {
		self.0.set_piece_drawer(sq_drawer)
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
pub fn create_board() -> GenericBoardApp<Abalone> {
	let mut board=GenericBoardApp::new(Abalone::default(), new_move_searcher_vec("Material".into(), AbaloneMaterialEval{}, 4));
	board.board_drawer = Box::new(AbaloneBoardDrawer(DefaultBoardDrawer::new()));
	*board.board_drawer.get_style_mut()=Abalone::default_style();
	board.max_depth = 6;
	board.depth = 4;
	board
}

	/*fn pixel_to_coords(
		&self,
		board_rect: &egui::Rect,
		cell_size: f32,
		pos: egui::Pos2,
		w: u8,
		h: u8,
	) -> Option<(u8, u8)> {

		// 1. Décalage par rapport au centre
		let dx = pos.x - board_rect.center().x;
		let dy = pos.y - board_rect.center().y;

		// 2. Conversion inverse screen → axial (q, r)
		let r_float = dy / (1.5 * cell_size);
		let x_prime = dx / cell_size;
		let q_float = (x_prime - 0.866 * r_float) / 1.732;

		// 3. Arrondi axial → hex discret
		let hex = axial_round(q_float, r_float);

		// 4. Trouver l’index correspondant dans HEXES
		let index = HEXES.iter().position(|h| h.q == hex.q && h.r == hex.r)?;

		// 5. Convertir index → (x_coord, y_coord)
		let x = (index % 8) as u8;
		let y = (index / 8) as u8;

		// 6. Vérifier les bornes
		if x < w && y < h {
			Some((x, y))
		} else {
			None
		}
	}
	
fn axial_round(q: f32, r: f32) -> super::rules::Hex {
	// conversion axial -> cube
	let x = q;
	let z = r;
	let y = -x - z;

	let mut rx = x.round();
	let mut ry = y.round();
	let mut rz = z.round();

	let x_diff = (rx - x).abs();
	let y_diff = (ry - y).abs();
	let z_diff = (rz - z).abs();

	if x_diff > y_diff && x_diff > z_diff {
		rx = -ry - rz;
	} else if y_diff > z_diff {
		ry = -rx - rz;
	} else {
		rz = -rx - ry;
	}

	// cube -> axial
	super::rules::Hex {
		q: rx as i8,
		r: rz as i8,
	}
}

	*/