use bitboard::Bitboard;
use eframe::egui::{Color32, Stroke, StrokeKind};
use kudchuet::{
	ai::{AIBuilder, AIEngineProvider, move_search::astar::AStar},
	gui::{
		BoardGame, BoardMove, BoardStyle, DefaultSettings, EGUIPieceType, GUIGame,
		game_app::GenericBoardApp, shapes::StrokeData,
	},
};

use crate::{
	bitboard::SolitaireBoard,
	rules::{Move, Solitaire},
};

impl GUIGame for Solitaire {
	type Click = u16;
	type Settings = DefaultSettings;

	type Style = BoardStyle;
	fn nb_players(&self) -> u8 {
		1
	}
	fn default_style() -> Self::Style {
		let mut style = BoardStyle::default();
		style.checkerboard_mod = kudchuet::gui::CheckerBoardMod::None;
		style.show_coordinates_mod = kudchuet::gui::CoordMod::None;
		style.uniform_color = Color32::from_rgb(171,148, 61);
		style.empty_cell_shape = Some(kudchuet::gui::shapes::Shape::Circle {
			fill_color: Some(Color32::from_rgb(25,25,25)),
			size: 0.65,
			text: None,
			stroke: Some(StrokeData {
				stroke: Stroke {
					width: 1.1,
					color: Color32::BLACK,
				},
				kind: StrokeKind::Inside,
			}),
		});
		style.mirrored = true;
		style
	}
}
#[derive(Copy, Clone, Debug)]
pub struct Tile;
impl EGUIPieceType for Tile {
	fn shape(&self) -> kudchuet::gui::shapes::Shape {
		kudchuet::gui::shapes::Shape::Circle {
			fill_color: Some(Color32::from_rgb(30,80,180)),
			size: 0.8,
			text: None,
			stroke: Some(StrokeData {
				stroke: Stroke {
					width: 1.1,
					color: Color32::BLACK,
				},
				kind: StrokeKind::Inside,
			}),
		}
	}
}
impl BoardGame for Solitaire {
	type PieceType = Tile;

	fn width(&self) -> u8 {
		7
	}

	fn height(&self) -> u8 {
		7
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		if self.board.get(x, y) {
			Some(Tile)
		} else {
			None
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		let index = SolitaireBoard::index_from_coords(x, y) as u16;
		if [0, 1, 5, 6, 7, 8, 12, 13, 35, 36, 40, 41, 42, 43, 47, 48].contains(&index) {
			return u16::MAX;
		}
		index
	}

	fn coords_from_index(index: u16) -> (u8, u8) {
		SolitaireBoard::coords_from_index(index as usize)
	}
}
impl BoardMove<Solitaire> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.from as u16)
	}
	fn to(&self) -> u16 {
		self.to as u16
	}
}
pub fn create_board() -> GenericBoardApp<Solitaire> {
	let engines: Vec<Box<dyn AIEngineProvider<Solitaire>>> = vec![Box::new(AIBuilder::<
		Solitaire,
		AStar<Solitaire>,
	>::new("A*"))];
	let mut board = GenericBoardApp::new(Solitaire::new(), engines);
	board.max_depth = 13;
	board.depth = 8;
	board
}
