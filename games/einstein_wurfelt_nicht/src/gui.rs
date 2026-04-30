use eframe::egui;
use egui::{Color32, Stroke, StrokeKind};

use kudchuet::ai::MoveSearcherBuilderDyn;
use kudchuet::gui::board_app::GenericBoardApp;
use kudchuet::gui::shapes::{Shape, StrokeData, TextData};
use kudchuet::gui::{BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType};

use crate::game::EinsteinWurfeltNichtDumbEval;

use super::rules::{EinsteinWurfeltNicht, MovePlay};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
	Player1(u8),
	Player2(u8),
	Dice(u8),
}

impl EGUIPieceType for Piece {
	fn shape(&self) -> Shape {
		match self {
			Piece::Player1(nb) => Shape::Circle {
				fill_color: Some(Color32::BLUE),
				text: Some(TextData {
					text: nb.to_string(),
					size: 0.5,
					color: Color32::BLACK,
				}),
				size: 0.98,
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
			},
			Piece::Player2(nb) => Shape::Circle {
				fill_color: Some(Color32::RED),
				text: Some(TextData {
					text: nb.to_string(),
					size: 0.5,
					color: Color32::WHITE,
				}),
				size: 0.98,
				stroke: None,
			},
			Piece::Dice(d) => Shape::Rect {
				fill_color: Some(Color32::WHITE),
				size: 1.0,
				text: Some(TextData {
					text: dice_string(*d),
					color: Color32::BLACK,
					size: 0.6,
				}),
				stroke: Some(StrokeData {
					stroke: Stroke {
						width: 3.0,
						color: Color32::BLACK,
					},
					kind: StrokeKind::Inside,
				}),
			},
		}
	}
}
impl BoardMove<EinsteinWurfeltNicht> for MovePlay {
	fn to(&self) -> u16 {
		match self {
			MovePlay::Dice(_) => 0,
			MovePlay::Move(m) => m.to as u16,
		}
	}
	fn from(&self) -> Option<u16> {
		match self {
			MovePlay::Dice(_) => None,
			MovePlay::Move(m) => Some(m.from as u16),
		}
	}
}

impl BoardGame for EinsteinWurfeltNicht {
	type PieceType = Piece;
	type Settings = kudchuet::gui::DefaultSettings;

	fn width(&self) -> u8 {
		5
	}

	fn height(&self) -> u8 {
		6
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		if y < 5 {
			for (i, e) in self.red.iter().enumerate() {
				if *e as u16 == Self::index_from_coords(x, y) {
					return Some(Piece::Player2(i as u8 + 1));
				}
			}
			for (i, e) in self.blue.iter().enumerate() {
				if *e as u16 == Self::index_from_coords(x, y) {
					return Some(Piece::Player1(i as u8 + 1));
				}
			}
		}
		if let Some(d) = self.dice {
			if x == 0 && y == 5 {
				return Some(Piece::Dice(d));
			}
		}
		None
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		x as u16 + y as u16 * 5
	}

	fn coords_from_index(index: u16) -> (u8, u8) {
		(index as u8 % 5, index as u8 / 5)
	}
	fn play_random(&mut self) {
		self.roll_dice();
	}

	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: CheckerBoardMod::OddDark,
			uniform_color: Color32::from_rgb(40, 70, 125),
			dark_color: Color32::from_rgb(60, 45, 30),
			light_color: Color32::from_rgb(200, 175, 140),
			show_coordinates_mod: CoordMod::None,
			square_stroke_color: None,
			..Default::default()
		}
	}
}
fn dice_string(dice: u8) -> String {
	match dice {
		1 => "🎲1".into(), //⚀
		2 => "🎲2".into(), //⚁
		3 => "🎲3".into(), //⚂
		4 => "🎲4".into(), //⚃
		5 => "🎲5".into(), //⚄
		6 => "🎲6".into(), //⚅
		_ => dice.to_string(),
	}
}

pub fn create_board() -> GenericBoardApp<EinsteinWurfeltNicht> {
	let ai_provider =
		MoveSearcherBuilderDyn::new("Dumb".into(), EinsteinWurfeltNichtDumbEval::default(), 4);
	let board = GenericBoardApp::new(EinsteinWurfeltNicht::default(), vec![Box::new(ai_provider)]);
	board
}
