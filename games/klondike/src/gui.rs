use crate::game::KlondikeEval;
use crate::rules::{Klondike, Move};
use kudchuet::egui::{Rect, pos2};
use kudchuet::{
	ai::{AIEngineProvider, MoveSearcherBuilder},
	cards::{ordered_card_set::OrderedCardSet54, playing_cards54::PlayingCard54},
	gui::{
		BoardStyle, CardGame, DefaultSettings, GUIGame, GUIMove,
		card_game_drawer::{CardBoard, CardGameClick, CardSetLayout, CardZone, DefaultCardGameDrawer},
		game_app::GameApp,
	},
};

impl GUIGame for Klondike {
	type Click = CardGameClick<PlayingCard54>;
	type Settings = DefaultSettings;

	type Style = BoardStyle;
	fn nb_players(&self) -> u8 {
		1
	}
}
impl GUIMove<Klondike> for Move {
	fn click_sequence(&self, state: &Klondike) -> Vec<<Klondike as GUIGame>::Click> {
		let mut res = vec![];

		match self {
			Move::TableauToTableau { from, to, count } => {
				let from_col = &state.tableau[*from];
				let to_col = &state.tableau[*to];

				if let Some(card_from) = from_col.0.iter().rev().nth(count - 1) {
					let target = if let Some(card) = to_col.0.iter().last() {
						CardGameClick::Card(*card)
					} else {
						CardGameClick::CardZone(*to as u8 + 6)
					};

					res.push(CardGameClick::Card(*card_from));
					res.push(target);
				}
			}

			Move::TableauToFoundation { from, foundation } => {
				let from_col = &state.tableau[*from];

				if let Some(card_from) = from_col.0.iter().last() {
					res.push(CardGameClick::Card(*card_from));
					res.push(CardGameClick::CardZone(*foundation as u8 + 2));
				}
			}

			Move::RevealToFoundation { foundation } => {
				if let Some(card) = state.revealed_stock.iter().last() {
					res.push(CardGameClick::Card(*card));
					res.push(CardGameClick::CardZone(*foundation as u8 + 2));
				}
			}

			Move::StockToReveal => {
				res.push(CardGameClick::CardZone(0));
			}

			Move::RecycleStock => {
				res.push(CardGameClick::CardZone(0));
			}

			Move::RevealToTableau { to } => {
				if let Some(card) = state.revealed_stock.iter().last() {
					res.push(CardGameClick::Card(*card));
					let to_col = &state.tableau[*to];

					if let Some(card) = to_col.0.iter().last() {
						res.push(CardGameClick::Card(*card));
					} else {
						res.push(CardGameClick::CardZone(*to as u8 + 6));
					}
				}
			}
		}

		res
	}
}
impl CardGame for Klondike {
	type Card = PlayingCard54;
	#[allow(refining_impl_trait)]
	fn build_board(&self) -> CardBoard<OrderedCardSet54, PlayingCard54> {
		let mut zones = vec![];

		zones.push(CardZone {
			id: 0,
			set: self.stock.clone(),
			layout: CardSetLayout::Stack,
			rect: Rect::from_min_max(pos2(0.05, 0.05), pos2(0.2, 0.35)),
			rotation: 0.0,
			card_spacing: 0.0,
			face_up: false,
			face_up_predicat: None,
			draw_empty: true,
			zone_only: true,
		});

		zones.push(CardZone {
			id: 1,
			set: self.revealed_stock.clone(),
			layout: CardSetLayout::Stack,
			rect: Rect::from_min_max(pos2(0.2, 0.05), pos2(0.35, 0.35)),
			rotation: 0.0,
			card_spacing: 30.0,
			face_up: true,
			face_up_predicat: None,
			draw_empty: true,
			zone_only: false,
		});
		let len = self.foundations.len();
		let table_width = 0.45 / len as f32;
		for (i, col) in self.foundations.iter().enumerate() {
			zones.push(CardZone {
				id: i as u8 + 2,
				set: OrderedCardSet54::from_iter(*col),
				layout: CardSetLayout::Stack,
				rect: Rect::from_min_max(
					pos2(0.5 + i as f32 * table_width, 0.05),
					pos2(0.5 + (i + 1) as f32 * table_width, 0.35),
				),
				rotation: 0.0,
				card_spacing: 20.0,
				face_up: true,
				face_up_predicat: None,
				draw_empty: true,
				zone_only: true,
			});
		}
		let len = self.tableau.len();
		let table_width = 0.94 / len as f32;
		for (i, col) in self.tableau.iter().enumerate() {
			let face_up_index = col.1;
			zones.push(CardZone {
				id: i as u8 + 6,
				set: col.0.clone(),
				layout: CardSetLayout::Vertical,
				rect: Rect::from_min_max(
					pos2(0.03 + i as f32 * table_width, 0.40),
					pos2(0.03 + (i + 1) as f32 * table_width, 1.0),
				),
				rotation: 0.0,
				card_spacing: 20.0,
				face_up: true,
				face_up_predicat: Some(Box::new(move |_z, index| index >= face_up_index as usize)),
				draw_empty: true,
				zone_only: false,
			});
		}

		CardBoard { zones }
	}
}

pub fn create_board() -> GameApp<Klondike, DefaultCardGameDrawer<Klondike, PlayingCard54>> {
	let engines: Vec<Box<dyn AIEngineProvider<Klondike>>> = vec![Box::new(
		MoveSearcherBuilder::new("Simple", KlondikeEval, 5),
	)];
	let mut board = GameApp::new(Klondike::new(), engines);
	board.game_drawer = Box::new(DefaultCardGameDrawer::default());
	board.max_depth = 13;
	board.depth = 8;
	board
	/*use winit::platform::windows::EventLoopBuilderExtWindows;
	let engines: Vec<Box<dyn AIEngineProvider<Klondike>>> = vec![Box::new(
		MoveSearcherBuilder::new("Cheating AI", KlondikeEval, 8),
	)];
	let mut board = GameApp::new(Klondike::new(), engines);
	board.game_drawer = Box::new(DefaultCardGameDrawer::default());
	board.max_depth = 13;
	board.depth = 8;
	let mut options = eframe::NativeOptions::default();
	options.event_loop_builder = Some(Box::new(|builder| {
		builder.with_any_thread(true);
	}));
	eframe::run_native("Klondike", options, Box::new(|_cc| Ok(Box::new(board))))*/
}
