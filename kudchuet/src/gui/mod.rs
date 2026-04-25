
use std::fmt::Debug;

use egui::{Color32, Stroke};
use egui_field_editor::EguiInspect;
use crate::ai::minimax::interface::Game;
use serde::{Deserialize, Serialize};
use crate::gui::shapes::StrokeData;
use crate::gui::{input_handler::MoveResult, shapes::Shape};


use super::{GameOutcome, Player};
use self::options_panel::RightTab;

pub mod board_app;
pub mod board_drawer;
pub mod options_panel;
pub mod game_state_manager;
pub mod input_handler;
pub mod shapes;

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
#[derive(EguiInspect, Debug, Default, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum CheckerBoardMod {
	None,
	#[default]
	EvenDark,
	OddDark
}

#[derive(EguiInspect, Debug, Default, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum RowOffsetPattern {
	#[default]
	NoOffset,
	EvenRowsShifted,
	OddRowsShifted
}
#[derive(EguiInspect, Clone, Debug, Serialize, Deserialize)]
pub struct BoardStyle {
	pub checkerboard_mod: CheckerBoardMod,
	pub row_offset_pattern: RowOffsetPattern,
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
			row_offset_pattern: RowOffsetPattern::NoOffset,
			checkerboard_mod: CheckerBoardMod::EvenDark,
			light_color: egui::Color32::from_rgb(240, 217, 181),
			dark_color: egui::Color32::from_rgb(181, 136, 99),
			uniform_color: egui::Color32::from_gray(220),
			show_coordinates_mod: CoordMod::FileRankAside,
			square_stroke_color: None,
			empty_cell_shape: None,
			selected_highlights_shape: Shape::Rect {
				fill_color: None,
				size: 1.0,
				text: None,
				stroke: Some(StrokeData { stroke: Stroke::new( 3.0, egui::Color32::YELLOW), kind: egui::StrokeKind::Inside})
			},
			legal_highlights_shape: Shape::Circle {
				fill_color: Some(egui::Color32::from_rgb(50, 50, 150)),
				size: 0.3,
				text: None,
				stroke: None
			},
			played_highlights_shape: Shape::Rect {
				fill_color: Some(Color32::from_rgba_unmultiplied(150, 150, 250, 80)),
				size: 1.0,
				text: None,
				stroke: None
			},
			mirrored: false,
		}
	}
}
pub trait EGUIPieceType {
	fn shape(&self) -> Shape;
}
#[derive(Default, Debug, EguiInspect)]
pub struct DefaultSettings;
pub trait BoardGame : Game<S = Self>+Default+Clone
	where Self::M: BoardMove<Self> + Copy,
{
	type PieceType: Copy+EGUIPieceType;
	type Settings: Default+EguiInspect;
	/// Create a Board game from the provided settings.
	/// 
	/// Default implementation ignores settings; override if game requires configuration.
	fn build_from_settings(_settings: &Self::Settings) -> Self {
		Self::default()
	}
	/// Width of the board
	fn width(&self) -> u8;
	/// Height of the board
	fn height(&self) -> u8;

	/// Generate legal moves from the current position.
	/// 
	/// This is a convinience method. Default implementation call Game::generate_moves.
	/// Collects all generated moves into a Vec.
	/// Prefer using generate_moves for performance-critical paths.
	/// This does not need to be reimplemented.
	fn legal_moves(&self) -> Vec<Self::M> {
		let mut moves=vec![];
		<Self as Game>::generate_moves(self, &mut moves);
		moves
	}

	/// Apply a move to the game.
	/// 
	/// This is a convinient method. Default implementation call Game::apply(self).
	/// This does not need to be reimplemented.
	fn play(&mut self, mv: Self::M) {
		let state=<Self as Game>::apply(self, mv);
		if let Some(state) = state {
			*self=state;
		}
	}
	/// Get the current status of the game.
	/// 
	/// This is a convinient method. Default implementation call Game::get_winner(self) and convert it to GameResult.
	/// This does not need to be reimplemented.
	fn result(&self) -> GameOutcome {
		Self::get_winner(self).into()
	}
	/// Get the player which have to play
	/// 
	/// This is a convinient method. Default implementation call Game::current_player(self).
	/// This does not need to be reimplemented.
	fn current_player(&self) -> Player {
		<Self as Game>::get_current_player(self)
	}
	/// Number of players (default is 2)
	fn nb_players(&self) -> u8 {
		2
	}
	/// Should returns Player's name (ex: White, Black).
	/// Default is Player n.
	fn get_name(&self, p: Player) -> String {
		p.to_string()
	}
	/// Returns the piece at a particular square on the board
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType>;
	/// 
	fn index_from_coords(x: u8, y: u8) -> u16;
	fn coords_from_index(index: u16) -> (u8, u8);
	fn position_to_string(&self) -> Option<String> {
		None
	}
	fn game_to_string(&self, _mvs: &[Self::M]) -> Option<String> {
		None
	}
	fn game_from_string(&self, _game_str: &str) -> Result<Vec<Self::M>, String> {
		Err("Not Supported".into())
	}
	fn get_position_from_string(&self, _pos_str: &str) -> Result<Self, String> {
		Err("Not Supported".into())
	}
	fn move_from_string(&self, m_str: &str) -> Result<Self::M, String> {
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

pub trait BoardMove<G : Game> : Debug + Sized + Copy
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
	fn handle_clicks_interaction(state: &G, legals: &[G::M], clicks: &[u16]) -> MoveResult<G> 
		where
			G: BoardGame,
			G::M: BoardMove<G> 
	{
		let candidates: Vec<G::M> = legals.iter()
			.filter(|&&m| m.matches_prefix(state, legals, clicks))
			.copied()
			.collect();

		if candidates.is_empty() { return MoveResult::Invalid; }

		let n = clicks.len();

		// Possible next clicks
		let mut next_possible_clicks = Vec::new();

		for m in &candidates {
			let seq = m.click_sequence(state);
			if seq.len() > n {
				let idx = seq[n];
				if !next_possible_clicks.contains(&idx) {
					next_possible_clicks.push(idx);
				}
			}
		}

		if next_possible_clicks.is_empty() {
			let exact_matches: Vec<G::M> = candidates.iter()
				.filter(|m| m.click_sequence(state).len() == n)
				.copied()
				.collect();

			return match exact_matches.len() {
				0 => MoveResult::Invalid,
				1 => MoveResult::Created {
					mv: exact_matches[0],
					highlights_played: clicks.to_owned(),
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
	fn from_uci(_m_str: &str) -> Result<Self, String> {
		Err("Not supported".into())
	}
}


pub enum MultipleMoveSelectionResult<M> {
	Pending,
	Cancelled,
	Selected(M)
}