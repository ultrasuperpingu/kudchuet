use bitboard::Bitboard;
use eframe::egui;
use egui::{Color32, Stroke, StrokeKind};

use kudchuet::ai::move_search::{MCTS, UniformRolloutPolicy};
use kudchuet::ai::{AIBuilder, AIEngineProvider, MoveSearcherBuilder};
use kudchuet::gui::board_app::GenericBoardApp;
use kudchuet::gui::shapes::{Shape, StrokeData};
use kudchuet::gui::{BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType};
use kudchuet::Player;

use crate::game::{ClobberDumbEval, ClobberSimpleEval};
use crate::rules::Bitboard5x6;

use super::rules::{Clobber, Move};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
	Player1,
	Player2,
}

impl EGUIPieceType for Piece {
	fn shape(&self) -> Shape {
		match self {
			Piece::Player1 => Shape::Circle {
				fill_color: Some(Color32::WHITE),
				text: None,
				size: 0.7,
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
			},
			Piece::Player2 => Shape::Circle {
				fill_color: Some(Color32::BLACK),
				text: None,
				size: 0.7,
				stroke: None,
			},
		}
	}
}
impl BoardMove<Clobber> for Move {
	fn to(&self) -> u16 {
		self.to as u16
	}
	fn from(&self) -> Option<u16> {
		Some(self.from as u16)
	}
}

impl BoardGame for Clobber {
	type PieceType = Piece;
	type Settings = kudchuet::gui::DefaultSettings;

	fn width(&self) -> u8 {
		5
	}

	fn height(&self) -> u8 {
		6
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		if self.white.get(x, y) {
			Some(Piece::Player1)
		} else if self.black.get(x, y) {
			Some(Piece::Player2)
		} else {
			None
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard5x6::index_from_coords(x, y) as u16
	}

	fn coords_from_index(index: u16) -> (u8, u8) {
		Bitboard5x6::coords_from_index(index as usize)
	}
	fn get_name(&self, p: Player) -> String {
		match p {
			Player::PLAYER1 => "White",
			Player::PLAYER2 => "Black",
			_ => unreachable!(),
		}
		.into()
	}

	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: CheckerBoardMod::OddDark,
			uniform_color: Color32::from_rgb(40, 70, 125),
			dark_color: Color32::from_rgb(60, 45, 30),
			light_color: Color32::from_rgb(200, 175, 140),
			show_coordinates_mod: CoordMod::FileRankOnSquare,
			square_stroke_color: None,
			..Default::default()
		}
	}
}

pub fn create_board() -> GenericBoardApp<Clobber> {
	let engines: Vec<Box<dyn AIEngineProvider<Clobber>>> = vec![
		Box::new(MoveSearcherBuilder::new("Dumb", ClobberDumbEval, 8)),
		Box::new(MoveSearcherBuilder::new("Simple", ClobberSimpleEval, 8)),
		Box::new(AIBuilder::<Clobber, MCTS<Clobber, UniformRolloutPolicy<Clobber>>>::new("MCTS")),
	];
	//let ai_provider = MoveSearcherBuilderDyn::new("Dumb".into(), ClobberDumbEval::default(), 8);
	GenericBoardApp::new(Clobber::default(), engines)
}
