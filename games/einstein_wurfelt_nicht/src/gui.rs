use eframe::egui::{self, Rect};
use egui::{Color32, Stroke, StrokeKind};

use kudchuet::ai::move_search::{UniformRolloutPolicy, MCTS};
use kudchuet::ai::{AIBuilder, AIEngineProvider, MoveSearcherBuilder};
use kudchuet::gui::board_app::GenericBoardApp;
use kudchuet::gui::board_drawer::{DefaultSquareDrawer, SquareDrawer};
use kudchuet::gui::shapes::{Shape, StrokeData, TextData};
use kudchuet::gui::{
	BoardGame, BoardMove, BoardStyle, CheckerBoardMod, CoordMod, EGUIPieceType, GUIGame, GUIMove,
};
use kudchuet::Player;

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
				fill_color: Some(Color32::from_rgb(30, 30, 200)),
				text: Some(TextData {
					text: nb.to_string(),
					size: 0.5,
					color: Color32::BLACK,
				}),
				size: 0.9,
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
			},
			Piece::Player2(nb) => Shape::Circle {
				fill_color: Some(Color32::from_rgb(200, 30, 30)),
				text: Some(TextData {
					text: nb.to_string(),
					size: 0.5,
					color: Color32::WHITE,
				}),
				size: 0.9,
				stroke: Some(StrokeData {
					stroke: Stroke::new(3.0, Color32::BLACK),
					kind: StrokeKind::Inside,
				}),
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
impl GUIMove<EinsteinWurfeltNicht> for MovePlay {
	fn click_sequence(&self, _state: &EinsteinWurfeltNicht) -> Vec<<EinsteinWurfeltNicht as GUIGame>::Click> {
		self.click_sequence_board_move_default(_state)
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

impl GUIGame for EinsteinWurfeltNicht {
	type Click = u16;
	type Settings = kudchuet::gui::DefaultSettings;
	type Style = kudchuet::gui::BoardStyle;
	fn play_random(&mut self) {
		self.roll_dice();
	}
	fn get_name(&self, p: Player) -> String {
		match p {
			Player::PLAYER1 => "Blue",
			Player::PLAYER2 => "Red",
			_ => unreachable!(),
		}
		.into()
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			checkerboard_mod: CheckerBoardMod::OddDark,
			uniform_color: Color32::from_rgb(40, 40, 55),
			dark_color: Color32::from_rgb(73, 97, 38),
			light_color: Color32::from_rgb(110, 150, 100),
			show_coordinates_mod: CoordMod::None,
			square_stroke_color: None,
			..Default::default()
		}
	}
}
impl BoardGame for EinsteinWurfeltNicht {
	type PieceType = Piece;

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
			#[allow(clippy::nonminimal_bool)]
			if x == 0 && y == 5 && self.is_red || x == 4 && y == 5 && !self.is_red {
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
struct MySquareDrawer {
	default: DefaultSquareDrawer,
}
impl MySquareDrawer {
	fn new() -> Self {
		Self {
			default: DefaultSquareDrawer {},
		}
	}
}
impl SquareDrawer<EinsteinWurfeltNicht> for MySquareDrawer {
	fn draw(
		&self,
		painter: &egui::Painter,
		style: &BoardStyle,
		game: &EinsteinWurfeltNicht,
		square: &Rect,
		x_coord: u8,
		y_coord: u8,
	) {
		if y_coord == 5 {
			painter.rect_filled(*square, 0.0, style.uniform_color);
		} else {
			self.default
				.draw(painter, style, game, square, x_coord, y_coord);
		}
	}
}
pub fn create_board() -> GenericBoardApp<EinsteinWurfeltNicht> {
	use kudchuet::gui::board_drawer::BoardDrawer;
	let engines: Vec<Box<dyn AIEngineProvider<EinsteinWurfeltNicht>>> = vec![
		Box::new(MoveSearcherBuilder::new(
			"Dumb",
			EinsteinWurfeltNichtDumbEval,
			20,
		)),
		Box::new(AIBuilder::<
			EinsteinWurfeltNicht,
			MCTS<EinsteinWurfeltNicht, UniformRolloutPolicy<EinsteinWurfeltNicht>>,
		>::new("MCTS")),
	];
	//let ai_provider =
	//	MoveSearcherBuilderDyn::new("Dumb".into(), EinsteinWurfeltNichtDumbEval::default(), 20);
	let mut board = GenericBoardApp::new(EinsteinWurfeltNicht::default(), engines);
	board
		.game_drawer
		.set_square_drawer(Box::new(MySquareDrawer::new()));
	board
}
