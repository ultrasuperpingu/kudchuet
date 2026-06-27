use kudchuet::egui;
use egui::Color32;
use kudchuet::ai::move_search::{MCTS, UniformRolloutPolicy};
use kudchuet::ai::{AIBuilder, AIEngineProvider, MoveSearcherBuilder};
use kudchuet::gui::{GUIGame, GUIMove};

use crate::bitboard::Bitboard5x5;
use crate::game::TeekoEvalDumb;
use crate::rules::{Move, Teeko};
use kudchuet::gui::shapes::Shape;
use kudchuet::{
	Player,
	gui::{BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType},
};

use kudchuet::gui::game_app::GenericBoardApp;
//use super::{game::ThreeMusketeersEvalAdvance};

impl GUIMove<Teeko> for Move {
	fn click_sequence(&self, _state: &Teeko) -> Vec<<Teeko as GUIGame>::Click> {
		self.click_sequence_board_move_default(_state)
	}
}

impl BoardMove<Teeko> for Move {
	fn from(&self) -> Option<u16> {
		self.from.map(|m| m as u16)
	}

	fn to(&self) -> u16 {
		self.to as u16
	}

	fn to_uci(&self) -> Option<String> {
		None
	}

	fn from_uci(_m_str: &str) -> Result<Self, String> {
		Err("Not supported".into())
	}
}
#[derive(Copy, Clone)]
pub enum TeekoPiece {
	Black,
	Red,
}
impl EGUIPieceType for TeekoPiece {
	fn shape(&self) -> Shape {
		match self {
			TeekoPiece::Black => Shape::Circle {
				fill_color: Some(Color32::from_rgb(20, 20, 30)),
				size: 0.7,
				text: None,
				stroke: None,
			},
			TeekoPiece::Red => Shape::Circle {
				fill_color: Some(Color32::from_rgb(200, 50, 50)),
				size: 0.7,
				text: None,
				stroke: None,
			},
		}
	}
}

impl GUIGame for Teeko {
	type Click = u16;
	type Settings = kudchuet::gui::DefaultSettings;
	type Style = kudchuet::gui::BoardStyle;
	fn get_name(&self, p: Player) -> String {
		match p {
			Player::PLAYER1 => "Black".into(),
			Player::PLAYER2 => "Red".into(),
			_ => unreachable!(),
		}
	}

	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
	fn get_position_from_string(&self, pos_str: &str) -> Result<Self, String> {
		Self::from_fen(pos_str)
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
}
impl BoardGame for Teeko {
	type PieceType = TeekoPiece;

	fn width(&self) -> u8 {
		5
	}

	fn height(&self) -> u8 {
		5
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		match self.get_cell(x, y) {
			Some(Player::PLAYER1) => Some(TeekoPiece::Black),
			Some(Player::PLAYER2) => Some(TeekoPiece::Red),
			_ => None,
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard5x5::index_from_coords(x, y) as u16
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		Bitboard5x5::coords_from_index(index as usize)
	}
}

pub fn create_board() -> GenericBoardApp<Teeko> {
	let engines: Vec<Box<dyn AIEngineProvider<Teeko>>> = vec![
		Box::new(MoveSearcherBuilder::new("Dumb", TeekoEvalDumb::new(), 4)),
		Box::new(AIBuilder::<Teeko, MCTS<Teeko, UniformRolloutPolicy<Teeko>>>::new("MCTS")),
	];
	GenericBoardApp::new(Teeko::default(), engines)
}
