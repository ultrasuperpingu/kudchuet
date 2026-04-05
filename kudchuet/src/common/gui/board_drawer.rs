use egui::{Color32, Painter, Pos2, Rect};

use crate::common::gui::{BoardGame, BoardMove, BoardStyle, CoordMod, EGUIPieceType};


pub trait BoardDrawer<G: BoardGame>
	where G::M : BoardMove<G>
{
	fn draw_board(&self, ui: &mut egui::Ui, game: &G, can_interact: bool) -> Option<(u8, u8)> {
		let mut click = None;
		let w = game.width();
		let h = game.height();
		
		let avail_w = ui.available_width().max(10.0)* if self.get_style().show_coordinates_mod.is_aside() {0.90} else {1.0};
		let avail_h = ui.available_height().max(10.0)* if self.get_style().show_coordinates_mod.is_aside() {0.90} else {1.0};
		let cell_size = (avail_w / w as f32).min(avail_h / h as f32);

		let board_width  = cell_size * w as f32;
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
						let index = G::index_from_coords(x_coord, y_coord);
						//println!("index_from_coords: {}", index);
						println!("coords_from_index: {} -> {:?}", index, G::coords_from_index(index));
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
				if self.get_played_highlights().contains(&G::index_from_coords(x_coord, y_coord)) {
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
			let (sx, sy) = G::coords_from_index(sindex);
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
			let (tx, ty) = G::coords_from_index(index);
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
	}

	fn draw_coordinates_aside(&self,  painter: &egui:: Painter, board_rect: egui::Rect, cell_size: f32, w: u8, h: u8) {
		if !self.get_style().show_coordinates_mod.is_aside() {
			return;
		}

		for file in 0..w {
			let pos = Self::coords_to_pixel(&self, &board_rect, cell_size, file, 0, h);
			//let x = board_rect.left() + (file as f32 + 0.5) * cell_size;
			let x = pos.x + 0.5 * cell_size;
			let y = board_rect.bottom() + 2.0;
			let t = if self.get_style().show_coordinates_mod.is_file_rank()  { ((b'a' + file) as char).to_string() } else { (file+1).to_string() };
			painter.text(
				egui::pos2(x, y),
				egui::Align2::CENTER_TOP,
				t,
				egui::FontId::proportional(cell_size * 0.4),
				egui::Color32::WHITE,
			);
		}

		for rank in (0..h).rev() {
			let pos = Self::coords_to_pixel(&self, &board_rect, cell_size, 0, rank, h);
			let x = board_rect.left() - 6.0;
			//let y = board_rect.top() + ((h-1-rank) as f32 + 0.5) * cell_size;
			let y = pos.y + 0.5 * cell_size;

			painter.text(
				egui::pos2(x, y),
				egui::Align2::RIGHT_CENTER,
				(rank + 1).to_string(),
				egui::FontId::proportional(cell_size * 0.4),
				egui::Color32::WHITE,
			);
		}
	}
	fn coords_to_pixel(&self, board_rect: &egui::Rect, cell_size: f32, x_coord: u8, y_coord: u8, h: u8) -> Pos2 {
		let (x_visual, y_visual) = if self.get_style().mirrored {
			(x_coord, h - 1 - y_coord)
		} else {
			(x_coord, y_coord)
		};

		let x = board_rect.left() + x_visual as f32 * cell_size;
		let y = board_rect.top() + (h - 1 - y_visual) as f32 * cell_size;

		Pos2::new(x, y)
	}

	fn pixel_to_coords(&self, board_rect: &egui::Rect, cell_size: f32, pos: egui::Pos2, w: u8, h: u8) -> Option<(u8, u8)> {
		if !board_rect.contains(pos) { return None; }

		let x_off = pos.x - board_rect.left();
		let y_off = pos.y - board_rect.top();

		let x_visual = (x_off / cell_size).floor() as u8;
		let y_visual = (h - 1) - (y_off / cell_size).floor() as u8;

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
	fn get_square_drawer(&self) -> &Box<dyn SquareDrawer<G>>;
	fn set_square_drawer(&mut self, sq_drawer: Box<dyn SquareDrawer<G>>);
	fn get_style(&self) -> &BoardStyle;
	fn get_style_mut(&mut self) -> &mut BoardStyle;
	fn set_style(&mut self, style: BoardStyle);
	fn get_selected(&self) -> Option<u16>;
	fn set_selected(&mut self, selected: Option<u16>);
	fn clear_selection(&mut self);
	fn get_legal_highlights(&self) -> &Vec<u16>;
	fn set_legal_highlights(&mut self, legal_highlights: Vec<u16>);
	fn get_played_highlights(&self) -> &Vec<u16>;
	fn set_played_highlights(&mut self, played_highlights: Vec<u16>);
	fn full_reset(&mut self);
	fn load_style(&mut self, ctx: &egui::Context) {
		if let Some(json) = ctx.data_mut(|d| d.get_persisted::<String>("theme".into())) {
			eprintln!("loading theme: {}", json);
			if let Ok(settings) = serde_json::from_str(&json) {
				*self.get_style_mut() = settings;
			}
		}
	}
	fn save_style(&mut self, ctx: &egui::Context) {
		let json = serde_json::to_string(&self.get_style()).unwrap();
		eprintln!("saving theme: {}", json);
		ctx.data_mut(|d| d.insert_persisted("theme".into(), json));
	}
}
pub struct DefaultBoardDrawer<G> {
	style: BoardStyle,
	square_drawer: Box<dyn SquareDrawer<G>>,

	selected: Option<u16>,
	legal_highlights: Vec<u16>,
	played_highlights: Vec<u16>,
	//intermediate_state: Option<G>,
}
impl<G: BoardGame> Default for DefaultBoardDrawer<G>
	where G::M : BoardMove<G> {
	fn default() -> Self {
		Self {
			style: G::default_style(),
			square_drawer: Box::new(DefaultSquareDrawer::new()),
			selected: None,
			legal_highlights: vec![],
			played_highlights: vec![],
			//intermediate_state:None,
		}
	}
}
impl<G: BoardGame> DefaultBoardDrawer<G>
	where G::M : BoardMove<G>
{
	pub fn new() -> Self {
		Self {
			style: G::default_style(),
			square_drawer: Box::new(DefaultSquareDrawer::new()),
			selected: None,
			legal_highlights: vec![],
			played_highlights: vec![],
			//intermediate_state:None,
		}
	}
}
impl<G: BoardGame> BoardDrawer<G> for DefaultBoardDrawer<G>
	where G::M: BoardMove<G>
{
	fn get_square_drawer(&self) -> &Box<dyn SquareDrawer<G>> {
		&self.square_drawer
	}

	fn set_square_drawer(&mut self, sq_drawer: Box<dyn SquareDrawer<G>>) {
		self.square_drawer = sq_drawer;
	}

	fn get_style(&self) -> &BoardStyle {
		&self.style
	}

	fn get_style_mut(&mut self) -> &mut BoardStyle {
		&mut self.style
	}

	fn set_style(&mut self, style: BoardStyle) {
		self.style = style;
	}

	fn get_selected(&self) -> Option<u16> {
		self.selected
	}

	fn set_selected(&mut self, selected: Option<u16>) {
		self.selected = selected
	}

	fn get_legal_highlights(&self) -> &Vec<u16> {
		&self.legal_highlights
	}

	fn set_legal_highlights(&mut self, legal_highlights: Vec<u16>) {
		self.legal_highlights = legal_highlights;
	}

	fn get_played_highlights(&self) -> &Vec<u16> {
		&self.played_highlights
	}

	fn set_played_highlights(&mut self, played_highlights: Vec<u16>) {
		self.played_highlights = played_highlights;
	}
	fn clear_selection(&mut self) {
		self.selected = None;
		self.legal_highlights.clear();
		//self.intermediate_state = None;
	}

	fn full_reset(&mut self) {
		self.clear_selection();
		self.played_highlights.clear();
	}
}

pub trait SquareDrawer<G>
	where G: BoardGame,
		G::M: BoardMove<G> {
	fn draw(&self, painter: &Painter, style: &BoardStyle, _game: &G, square: &Rect, x_coord: u8, y_coord: u8)
	{
		let (bg_color, txt_color) = match style.checkerboard_mod {
			super::CheckerBoardMod::None => (style.uniform_color, Color32::BLACK),
			super::CheckerBoardMod::EvenDark => {
				if (x_coord + y_coord) % 2 == 1 {
					(style.light_color, Color32::BLACK)
				} else {
					(style.dark_color, Color32::WHITE)
				}
			},
			super::CheckerBoardMod::OddDark => {
				if (x_coord + y_coord) % 2 == 0 {
					(style.light_color, Color32::BLACK)
				} else {
					(style.dark_color, Color32::WHITE)
				}
			},
		};

		painter.rect_filled(*square, 0.0, bg_color);

		if let Some(color) = style.square_stroke_color {
			painter.rect_stroke(
				*square,
				0.0,
				egui::Stroke::new(1.0, color),
				egui::StrokeKind::Middle,
			);
		}
		match style.show_coordinates_mod {
			CoordMod::FileRankOnSquare => {
				if x_coord == 0 {
					painter.text(
						square.left_top(),
						egui::Align2::LEFT_TOP,
						(y_coord+1).to_string(),
						egui::FontId { size: square.width() * 0.2, family: egui::FontFamily::Monospace },
						txt_color
					);
				}
				if y_coord == 0 {
					painter.text(
						square.right_bottom(),
						egui::Align2::RIGHT_BOTTOM,
						(b'a' + x_coord) as char,
						egui::FontId { size: square.width() * 0.2, family: egui::FontFamily::Monospace },
						txt_color
					);
				}
			},
			CoordMod::NumbersOnSquare => {
				let index = G::index_from_coords(x_coord, y_coord) + 1;
				if index <= _game.width() as u16 * _game.height() as u16 {
					painter.text(
						square.right_bottom(),
						egui::Align2::RIGHT_BOTTOM,
						index.to_string(),
						egui::FontId { size: square.width() * 0.2, family: egui::FontFamily::Monospace },
						txt_color.gamma_multiply_u8(128)
					);
				}
			},
			_ => {}
		}
	}
	fn draw_overlay(&self, _painter: &egui::Painter, _style: &BoardStyle, _game: &G, _board_rect: &Rect, _cell_size: f32) {
	}
}

#[derive(Default)]
pub struct DefaultSquareDrawer;
impl DefaultSquareDrawer
{
	pub fn new() -> Self {
		Self { }
	}
}

impl<G> SquareDrawer<G> for DefaultSquareDrawer
	where G: BoardGame,
		G::M: BoardMove<G> 
{
	
}
