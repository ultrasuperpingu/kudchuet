use std::fmt::Debug;

use egui::{Color32, Pos2, Rect, Ui, Vec2};
use egui_field_editor::EguiInspect;
use minimax::Game;
use serde::{Deserialize, Serialize};
use crate::common::gui::input_handler::MoveResult;
use crate::common::gui::board_drawer::BoardDrawer;


use super::{GameResult, Player};
use self::options_panel::RightTab;

pub mod board_app;
pub mod board_drawer;
pub mod options_panel;
pub mod game_state_manager;
pub mod input_handler;

#[derive(EguiInspect, Clone, PartialEq, Eq, Copy, Debug, Serialize, Deserialize)]
pub enum CoordMod {
	None,
	NumbersOnSquare,
	FileRankOnSquare,
	NumbersAside,
	FileRankAside,
}
impl CoordMod {
	pub const fn is_aside(&self) -> bool {
		match self {
			CoordMod::None => false,
			CoordMod::NumbersOnSquare => false,
			CoordMod::FileRankOnSquare => false,
			CoordMod::NumbersAside => true,
			CoordMod::FileRankAside => true,
		}
	}
	pub const fn is_on_square(&self) -> bool {
		match self {
			CoordMod::None => false,
			CoordMod::NumbersOnSquare => true,
			CoordMod::FileRankOnSquare => true,
			CoordMod::NumbersAside => false,
			CoordMod::FileRankAside => false,
		}
	}
	pub const fn is_file_rank(&self) -> bool {
		match self {
			CoordMod::None => false,
			CoordMod::NumbersOnSquare => false,
			CoordMod::FileRankOnSquare => true,
			CoordMod::NumbersAside => false,
			CoordMod::FileRankAside => true,
		}
	}
	pub const fn is_number(&self) -> bool {
		match self {
			CoordMod::None => false,
			CoordMod::NumbersOnSquare => true,
			CoordMod::FileRankOnSquare => false,
			CoordMod::NumbersAside => true,
			CoordMod::FileRankAside => false,
		}
	}
	pub fn is_none(&self) -> bool {
		*self == CoordMod::None
	}
}
#[derive(EguiInspect, Default, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum CheckerBoardMod {
	None,
	#[default]
	EvenDark,
	OddDark
}

