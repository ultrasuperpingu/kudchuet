use eframe::egui;
use egui::Color32;
use crate::common::gui::board_app::GenericBoardApp;
use crate::common::gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType, Shape};
use crate::common::new_move_searcher_vec;
use crate::neutron::game::NeutronDumbEval;

use super::*;

impl Player {
	pub fn piece(&self) -> Piece {
		match self {
			Player::Player1 => Piece::White,
			Player::Player2 => Piece::Black,
			_ => unreachable!(),
		}
	}
}

impl BoardMove<Neutron> for Move {
	fn from(&self) -> Option<u16> { Some(self.pawn.0 as u16) }
	fn to(&self) -> u16 { self.pawn.1 as u16 }

	fn click_sequence(&self, state: &Neutron) -> Vec<u16> {
		match self.neutron {
			None => vec![self.pawn.0 as u16, self.pawn.1 as u16],
			Some(n_to) => {
				vec![state.get_neutron_index() as u16 , n_to as u16, self.pawn.0 as u16, self.pawn.1 as u16] 
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
			Piece::Neutron =>  Shape::Circle{color:Color32::RED, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
			Piece::White => Shape::Circle{color:Color32::WHITE, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
			Piece::Black => Shape::Circle{color:Color32::BLACK, size: 0.7, text: "".into(), text_color: Color32::BLACK, stroke_color: Some(Color32::BLACK)},
		}
	}
}
impl BoardGame for Neutron {
	type PieceType=Piece;

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
		let mut style = BoardStyle::default();
		style.dark_color=egui::Color32::from_rgb(181, 136, 99);
		style.light_color=Color32::from_rgb(240, 217, 181);
		style.show_coordinates_mod=crate::common::gui::CoordMod::FileRankOnSquare;
		style
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
	fn get_position_from_string(&self, pos_str: &String) -> Result<Self, String> {
		Self::from_fen(pos_str)
	}
}

pub fn create_board() -> GenericBoardApp<Neutron> {
	let board=GenericBoardApp::new(Neutron::default(), new_move_searcher_vec("Dumb".into(), NeutronDumbEval{}, 5));
	board
}
