use kudchuet::egui::{Rect, pos2};
use kudchuet::{
	ai::{AIEngineProvider, MoveSearcherBuilder},
	cards::{ordered_card_set::OrderedCardSet54, playing_cards54::PlayingCard54},
	gui::{
		CardGame, GUIGame, GUIMove,
		card_game_drawer::{CardBoard, CardGameClick, CardSetLayout, CardZone, DefaultCardGameDrawer},
		game_app::GameApp,
	},
};

use crate::{
	game::FreecellEval,
	rules::{Freecell, Move},
};

impl CardGame for Freecell {
	type Card = PlayingCard54;

	#[allow(refining_impl_trait)]
	fn build_board(&self) -> CardBoard<OrderedCardSet54, PlayingCard54> {
		let mut zones = vec![];

		let x_step = 0.08;
		let base_y_top = 0.05;
		//let base_y_mid = 0.15;
		let base_y_bottom = 0.4;

		for (i, col) in self.foundations.iter().enumerate() {
			zones.push(CardZone {
				id: i as u8 + 4,
				set: OrderedCardSet54::from_iter(*col),
				layout: CardSetLayout::Stack,
				//origin: pos2(0.60 + i as f32 * x_step, base_y_top),
				rect: Rect::from_min_max(
					pos2(0.60 + i as f32 * x_step, base_y_top),
					pos2(0.60 + (i + 1) as f32 * x_step, 0.35),
				),
				rotation: 0.0,
				card_spacing: 0.0,
				face_up: true,
				face_up_predicat: None,
				draw_empty: true,
				zone_only: true,
			});
		}

		for (i, cell) in self.free_cells.iter().enumerate() {
			zones.push(CardZone {
				id: i as u8,
				set: OrderedCardSet54::from_iter(*cell),
				layout: CardSetLayout::Stack,
				//origin: pos2(0.05 + i as f32 * x_step, base_y_top),
				rect: Rect::from_min_max(
					pos2(0.05 + i as f32 * x_step, base_y_top),
					pos2(0.05 + (i + 1) as f32 * x_step, 0.35),
				),
				rotation: 0.0,
				card_spacing: 0.03,
				face_up: true,
				face_up_predicat: None,
				draw_empty: true,
				zone_only: true,
			});
		}

		let len = self.tableau.len();
		let column_width = 0.94 / len as f32;
		for (i, col) in self.tableau.iter().enumerate() {
			zones.push(CardZone {
				id: i as u8 + 8,
				set: col.clone(),
				layout: CardSetLayout::Vertical,
				//origin: pos2(0.05 + i as f32 * 0.08, base_y_bottom),
				rect: Rect::from_min_max(
					pos2(0.03 + i as f32 * column_width, base_y_bottom),
					pos2(0.03 + (i + 1) as f32 * column_width, 1.0),
				),
				rotation: 0.0,
				card_spacing: 25.0,
				face_up: true,
				face_up_predicat: None,
				draw_empty: true,
				zone_only: false,
			});
		}
		CardBoard { zones }
	}
}

impl GUIMove<Freecell> for Move {
	fn click_sequence(&self, state: &Freecell) -> Vec<<Freecell as GUIGame>::Click> {
		let mut res = vec![];
		match self {
			Move::TableauToTableau { from, to, count } => {
				let len = state.tableau[*from].len();
				if len < *count {
					println!("error on {} with move: {:?}", state, self);
				}
				let card_from = state.tableau[*from].iter().nth(len - *count).unwrap();
				let to = if let Some(card) = state.tableau[*to].iter().last() {
					CardGameClick::Card(*card)
				} else {
					CardGameClick::CardZone(*to as u8 + 8)
				};
				res.push(CardGameClick::Card(*card_from));
				res.push(to);
			}
			Move::TableauToFreeCell { from, cell } => {
				let card_from = state.tableau[*from].iter().last().unwrap();
				res.push(CardGameClick::Card(*card_from));
				res.push(CardGameClick::CardZone(*cell as u8));
			}
			Move::FreeCellToTableau { cell, to } => {
				let to = if let Some(card) = state.tableau[*to].iter().last() {
					CardGameClick::Card(*card)
				} else {
					CardGameClick::CardZone(*to as u8 + 8)
				};
				res.push(CardGameClick::CardZone(*cell as u8));
				res.push(to);
			}
			Move::TableauToFoundation { from, foundation } => {
				if state.tableau[*from].iter().last().is_none() {
					println!("error on {} with move: {:?}", state, self);
				}
				let card_from = state.tableau[*from].iter().last().unwrap();
				res.push(CardGameClick::Card(*card_from));
				res.push(CardGameClick::CardZone(*foundation as u8 + 4));
			}
			Move::FreeCellToFoundation { cell, foundation } => {
				res.push(CardGameClick::CardZone(*cell as u8));
				res.push(CardGameClick::CardZone(*foundation as u8 + 4));
			}
			Move::Finish => {}
		}
		res
	}
}

pub fn create_board() -> GameApp<Freecell, DefaultCardGameDrawer<Freecell, PlayingCard54>> {
	let engines: Vec<Box<dyn AIEngineProvider<Freecell>>> = vec![Box::new(
		MoveSearcherBuilder::new("Simple", FreecellEval, 5),
	)];
	let mut board = GameApp::new(Freecell::new(), engines);
	board.game_drawer = Box::new(DefaultCardGameDrawer::default());
	board.max_depth = 13;
	board.depth = 8;
	board
}
