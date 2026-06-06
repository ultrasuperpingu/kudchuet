use crate::game::ManilleMaterialEval;
use crate::gui::card_app::CardApp;
use crate::gui::card_view::{CardGameClick, CardZone};
use crate::gui::{CardGame, CardMove};
use crate::playing_cards32::PlayingCard32;
use eframe::egui;
use egui::Color32;
use kudchuet::ai::{AIEngineProvider, MoveSearcherBuilder};
use kudchuet::gui::{BoardStyle, CoordMod, GUIGame};

use crate::manille::{Manille, Move};

impl CardMove<Manille> for Move {
	fn click(&self) -> Option<CardGameClick<PlayingCard32>> {
		if let Move::Play(card) = self {
			return Some(CardGameClick::Card(*card));
		}
		None
	}
}
impl GUIGame for Manille {
	type Click = CardGameClick<PlayingCard32>;
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
	type Card = PlayingCard32;
	/*fn player_ply_card(&self, p: kudchuet::Player) -> Option<PlayingCard32> {
		self.ply[p.0 as usize]
	}

	fn player_hand_cards(
		&self,
		p: kudchuet::Player,
	) -> crate::unordered_card_set::UnorderedCardSet32 {
		self.players[p.0 as usize]
	}

	fn revealed_cards(&self) -> crate::unordered_card_set::UnorderedCardSet32 {
		self.trump_card.into()
	}

	fn draw_revealed_cards(&self) -> bool {
		self.scores[0] == 0 && self.scores[1] == 0 && self.ply.iter().all(|m| m.is_none())
	}*/

	fn build_board(&self) -> Vec<CardZone<crate::ordered_card_set::OrderedCardSet32, Self::Card>> {
		use crate::gui::card_view::{CardSetLayout, CardZone};
		use crate::ordered_card_set::OrderedCardSet32;
		use eframe::egui::pos2;

		let mut zones = Vec::new();

		// Mains des joueurs
		for p in 0..4 {
			zones.push(CardZone {
				set: OrderedCardSet32::from_iter(self.players[p].iter()),
				layout: match p {
					0 => CardSetLayout::Horizontal, // Sud
					1 => CardSetLayout::Vertical, // Ouest
					2 => CardSetLayout::Horizontal, // Nord
					3 => CardSetLayout::Vertical, // Est
					_ => unreachable!(),
				},
				origin: match p {
					0 => pos2(0.50, 0.85), // Sud
					1 => pos2(0.10, 0.50), // Ouest
					2 => pos2(0.50, 0.10), // Nord
					3 => pos2(0.90, 0.50), // Est
					_ => unreachable!(),
				},
				rotation: match p {
					0 => 0.0,
					1 => std::f32::consts::FRAC_PI_2,
					2 => std::f32::consts::PI,
					3 => -std::f32::consts::FRAC_PI_2,
					_ => 0.0,
				},
				card_spacing: 25.0,
				face_up: p == 0,
			});
		}

		// Atout visible
		if self.scores[0] == 0 && self.scores[1] == 0 && self.ply.iter().all(|m| m.is_none()) {
			zones.push(CardZone {
				set: self.trump_card.into(),
				layout: CardSetLayout::Stack,
				origin: pos2(0.50, 0.35),
				rotation: 0.0,
				card_spacing: 0.0,
				face_up: true,
			});
		}

		// Pli courant
		let ply = OrderedCardSet32::from_iter(self.ply.iter().flatten().copied());

		zones.push(CardZone {
			set: ply,
			layout: CardSetLayout::Circle {
				start_angle: -std::f32::consts::FRAC_PI_2,
			},
			origin: pos2(0.50, 0.50),
			rotation: 0.0,
			card_spacing: 0.0,
			face_up: true,
		});

		zones
	}
}
pub fn create_board() -> CardApp<Manille> {
	let engines: Vec<Box<dyn AIEngineProvider<Manille>>> = vec![Box::new(
		MoveSearcherBuilder::new("Cheating Material", ManilleMaterialEval, 8),
	)];
	let mut board = CardApp::new(Manille::default(), engines);
	//board.board_drawer.get_style_mut().dark_color=egui::Color32::from_rgb(181, 136, 99);
	//board.board_drawer.get_style_mut().light_color=Color32::from_rgb(240, 217, 181);
	//board.board_drawer.get_style_mut().show_coordinates_mod=crate::common::gui::CoordMod::NumbersAside;
	board.max_depth = 13;
	board.depth = 8;
	board
}
