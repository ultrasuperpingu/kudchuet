use bitboard::Bitboard;
use eframe::egui;
use egui::Color32;
use crate::common::bitboards::Bitboard7x7;
use crate::common::{Player, new_move_searcher_vec};
use crate::common::gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType};
use crate::common::gui::shapes::{Shape, StrokeData, TextData};

use crate::common::gui::board_app::GenericBoardApp;
use crate::diaballik::Action;

use super::game::DiaballikEvalMaterial;

use super::{Cell, Move, Diaballik};
impl BoardMove<Diaballik> for Move {

	fn click_sequence(&self, _state: &Diaballik) -> Vec<u16> {
		let mut seq = Vec::new();
		for action in self.0.iter().flatten() {
			match action {
				Action::Move { from, to } | Action::Pass { from, to } => {
					seq.push(*from as u16);
					seq.push(*to as u16);
				}
			}
		}
		seq
	}
	fn compute_intermediate_state(&self, state: &Diaballik, clicks: &[u16]) -> Option<Diaballik> {
		if clicks.len() < 2 { return None; }

		let mut sim = state.clone();
		let player = state.current_player();
		let mut i = 0;

		while i + 1 < clicks.len() {
			let from = clicks[i] as u8;
			let to = clicks[i+1] as u8;

			if player == Player::PLAYER1 {
				if from == sim.ball_player1 {
					sim.ball_player1 = to;
				} 
				else if sim.player1.get_at_index(from as usize) {
					sim.player1.reset_at_index(from as usize);
					sim.player1.set_at_index(to as usize);
					
					/*if from == sim.ball_player1 {
						sim.ball_player1 = to;
					}*/
				}
			} else {
				if from == sim.ball_player2 {
					sim.ball_player2 = to;
				} else if sim.player2.get_at_index(from as usize) {
					sim.player2.reset_at_index(from as usize);
					sim.player2.set_at_index(to as usize);
					
					/*if from == sim.ball_player2 {
						sim.ball_player2 = to;
					}*/
				}
			}
			i += 2;
		}

		Some(sim)
	}
}
impl EGUIPieceType for Cell {
	fn shape(&self) -> Shape {
		match self {
			Cell::Empty => unreachable!(),
			Cell::White => Shape::Circle { fill_color: Some(Color32::WHITE), size: 0.7, text: None, stroke: Some(StrokeData::default()) },
			Cell::Black => Shape::Circle { fill_color: Some(Color32::BLACK), size: 0.7, text: None, stroke: Some(StrokeData::default()) },
			Cell::WhiteWithBall => Shape::Circle {
				fill_color: Some(Color32::YELLOW),
				size: 0.7,
				text: Some(TextData {
					text: "⚽".into(),
					color: Color32::BLACK,
					size: 0.5,
				}),
				stroke: Some(StrokeData::default())
			},
			Cell::BlackWithBall => Shape::Circle {
				fill_color: Some(Color32::YELLOW),
				size: 0.7,
				text: Some(TextData {
					text: "⚽".into(),
					color: Color32::WHITE,
					size: 0.5,
				}),
				stroke: Some(StrokeData::default())
			}
		}
	}
}

impl BoardGame for Diaballik {
	type PieceType=Cell;

	fn width(&self) -> u8 {
		7
	}

	fn height(&self) -> u8 {
		7
	}

	fn legal_moves(&self) -> Vec<Self::M> {
		let mut moves=vec![];
		self.legal_moves(&mut moves);
		moves
	}
	fn play(&mut self, mv: Self::M) {
		self.play_unchecked(&mv);
	}

	fn result(&self) -> crate::common::GameResult {
		self.result()
	}

	fn current_player(&self) -> crate::common::Player {
		self.turn()
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		match self.cell(x, y) {
			Cell::Empty => None,
			e => Some(e),
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard7x7::index_from_coords(x, y) as u16
	}
	fn coords_from_index(i: u16) -> (u8, u8) {
		Bitboard7x7::coords_from_index(i as usize)
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: crate::common::gui::CheckerBoardMod::None,
			uniform_color: Color32::LIGHT_BLUE,
			square_stroke_color: Some(Color32::BLUE),
			show_coordinates_mod: crate::common::gui::CoordMod::NumbersOnSquare,
			..Default::default()
		}
	}
}

pub fn create_board() -> GenericBoardApp<Diaballik> {
	let mut board=GenericBoardApp::new(Diaballik::default(), new_move_searcher_vec("Material".into(), DiaballikEvalMaterial{}, 3));
	board.depth = 3;
	board.max_depth = 4;
	board
}