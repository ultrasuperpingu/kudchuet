use crate::game::ManilleMaterialEval;
use crate::gui::card_app::CardApp;
use crate::gui::{CardGame, CardMove};
use crate::playing_cards32::PlayingCard32;
use eframe::egui;
use egui::Color32;
use kudchuet::ai::{AIEngineProvider, MoveSearcherBuilder};
use kudchuet::gui::{BoardStyle, CoordMod, GUIGame};

use crate::manille::{Manille, Move};

impl CardMove<Manille> for Move {
	fn card(&self) -> Option<PlayingCard32> {
		if let Move::Play(card) = self {
			return Some(*card);
		}
		None
	}
}
impl GUIGame for Manille {
	type Settings = kudchuet::gui::DefaultSettings;
	type Style = kudchuet::gui::BoardStyle;
	fn nb_players(&self) -> u8 {
		4
	}
	fn get_name(&self, p: kudchuet::Player) -> String {
		if p.0 == 0 {
			"Team 1 (S)".into()
		} else if p.0 == 1 {
			"Team 2 (W)".into()
		} else if p.0 == 3 {
			"Team 1 (N)".into()
		} else {
			"Team 2 (E)".into()
		}
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

	fn revealed_cards(&self) -> crate::unordered_card_sets32::UnorderedCardSet32 {
		self.trump_card.into()
	}

	fn draw_revealed_cards(&self) -> bool {
		self.scores[0] == 0 && self.scores[1] == 0 && self.ply.iter().all(|m| m.is_none())
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
