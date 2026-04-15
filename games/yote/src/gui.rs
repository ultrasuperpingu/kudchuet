use bitboard::Bitboard;
use eframe::egui;
use egui::Color32;

use kudchuet::gui::input_handler::MoveResult;
use kudchuet::gui::shapes::Shape;
use kudchuet::{
	GameResult, Player,
	gui::{BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType},
	new_move_searcher_vec,
};

use crate::{bitboard::Bitboard6x5, rules::YotePlayer};
use kudchuet::gui::board_app::GenericBoardApp;

use super::game::YoteMaterialEval;
use super::rules::{Move, Yote};

impl BoardMove<Yote> for Move {
	fn handle_clicks_interaction(
		_state: &Yote,
		legals: &[Move],
		clicks: &[u16],
	) -> MoveResult<Yote> {
		if clicks.len() == 1 {
			let cindex = clicks[0] as u8;
			let filtered = legals.iter().filter(|m| match m {
				Move::Add { index } => *index == cindex,
				Move::Move { from, to: _ } => *from == cindex,
				Move::Take {
					from,
					to: _,
					supplement_pawn: _,
				} => *from == cindex,
			});
			let mut highlights = vec![];
			for m in filtered.clone() {
				match m {
					Move::Add { index: _ } => {
						return MoveResult::Created {
							mv: *m,
							highlights_played: vec![cindex as u16],
						};
					}
					Move::Move { from: _, to } => highlights.push(*to as u16),
					Move::Take {
						from: _,
						to,
						supplement_pawn: _,
					} => highlights.push(*to as u16),
				};
			}
			if !highlights.is_empty() {
				return MoveResult::Incomplete {
					selected: None,
					highlights,
					matching_moves: filtered.copied().collect(), //intermediate_state: None
				};
			}
		} else if clicks.len() == 2 {
			let from_index = clicks[0] as u8;
			let to_index = clicks[1] as u8;
			let filtered = legals.iter().filter(|m| match m {
				Move::Add { index: _ } => false,
				Move::Move { from, to } => *from == from_index && to_index == *to,
				Move::Take {
					from,
					to,
					supplement_pawn: _,
				} => *from == from_index && to_index == *to,
			});
			let mut highlights = vec![];
			let mut no_supplement_capture = None;
			for m in filtered.clone() {
				match m {
					Move::Add { index: _ } => unreachable!(),
					Move::Move { from: _, to: _ } => {
						return MoveResult::Created {
							mv: *m,
							highlights_played: vec![from_index as u16, to_index as u16],
						};
					}
					Move::Take {
						from: _,
						to: _,
						supplement_pawn,
					} => {
						if let Some(supplement_pawn) = supplement_pawn {
							highlights.push(*supplement_pawn as u16)
						} else {
							no_supplement_capture = Some(m);
						}
					}
				};
			}
			if !highlights.is_empty() {
				return MoveResult::Incomplete {
					selected: None,
					highlights,
					matching_moves: filtered.copied().collect(), //intermediate_state: None
				};
			} else if let Some(no_supplement_capture) = no_supplement_capture {
				return MoveResult::Created {
					mv: *no_supplement_capture,
					highlights_played: vec![from_index as u16, to_index as u16],
				};
			}
		} else if clicks.len() == 3 {
			let from_index = clicks[0] as u8;
			let to_index = clicks[1] as u8;
			let supp_index = clicks[2] as u8;
			let filtered = legals.iter().filter(|m| match m {
				Move::Add { index: _ } => false,
				Move::Move { from: _, to: _ } => false,
				Move::Take {
					from,
					to,
					supplement_pawn: _,
				} => *from == from_index && to_index == *to,
			});
			for m in filtered {
				match m {
					Move::Add { index: _ } => unreachable!(),
					Move::Move { from: _, to: _ } => unreachable!(),
					Move::Take {
						from,
						to,
						supplement_pawn: Some(supp),
					} => {
						if *supp == supp_index {
							return MoveResult::Created {
								mv: *m,
								highlights_played: vec![*from as u16, *to as u16],
							};
						}
					}
					_ => {}
				};
			}
		}
		MoveResult::Invalid
	}
}
impl EGUIPieceType for YotePlayer {
	fn shape(&self) -> Shape {
		match self {
			YotePlayer::Player1 => Shape::Circle {
				fill_color: Some(Color32::WHITE),
				size: 0.7,
				text: None,
				stroke: None,
			},
			YotePlayer::Player2 => Shape::Circle {
				fill_color: Some(Color32::BLACK),
				size: 0.7,
				text: None,
				stroke: None,
			},
		}
	}
}

impl BoardGame for Yote {
	type PieceType = YotePlayer;
	type Settings = kudchuet::gui::DefaultSettings;

	fn width(&self) -> u8 {
		6
	}

	fn height(&self) -> u8 {
		5
	}

	fn legal_moves(&self) -> Vec<Self::M> {
		let mut moves = vec![];
		self.legal_moves_inplace(&mut moves);
		moves
	}

	fn play(&mut self, mv: Self::M) {
		self.play(mv);
	}

	fn result(&self) -> GameResult {
		self.result()
	}

	fn current_player(&self) -> Player {
		self.turn
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		if self.white.get(x, y) {
			Some(YotePlayer::Player1)
		} else if self.black.get(x, y) {
			Some(YotePlayer::Player2)
		} else {
			None
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard6x5::index_from_coords(x, y) as u16
	}

	fn coords_from_index(index: u16) -> (u8, u8) {
		Bitboard6x5::coords_from_index(index as usize)
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: CheckerBoardMod::None,
			uniform_color: Color32::from_rgb(235, 230, 220),
			show_coordinates_mod: CoordMod::FileRankAside,
			square_stroke_color: Some(egui::Color32::BLACK),
			..Default::default()
		}
	}
}

pub fn create_board() -> GenericBoardApp<Yote> {
	let board = GenericBoardApp::new(
		Yote::default(),
		new_move_searcher_vec("Material".into(), YoteMaterialEval {}, 5),
	);
	board
}