#[derive(EguiInspect, Default, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum HalfSizeOffsetMod {
	#[default]
	None,
	Even,
	Odd
}
#[derive(EguiInspect, Clone, Serialize, Deserialize)]
pub struct BoardStyle {
	pub checkerboard_mod: CheckerBoardMod,
	pub half_size_offset_mod: HalfSizeOffsetMod,
	pub clear_color: Option<egui::Color32>,
	pub light_color: egui::Color32,
	pub dark_color: egui::Color32,
	pub uniform_color: egui::Color32,
	pub show_coordinates_mod: CoordMod,
	pub square_stroke_color: Option<egui::Color32>,
	pub empty_cell_shape: Option<Shape>,
	pub selected_highlights_shape: Shape,
	pub legal_highlights_shape: Shape,
	pub played_highlights_shape: Shape,
	pub mirrored: bool,
}
impl Default for BoardStyle {
	fn default() -> Self {
		Self {
			clear_color: None,
			half_size_offset_mod: HalfSizeOffsetMod::None,
			checkerboard_mod: CheckerBoardMod::EvenDark,
			light_color: egui::Color32::from_rgb(240, 217, 181),
			dark_color: egui::Color32::from_rgb(181, 136, 99),
			uniform_color: egui::Color32::from_gray(220),
			show_coordinates_mod: CoordMod::FileRankAside,
			square_stroke_color: None,
			empty_cell_shape: None,
			selected_highlights_shape: Shape::StrokeRect { color: egui::Color32::YELLOW, size: 1.0, text: "".into(), text_color: Color32::BLACK, stroke_width: 3.0 },
			legal_highlights_shape: Shape::Circle { color: egui::Color32::from_rgb(50, 50, 150), size: 0.3, text: "".into(), text_color: Color32::BLACK, stroke_color: None },
			played_highlights_shape: Shape::Rect { color: Color32::from_rgba_unmultiplied(150, 150, 250, 80), size: 1.0, text: "".into(), text_color: Color32::BLACK, stroke_color: None },
			mirrored: false,
		}
	}
}
#[derive(Clone, EguiInspect, PartialEq, Serialize, Deserialize)]
pub enum Shape {
	Circle{color: Color32, size: f32, text:String, text_color: Color32, stroke_color: Option<Color32>},
	StrokeCircle{color: Color32, size: f32, stroke_width: f32,  text:String, text_color: Color32},
	Rect{color: Color32, size: f32, text:String, text_color: Color32, stroke_color: Option<Color32>},
	StrokeRect{color: Color32, size: f32, stroke_width: f32,  text:String, text_color: Color32},
	String{text:String, color:Color32}
}
impl Default for Shape {
	fn default() -> Self {
		Shape::Circle { color: Color32::WHITE, text: "".into(), size: 0.7, text_color: Color32::WHITE, stroke_color: None }
	}
}
impl Shape {
	pub fn draw(&self, painter: &egui::Painter, center: Pos2, cell_size: f32) {
		match self {
			Shape::Circle { color, size, text, text_color , stroke_color} => {
				painter.circle_filled(center, cell_size * size/2.0, *color);
				if let Some(c) = stroke_color.as_ref() {
					painter.circle_stroke(center, cell_size * size/2.0, egui::Stroke::new(1.0, *c));
				}
				if !text.is_empty() {
					painter.text(
						center,
						egui::Align2::CENTER_CENTER,
							text,
							egui::FontId::monospace(cell_size* size * 0.5),
							*text_color
						);
				}
			},
			Shape::StrokeCircle { color, size, stroke_width, text, text_color} => {
				painter.circle_stroke(center, cell_size * size/2.0, egui::Stroke::new(*stroke_width, *color));
				if !text.is_empty() {
					painter.text(
						center,
						egui::Align2::CENTER_CENTER,
							text,
							egui::FontId::monospace(cell_size * size * 0.5),
							*text_color
					);
				}
			},
			Shape::String { text, color } => {
				painter.text(
					center,
					egui::Align2::CENTER_CENTER,
						text,
						egui::FontId::proportional(cell_size * 0.8),
						*color
				);
			},
			Shape::Rect { color, size, text, text_color, stroke_color } => {
				let size_vec = Vec2::new(cell_size * size, cell_size * size);
				painter.rect_filled(Rect::from_center_size(center, size_vec), 0.0, *color);
				if let Some(c) = stroke_color.as_ref() {
					painter.rect_stroke(Rect::from_center_size(center, size_vec), 0.0, egui::Stroke::new(1.0, *c), egui::StrokeKind::Inside);
				}
				if !text.is_empty() {
					painter.text(
						center,
						egui::Align2::CENTER_CENTER,
							text,
							egui::FontId::monospace(cell_size * 0.5),
							*text_color
						);
				}
			},
			Shape::StrokeRect { color, size, stroke_width, text, text_color } => {
				painter.rect_stroke(Rect::from_center_size(center, Vec2::new(cell_size * size, cell_size * size)), 0.0, egui::Stroke::new(*stroke_width, *color), egui::StrokeKind::Inside);
				if !text.is_empty() {
					painter.text(
						center,
						egui::Align2::CENTER_CENTER,
							text,
							egui::FontId::monospace(cell_size * 0.5),
							*text_color
					);
				}
			},
		}
	}
}
pub trait EGUIPieceType {
	fn shape(&self) -> Shape {
		Shape::Circle { color: Color32::BLACK, size: 0.7, text: "N/A".to_owned(), text_color: Color32::WHITE, stroke_color: None }
	}
	fn draw(&self, ui: &mut Ui, center: Pos2, cell_size: f32) {
		self.shape().draw(ui.painter(), center, cell_size);
	}
}

