use egui::Color32;
use kudchuet::{ai::{AIBuilder, AIEngineProvider, move_search::astar::AStar}, gui::{BoardGame, BoardMove, BoardStyle, DefaultSettings, EGUIPieceType, GUIGame, board_app::GenericBoardApp, shapes::StrokeData}};

use crate::rules::{Move, Taquin};


impl<const W: usize, const H: usize, const NB: usize> GUIGame for Taquin<W, H, NB> {
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
		style.mirrored = true;
		style
	}
}
#[derive(Copy, Clone, Debug)]
pub struct Tile(u8);
impl EGUIPieceType for Tile {
	fn shape(&self) -> kudchuet::gui::shapes::Shape {
		if self.0 == 0 {
			kudchuet::gui::shapes::Shape::Rect {
				fill_color: Some(Color32::BLACK),
				size: 1.0,
				text: None,
				stroke: None,
			}
		} else {
			kudchuet::gui::shapes::Shape::Rect {
				fill_color: Some(Color32::LIGHT_BLUE),
				size: 1.0,
				text: Some(kudchuet::gui::shapes::TextData {
					text: self.0.to_string(),
					color: Color32::BLACK,
					size: 0.8,
				}),
				stroke: Some(StrokeData {
					stroke: egui::Stroke {
						width: 1.0,
						color: Color32::BLACK,
					},
					kind: egui::StrokeKind::Inside,
				}),
			}
		}
	}
}
impl<const W: usize, const H: usize, const NB: usize> BoardGame for Taquin<W, H, NB> {
	type PieceType = Tile;

	fn width(&self) -> u8 {
		W as u8
	}

	fn height(&self) -> u8 {
		H as u8
	}

	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		Some(Tile(self.content[x as usize][y as usize]))
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Self::index_from_coords(x as usize, y as usize) as u16
	}

	fn coords_from_index(index: u16) -> (u8, u8) {
		let coords = Self::coords_from_index(index as u8);
		(coords.0 as u8, coords.1 as u8)
	}

}
impl<const W: usize, const H: usize, const NB: usize> BoardMove<Taquin<W, H, NB>> for Move {
	fn to(&self) -> u16 {
		self.from as u16
	}
}

pub fn create_board<const W: usize, const H: usize, const NB: usize>()
-> GenericBoardApp<Taquin<W, H, NB>> {
	let engines: Vec<Box<dyn AIEngineProvider<Taquin<W, H, NB>>>> = vec![Box::new(AIBuilder::<
		Taquin<W, H, NB>,
		AStar<Taquin<W, H, NB>>,
	>::new("A*"))];
	let mut board = GenericBoardApp::new(Taquin::<W, H, NB>::new_random(), engines);
	board.max_depth = 13;
	board.depth = 8;
	board
}