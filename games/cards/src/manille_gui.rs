use crate::game::ManilleMaterialEval;
use kudchuet::gui::CardGame;
use kudchuet::gui::card_view::{CardBoard, CardGameClick, DefaultCardGameDrawer};
use eframe::egui;
use egui::{Color32, Rect};
use kudchuet::ai::{AIEngineProvider, MoveSearcherBuilder};
use kudchuet::cards::playing_cards32::PlayingCard32;
use kudchuet::gui::board_app::GenericGameApp;
use kudchuet::gui::{BoardStyle, CoordMod, GUIGame, GUIMove};

use crate::manille::{Manille, Move};

impl GUIMove<Manille> for Move {
	fn click_sequence(&self, _state: &Manille) -> Vec<<Manille as GUIGame>::Click> {
		if let Move::Play(card) = self {
			return vec![CardGameClick::Card(*card)];
		}
		vec![]
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
	#[allow(refining_impl_trait)]
	fn build_board(
		&self,
	) -> CardBoard<kudchuet::cards::ordered_card_set::OrderedCardSet32, Self::Card> {
		use kudchuet::gui::card_view::{CardSetLayout, CardZone};
		use eframe::egui::pos2;
		use kudchuet::cards::ordered_card_set::OrderedCardSet32;

		let mut zones = Vec::new();

		// Players hands
		for p in 0..4 {
			zones.push(CardZone {
				id: p as u8 + 2,
				set: OrderedCardSet32::from_iter(self.players[p].iter()),
				layout: match p {
					0 => CardSetLayout::Horizontal, // Sud
					1 => CardSetLayout::Vertical,   // Ouest
					2 => CardSetLayout::Horizontal, // Nord
					3 => CardSetLayout::Vertical,   // Est
					_ => unreachable!(),
				},
				rect: match p {
					0 => Rect::from_min_max(pos2(0.15, 0.80), pos2(0.85, 0.98)), // Sud
					1 => Rect::from_min_max(pos2(0.02, 0.15), pos2(0.18, 0.85)), // Ouest
					2 => Rect::from_min_max(pos2(0.15, 0.02), pos2(0.85, 0.20)), // Nord
					3 => Rect::from_min_max(pos2(0.82, 0.15), pos2(0.98, 0.85)), // Est
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
				face_up_predicat: None,
				draw_empty: true,
				zone_only: false,
			});
		}

		// Atout visible
		if self.scores[0] == 0 && self.scores[1] == 0 && self.ply.iter().all(|m| m.is_none()) {
			zones.push(CardZone {
				id: 1,
				set: self.trump_card.into(),
				layout: CardSetLayout::Stack,
				rect: Rect::from_min_max(pos2(0.45, 0.35), pos2(0.55, 0.55)),
				rotation: 0.0,
				card_spacing: 0.0,
				face_up: true,
				face_up_predicat: None,
				draw_empty: false,
				zone_only: false,
			});
		}

		// Pli courant
		let ply = OrderedCardSet32::from_iter(self.ply.iter().flatten().copied());

		zones.push(CardZone {
			id: 0,
			set: ply,
			layout: CardSetLayout::Circle {
				start_angle: -std::f32::consts::FRAC_PI_2,
				len: 4,
			},
			rect: Rect::from_min_max(pos2(0.2, 0.2), pos2(0.8, 0.8)),
			rotation: 0.0,
			card_spacing: 0.0,
			face_up: true,
			face_up_predicat: None,
			draw_empty: false,
			zone_only: false,
		});

		CardBoard { zones }
	}
}
pub fn create_board() -> GenericGameApp<Manille, DefaultCardGameDrawer<Manille, PlayingCard32>> {
	let engines: Vec<Box<dyn AIEngineProvider<Manille>>> = vec![Box::new(
		MoveSearcherBuilder::new("Cheating Material", ManilleMaterialEval, 8),
	)];
	let mut board = GenericGameApp::new(Manille::default(), engines);
	board.game_drawer = Box::new(DefaultCardGameDrawer::default());
	board.max_depth = 13;
	board.depth = 8;
	board
}
