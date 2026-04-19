use eframe::egui;
use egui::Color32;
use kudchuet::ai::minimax::Game;

use crate::bitboard::Bitboard5x5;
use crate::game::ThreeMusketeersEvalSimple;
use crate::rules::{Move, ThreeMusketeers};
use kudchuet::gui::shapes::Shape;
use kudchuet::{
	Player,
	gui::{
		BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType,
		shapes::TextData,
	},
	new_move_searcher_vec,
};

use kudchuet::gui::board_app::GenericBoardApp;
//use super::{game::ThreeMusketeersEvalAdvance};

impl BoardMove<ThreeMusketeers> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.from as u16)
	}

	fn to(&self) -> u16 {
		self.to as u16
	}

	fn played_highlights(&self, state: &ThreeMusketeers) -> Vec<u16> {
		self.click_sequence(state)
	}

	fn handle_clicks_interaction(
		state: &ThreeMusketeers,
		legals: &[<ThreeMusketeers as Game>::M],
		clicks: &[u16],
	) -> kudchuet::gui::input_handler::MoveResult<ThreeMusketeers>
	where
		ThreeMusketeers: BoardGame,
		<ThreeMusketeers as Game>::M: BoardMove<ThreeMusketeers>,
	{
		let candidates: Vec<<ThreeMusketeers as Game>::M> = legals
			.iter()
			.filter(|&&m| m.matches_prefix(state, legals, clicks))
			.copied()
			.collect();

		if candidates.is_empty() {
			return kudchuet::gui::input_handler::MoveResult::Invalid;
		}

		let n = clicks.len();

		// Possible next clicks
		let next_possible_clicks: Vec<u16> = candidates
			.iter()
			.filter_map(|m| {
				let seq = m.click_sequence(state);
				if seq.len() > n { Some(seq[n]) } else { None }
			})
			.collect();

		if next_possible_clicks.is_empty() {
			let exact_matches: Vec<<ThreeMusketeers as Game>::M> = candidates
				.iter()
				.filter(|m| m.click_sequence(state).len() == n)
				.copied()
				.collect();

			return match exact_matches.len() {
				0 => kudchuet::gui::input_handler::MoveResult::Invalid,
				1 => kudchuet::gui::input_handler::MoveResult::Created {
					mv: exact_matches[0],
					highlights_played: clicks.to_owned(),
				},
				_ => kudchuet::gui::input_handler::MoveResult::ChoiceRequired {
					candidates: exact_matches,
				},
			};
		}

		kudchuet::gui::input_handler::MoveResult::Incomplete {
			selected: Some(clicks[n - 1]),
			highlights: next_possible_clicks,
			matching_moves: candidates, //intermediate_state: Self::compute_intermediate_state(state, legals, clicks),
		}
	}

	fn compute_intermediate_state(
		&self,
		_state: &ThreeMusketeers,
		_clicks: &[u16],
	) -> Option<ThreeMusketeers> {
		None
	}

	fn click_sequence(&self, _state: &ThreeMusketeers) -> Vec<u16> {
		if let Some(f) = self.from() {
			vec![f, self.to()]
		} else {
			vec![self.to()]
		}
	}

	fn matches_prefix(
		&self,
		state: &ThreeMusketeers,
		_legals: &[<ThreeMusketeers as Game>::M],
		clicks: &[u16],
	) -> bool {
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
#[derive(Copy, Clone)]
pub enum ThreeMPiece {
	Musketeer,
	Guard,
}
impl EGUIPieceType for ThreeMPiece {
	fn shape(&self) -> Shape {
		match self {
			//✠    ✥♱♰✢✣
			ThreeMPiece::Musketeer => Shape::Circle {
				fill_color: Some(Color32::from_rgb(50, 100, 200)),
				size: 0.7,
				text: Some(TextData {
					text: "⚜".into(),
					color: Color32::WHITE,
					size: 0.5,
				}),
				stroke: None,
			},
			//✚
			ThreeMPiece::Guard => Shape::Circle {
				fill_color: Some(Color32::from_rgb(200, 50, 50)),
				size: 0.7,
				text: Some(TextData {
					text: "✝".into(),
					color: Color32::WHITE,
					size: 0.5,
				}),
				stroke: None,
			},
		}
	}
}

impl BoardGame for ThreeMusketeers {
	type PieceType = ThreeMPiece;
	type Settings = kudchuet::gui::DefaultSettings;

	fn width(&self) -> u8 {
		5
	}

	fn height(&self) -> u8 {
		5
	}

	fn get_name(&self, p: Player) -> String {
		match p {
			Player::PLAYER1 => "Musketeers".into(),
			Player::PLAYER2 => "Guards".into(),
			_ => unreachable!(),
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		match self.get_cell(x, y) {
			Some(Player::PLAYER1) => Some(ThreeMPiece::Musketeer),
			Some(Player::PLAYER2) => Some(ThreeMPiece::Guard),
			_ => None,
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
			checkerboard_mod: CheckerBoardMod::EvenDark,
			//uniform_color: Color32::from_rgb(235, 230, 220),
			light_color: Color32::from_rgb(200, 200, 250),
			dark_color: Color32::from_rgb(40, 40, 100),
			show_coordinates_mod: CoordMod::FileRankOnSquare,
			played_highlights_shape: Shape::Rect {
				fill_color: Some(Color32::from_rgba_unmultiplied(200, 200, 90, 120)),
				size: 1.0,
				text: None,
				stroke: None,
			},
			//square_stroke_color: Some(egui::Color32::BLACK),
			..Default::default()
		}
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
	fn get_position_from_string(&self, pos_str: &str) -> Result<Self, String> {
		Self::from_fen(pos_str)
	}

	fn nb_players(&self) -> u8 {
		2
	}

	fn game_to_string(&self, _mvs: &[Self::M]) -> Option<String> {
		None
	}

	fn game_from_string(&self, _game_str: &str) -> Result<Vec<Self::M>, String> {
		Err("Not Supported".into())
	}

	fn move_from_string(&self, m_str: &str) -> Result<Self::M, String> {
		Self::M::from_uci(m_str)
	}

	fn move_to_string(&self, m: &Self::M) -> Option<String> {
		Self::M::to_uci(m)
	}

	fn play_random(&mut self) {}
}

pub fn create_board() -> GenericBoardApp<ThreeMusketeers> {
	let board = GenericBoardApp::new(
		ThreeMusketeers::default(),
		new_move_searcher_vec("Simple".into(), ThreeMusketeersEvalSimple::new(), 5),
	);
	board
}
