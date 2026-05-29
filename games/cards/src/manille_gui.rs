use crate::game::ManilleMaterialEval;
use crate::gui::card_app::CardApp;
use crate::gui::{CardGame, CardMove};
use crate::playing_cards32::PlayingCard32;
use eframe::egui;
use egui::Color32;
use kudchuet::ai::{AIEngineProvider, MoveSearcherBuilder};
use kudchuet::gui::{BoardStyle, CoordMod, GUIGame};

use crate::manille::{Manille, Move};

impl CardMove<Manille> for Move {}
impl GUIGame for Manille {
	type Settings = kudchuet::gui::DefaultSettings;
	type Style = kudchuet::gui::BoardStyle;
	fn nb_players(&self) -> u8 {
		4
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			dark_color: egui::Color32::from_rgb(181, 136, 99),
			light_color: Color32::from_rgb(240, 217, 181),
			show_coordinates_mod: CoordMod::NumbersAside,
			..Default::default()
		}
	}
}
impl CardGame for Manille {
	fn player_ply_card(&self, p: kudchuet::Player) -> Option<PlayingCard32> {
		self.ply[p.0 as usize]
	}

	fn player_hand_cards(
		&self,
		p: kudchuet::Player,
	) -> crate::unordered_card_sets32::UnorderedCardSet32 {
		self.players[p.0 as usize]
	}
}
pub fn create_board() -> CardApp<Manille> {
	let engines: Vec<Box<dyn AIEngineProvider<Manille>>> = vec![Box::new(
		MoveSearcherBuilder::new("Material", ManilleMaterialEval, 8),
	)];
	let mut board = CardApp::new(Manille::default(), engines);
	//board.board_drawer.get_style_mut().dark_color=egui::Color32::from_rgb(181, 136, 99);
	//board.board_drawer.get_style_mut().light_color=Color32::from_rgb(240, 217, 181);
	//board.board_drawer.get_style_mut().show_coordinates_mod=crate::common::gui::CoordMod::NumbersAside;
	board.max_depth = 13;
	board.depth = 8;
	board
}
