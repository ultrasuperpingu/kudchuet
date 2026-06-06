use egui::Color32;
use kudchuet::{
	Player,
	ai::{AIBuilder, AIEngineProvider, move_search::{Game, astar::{AStar, Heuristic}}},
	gui::{
		BoardGame, BoardMove, BoardStyle, DefaultSettings, EGUIPieceType, GUIGame, GUIMove, board_app::GenericBoardApp, shapes::StrokeData
	},
};

use crate::{
	rules::{Move, Taquin},
};

impl<const W: usize, const H: usize, const NB: usize> GUIMove<Taquin<W, H, NB>> for Move {
	fn click_sequence(&self, _state: &Taquin<W, H, NB>) -> Vec<<Taquin<W, H, NB> as GUIGame>::Click> {
		self.click_sequence_board_move_default(_state)
	}
}
impl<const W: usize, const H: usize, const NB: usize> Game for Taquin<W, H, NB> {
	type S = Self;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> kudchuet::GameOutcome {
		*moves = state.legal_moves();
		Self::get_outcome(state)
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s = state.clone();
		s.play_unchecked(m);
		Some(s)
	}

	fn get_outcome(state: &Self::S) -> kudchuet::GameOutcome {
		if *state == Self::SOLVED {
			kudchuet::GameOutcome::PLAYER1
		} else {
			kudchuet::GameOutcome::OnGoing
		}
	}

	fn get_current_player(_state: &Self::S) -> kudchuet::Player {
		Player::PLAYER1
	}
	fn get_next_player(_state: &Self::S) -> Player {
		Player::PLAYER1
	}
	fn get_hash(state: &Self::S) -> u64 {
		//state.get_hash()
		state.compute_hash()
	}

}
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
impl<const W: usize, const H: usize, const NB: usize> Heuristic for Taquin<W, H, NB> {
	type G = Taquin<W, H, NB>;

	fn heuristic(&self, state: &<Self::G as Game>::S) -> u32 {
		state.manhattan_with_linear_conflict() as u32
	}
}
pub struct TaquinSettings {
	pub width: u8,
	pub height: u8,
}