pub trait BoardGame : Game<S = Self>+Default+Clone
	where Self::M: BoardMove<Self> + Copy,
{
	type PieceType: Copy+EGUIPieceType;

	fn width(&self) -> u8;
	fn height(&self) -> u8;

	fn legal_moves(&self) -> Vec<Self::M> {
		let mut moves=vec![];
		<Self as Game>::generate_moves(self, &mut moves);
		moves
	}
	fn play(&mut self, mv: Self::M) {
		let state=<Self as Game>::apply(self, mv);
		if let Some(state) = state {
			*self=state;
		}
	}
	fn do_random(&mut self) {
	}
	fn result(&self) -> GameResult;
	fn current_player(&self) -> Player;
	fn nb_players(&self) -> u8 {
		2
	}

	fn get_name(&self, p: Player) -> String {
		p.to_string()
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType>;
	fn index_from_coords(x: u8, y: u8) -> u16;
	fn coords_from_index(index: u16) -> (u8, u8);
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
	fn default_style() -> BoardStyle {
		BoardStyle::default()
	}
	fn play_random(&mut self) {}
}

pub trait BoardMove<G : Game> : std::fmt::Debug + Sized + Copy
{
	fn from(&self) -> Option<u16> {
		None
	}

	fn to(&self) -> u16 {
		0
	}
	fn played_highlights(&self, state: &G) -> Vec<u16> {
		self.click_sequence(state)
	}
	fn handle_clicks_interaction(state: &G, legals: &[G::M], clicks: &Vec<u16>) -> MoveResult<G> 
		where
			G: BoardGame,G::M: BoardMove<G> 
	{
		let candidates: Vec<G::M> = legals.iter()
			.filter(|&&m| m.matches_prefix(state, legals, clicks))
			.copied()
			.collect();

		if candidates.is_empty() { return MoveResult::Invalid; }

		let n = clicks.len();

		// Possible next clicks
		let next_possible_clicks: Vec<u16> = candidates.iter()
			.filter_map(|m| {
				let seq = m.click_sequence(state);
				if seq.len() > n { Some(seq[n]) } else { None }
			})
			.collect();

		if next_possible_clicks.is_empty() {
			let exact_matches: Vec<G::M> = candidates.iter()
				.filter(|m| m.click_sequence(state).len() == n)
				.copied()
				.collect();

			return match exact_matches.len() {
				0 => MoveResult::Invalid,
				1 => MoveResult::Created {
					mv: exact_matches[0],
					highlights_played: clicks.clone(),
				},
				_ => MoveResult::ChoiceRequired {
					candidates: exact_matches,
				},
			};
		}

		MoveResult::Incomplete {
			selected: Some(clicks[n - 1]),
			highlights: next_possible_clicks,
			matching_moves: candidates
			//intermediate_state: Self::compute_intermediate_state(state, legals, clicks),
		}
	}
	/// Some games need to move multiple piece per turn for example, hence, need to display an intermediate state.
	/// This method need to generate such intermediate state after a click
	fn compute_intermediate_state(&self, _state: &G, _clicks: &[u16]) -> Option<G> {
		None
	}
	/// Get the click sequence needed to generate this move
	/// By default, first click is from, second to for a piece move, otherwise, it's "to" for a drop.
	fn click_sequence(&self, _state: &G) -> Vec<u16> {
		if let Some(f) = self.from() {
			vec![f, self.to()]
		} else {
			vec![self.to()]
		}
	}

	/// Check if a sequence a click match the beginning sequence of clicks to generate this move
	fn matches_prefix(&self, state: &G, _legals: &[G::M], clicks: &[u16]) -> bool {
		let seq = self.click_sequence(state);
		clicks.len() <= seq.len() && &seq[..clicks.len()] == clicks
	}
	fn to_uci(&self) -> Option<String> {
		None
	}
	fn from_uci(_m_str: &String) -> Result<Self, String> {
		Err("Not supported".into())
	}
}


pub enum MultipleMoveSelectionResult<M> {
	Pending,
	Cancelled,
	Selected(M)
}