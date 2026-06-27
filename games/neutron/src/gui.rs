use crate::bitboard::Bitboard5x5;
use crate::game::NeutronMaterialEval;
use crate::rules::{Move, Neutron, Piece};
use bitboard::Bitboard;
use kudchuet::egui;
use egui::Color32;
use kudchuet::Player;
use kudchuet::ai::{AIEngineProvider, MoveSearcherBuilder};
use kudchuet::gui::game_app::GenericBoardApp;
use kudchuet::gui::shapes::{Shape, StrokeData};
use kudchuet::gui::{BoardGame, BoardMove, BoardStyle, CoordMod, EGUIPieceType, GUIGame, GUIMove};

use super::game::NeutronDumbEval;

impl GUIMove<Neutron> for Move {
	fn click_sequence(&self, state: &Neutron) -> Vec<u16> {
		match self.neutron {
			None => vec![self.pawn.0 as u16, self.pawn.1 as u16],
			Some(n_to) => {
				vec![
					state.get_neutron_index() as u16,
					n_to as u16,
					self.pawn.0 as u16,
					self.pawn.1 as u16,
				]
			}
		}
	}
	fn compute_intermediate_state(&self, state: &Neutron, clicks: &[u16]) -> Option<Neutron> {
		if clicks.len() >= 2 && !state.is_first_move() {
			let mut tmp = state.clone();
			let neutron_to = clicks[1];

			tmp.neutron.reset_at_index(state.get_neutron_index());
			tmp.neutron.set_at_index(neutron_to as usize);

			return Some(tmp);
		}
		None
	}
}
impl BoardMove<Neutron> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.pawn.0 as u16)
	}
	fn to(&self) -> u16 {
		self.pawn.1 as u16
	}
}
impl EGUIPieceType for Piece {
	fn shape(&self) -> Shape {
		match self {
			Piece::Neutron => Shape::Circle {
				fill_color: Some(Color32::RED),
				size: 0.7,
				text: None,
				stroke: Some(StrokeData::default()),
			},
			Piece::White => Shape::Circle {
				fill_color: Some(Color32::WHITE),
				size: 0.7,
				text: None,
				stroke: Some(StrokeData::default()),
			},
			Piece::Black => Shape::Circle {
				fill_color: Some(Color32::BLACK),
				size: 0.7,
				text: None,
				stroke: None,
			},
		}
	}
}
impl GUIGame for Neutron {
	type Click = u16;
	type Settings = kudchuet::gui::DefaultSettings;
	type Style = kudchuet::gui::BoardStyle;
	fn get_name(&self, p: Player) -> String {
		match p {
			Player::PLAYER1 => "Black".into(),
			Player::PLAYER2 => "White".into(),
			_ => unreachable!(),
		}
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
	fn get_position_from_string(&self, pos_str: &str) -> Result<Self, String> {
		Self::from_fen(pos_str)
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			dark_color: egui::Color32::from_rgb(181, 136, 99),
			light_color: Color32::from_rgb(240, 217, 181),
			show_coordinates_mod: CoordMod::FileRankOnSquare,
			..Default::default()
		}
	}
}
impl BoardGame for Neutron {
	type PieceType = Piece;

	fn width(&self) -> u8 {
		5
	}

	fn height(&self) -> u8 {
		5
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		self.piece_at(x, y)
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard5x5::index_from_coords(x, y) as u16
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		Bitboard5x5::coords_from_index(index as usize)
	}
}

pub fn create_board() -> GenericBoardApp<Neutron> {
	let engines: Vec<Box<dyn AIEngineProvider<Neutron>>> = vec![
		Box::new(MoveSearcherBuilder::new("Dumb", NeutronDumbEval::new(), 4)),
		Box::new(MoveSearcherBuilder::new(
			"Material",
			NeutronMaterialEval::new(),
			4,
		)),
	];
	GenericBoardApp::new(Neutron::default(), engines)
}
