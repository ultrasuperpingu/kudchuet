use crate::bitboard::Bitboard5x5;
use crate::rules::{Move, Neutron, Piece};
use bitboard::Bitboard;
use eframe::egui;
use egui::Color32;
use kudchuet::gui::board_app::GenericBoardApp;
use kudchuet::gui::shapes::{Shape, StrokeData};
use kudchuet::gui::{BoardGame, BoardMove, BoardStyle, CoordMod, EGUIPieceType};
use kudchuet::{GameResult, Player, new_move_searcher_vec};

use super::game::NeutronDumbEval;

impl BoardMove<Neutron> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.pawn.0 as u16)
	}
	fn to(&self) -> u16 {
		self.pawn.1 as u16
	}

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
impl BoardGame for Neutron {
	type PieceType = Piece;

	fn width(&self) -> u8 {
		5
	}

	fn height(&self) -> u8 {
		5
	}

	fn legal_moves(&self) -> Vec<Self::M> {
		self.legal_moves()
	}
	fn play(&mut self, mv: Self::M) {
		self.play(&mv);
	}

	fn result(&self) -> GameResult {
		self.result()
	}

	fn current_player(&self) -> Player {
		self.turn()
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
	fn default_style() -> BoardStyle {
		BoardStyle {
			dark_color: egui::Color32::from_rgb(181, 136, 99),
			light_color: Color32::from_rgb(240, 217, 181),
			show_coordinates_mod: CoordMod::FileRankOnSquare,
			..Default::default()
		}
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
	fn get_position_from_string(&self, pos_str: &String) -> Result<Self, String> {
		Self::from_fen(pos_str)
	}
}

pub fn create_board() -> GenericBoardApp<Neutron> {
	let board = GenericBoardApp::new(
		Neutron::default(),
		new_move_searcher_vec("Dumb".into(), NeutronDumbEval {}, 5),
	);
	board
}
