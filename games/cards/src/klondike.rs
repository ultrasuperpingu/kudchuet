use std::hash::{DefaultHasher, Hash, Hasher};

use egui::{Rect, pos2};
use kudchuet::{
	Player,
	ai::move_search::Game,
	gui::{BoardStyle, DefaultSettings, GUIGame, GUIMove},
};

use crate::{
	freecell::{are_alternate, foundation_index, rank},
	gui::{
		CardGame, CardMove,
		card_view::{CardBoard, CardGameClick, CardSetLayout, CardZone},
	},
	ordered_card_set::OrderedCardSet54,
	playing_cards::{CardSet, PlayingCard},
	playing_cards54::PlayingCard54,
	unordered_card_set::UnorderedCardSet54,
};
#[derive(Debug, Clone, Hash)]
pub struct Klondike {
	pub stock: OrderedCardSet54,
	pub revealed_stock: OrderedCardSet54,
	pub tableau: [OrderedCardSet54; 7],
	pub foundations: [UnorderedCardSet54; 4],
	h: u64,
}
impl Default for Klondike {
	fn default() -> Self {
		Self::new()
	}
}
#[derive(Copy, Clone, Debug)]
pub enum Move {
	StockToReveal,
	RecycleStock,

	RevealToTableau { to: usize },
	RevealToFoundation { foundation: usize },

	TableauToTableau { from: usize, to: usize },
	TableauToFoundation { from: usize, foundation: usize },
}
impl Klondike {
	pub fn new() -> Self {
		let mut deck = UnorderedCardSet54::ALL_BUT_JOKERS;

		let mut s = Self {
			stock: OrderedCardSet54::EMPTY,
			revealed_stock: OrderedCardSet54::EMPTY,
			tableau: [
				CardSet::draw_random(1, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(2, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(3, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(4, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(5, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(6, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(7, &mut deck)
					.expect("Should work")
					.into(),
			],
			foundations: [UnorderedCardSet54::EMPTY; 4],
			h: 0,
		};
		for t in &mut s.tableau {
			fastrand::shuffle(&mut t.0);
		}
		s.stock = deck.into();
		fastrand::shuffle(&mut s.stock.0);
		s.h = s.compute_hash();
		s
	}
	pub fn legal_moves(&self) -> Vec<Move> {
		let mut moves = Vec::new();

		// STOCK -> REVEAL / recycle
		if self.stock.iter().last().is_some() {
			moves.push(Move::StockToReveal);
		} else if !self.revealed_stock.is_empty() {
			moves.push(Move::RecycleStock);
		}

		// REVEAL
		if let Some(&card) = self.revealed_stock.iter().last() {
			let f = foundation_index(card.color());

			if can_place_on_foundation(card, self.foundations[f]) {
				moves.push(Move::RevealToFoundation { foundation: f });
			}

			for (to, col) in self.tableau.iter().enumerate() {
				if can_place_on_tableau(card, col) {
					moves.push(Move::RevealToTableau { to });
				}
			}
		}

		// TABLEAU
		for (from, col) in self.tableau.iter().enumerate() {
			let Some(&card) = col.iter().last() else {
				continue;
			};

			let f = foundation_index(card.color());

			if can_place_on_foundation(card, self.foundations[f]) {
				moves.push(Move::TableauToFoundation {
					from,
					foundation: f,
				});
			}

			// tableau -> tableau
			for (to, dst) in self.tableau.iter().enumerate() {
				if from != to && can_place_on_tableau(card, dst) {
					moves.push(Move::TableauToTableau { from, to });
				}
			}

			// KING -> EMPTY
			if rank(card) == 13 {
				for (to, dst) in self.tableau.iter().enumerate() {
					if dst.is_empty() {
						moves.push(Move::TableauToTableau { from, to });
					}
				}
			}
		}

		moves
	}
	pub fn play_unchecked(&mut self, m: Move) {
		match m {
			Move::StockToReveal => {
				let card = self.stock.pop().unwrap();
				let from = self.stock.len();
				let to = 10 + self.revealed_stock.len();

				self.revealed_stock.push(card);

				self.h ^= Self::ZOBRIST.stock[from][card.index() as usize];
				self.h ^= Self::ZOBRIST.stock[to][card.index() as usize];
			}

			Move::RecycleStock => {
				while let Some(card) = self.revealed_stock.pop() {
					self.stock.push(card);
				}
				self.h = self.compute_hash();
			}

			Move::RevealToTableau { to } => {
				let card = self.revealed_stock.pop().unwrap();

				let from = 10 + self.revealed_stock.len();
				let to_idx = self.tableau[to].len();

				self.tableau[to].push(card);

				self.h ^= Self::ZOBRIST.stock[from][card.index() as usize];
				self.h ^= Self::ZOBRIST.tableau[to][to_idx][card.index() as usize];
			}

			Move::RevealToFoundation { foundation } => {
				let card = self.revealed_stock.pop().unwrap();

				let from = 10 + self.revealed_stock.len();

				self.foundations[foundation].insert(card);

				self.h ^= Self::ZOBRIST.stock[from][card.index() as usize];
				self.update_foundation_hash(foundation, card, true);
			}

			Move::TableauToTableau { from, to } => {
				let card = self.tableau[from].pop().unwrap();

				let from_i = self.tableau[from].len();
				let to_i = self.tableau[to].len();

				self.tableau[to].push(card);

				self.h ^= Self::ZOBRIST.tableau[from][from_i][card.index() as usize];
				self.h ^= Self::ZOBRIST.tableau[to][to_i][card.index() as usize];
			}

			Move::TableauToFoundation { from, foundation } => {
				let card = self.tableau[from].pop().unwrap();

				let from_i = self.tableau[from].len();

				self.h ^= Self::ZOBRIST.tableau[from][from_i][card.index() as usize];
				self.update_foundation_hash(foundation, card, true);
			}
		}
	}
}
fn can_place_on_tableau(card: PlayingCard54, col: &OrderedCardSet54) -> bool {
	col.iter()
		.last()
		.map(|top| are_alternate(card.color(), top.color()) && rank(card) + 1 == rank(*top))
		.unwrap_or(true)
}

fn can_place_on_foundation(card: PlayingCard54, f: UnorderedCardSet54) -> bool {
	rank(card) == f.len() as u8 + 1
}
pub struct ZobristKlondike {
	pub stock: [[u64; 52]; 24],
	pub tableau: [[[u64; 52]; 20]; 7],
	pub foundations: [[u64; 14]; 4],
}
impl ZobristKlondike {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);

		let mut stock = [[0; 52]; 24];
		let mut tableau = [[[0; 52]; 20]; 7];
		let mut foundations = [[0; 14]; 4];

		let mut i = 0;
		while i < 24 {
			let mut c = 0;
			while c < 52 {
				stock[i][c] = rng.u64();
				c += 1;
			}
			i += 1;
		}

		let mut i = 0;
		while i < 7 {
			let mut j = 0;
			while j < 20 {
				let mut c = 0;
				while c < 52 {
					tableau[i][j][c] = rng.u64();
					c += 1;
				}
				j += 1;
			}
			i += 1;
		}

		let mut i = 0;
		while i < 4 {
			let mut c = 0;
			while c < 14 {
				foundations[i][c] = rng.u64();
				c += 1;
			}
			i += 1;
		}

		Self {
			stock,
			tableau,
			foundations,
		}
	}
}
impl Klondike {
	pub const ZOBRIST: ZobristKlondike = ZobristKlondike::new(0x15A4CDE);

	pub fn compute_hash(&self) -> u64 {
		let mut h = 0u64;

		// STOCK
		for (i, card) in self.stock.iter().enumerate() {
			h ^= Self::ZOBRIST.stock[i][card.index() as usize];
		}

		// REVEALED STOCK
		for (i, card) in self.revealed_stock.iter().enumerate() {
			h ^= Self::ZOBRIST.stock[i + 10][card.index() as usize];
		}

		// TABLEAU
		for (col, column) in self.tableau.iter().enumerate() {
			for (row, card) in column.iter().enumerate() {
				h ^= Self::ZOBRIST.tableau[col][row][card.index() as usize];
			}
		}

		// FOUNDATIONS
		for (f, foundation) in self.foundations.iter().enumerate() {
			for card in foundation.iter() {
				h ^= Self::ZOBRIST.foundations[f][card.index() as usize];
			}
		}

		h
	}
	fn update_stock_move(&mut self, card: PlayingCard54, from: usize, to: usize) {
		self.h ^= Self::ZOBRIST.stock[from][card.index() as usize];
		self.h ^= Self::ZOBRIST.stock[to][card.index() as usize];
	}
	fn update_tableau_move(
		&mut self,
		col: usize,
		from_row: usize,
		to_row: usize,
		card: PlayingCard54,
	) {
		self.h ^= Self::ZOBRIST.tableau[col][from_row][card.index() as usize];
		self.h ^= Self::ZOBRIST.tableau[col][to_row][card.index() as usize];
	}
	fn update_foundation_hash(&mut self, f: usize, card: PlayingCard54, add: bool) {
		let idx = card.index() as usize % 14;

		if add {
			self.h ^= Self::ZOBRIST.foundations[f][idx];
		} else {
			self.h ^= Self::ZOBRIST.foundations[f][idx];
		}
	}
}

impl Game for Klondike {
	type S = Self;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> kudchuet::GameOutcome {
		*moves = state.legal_moves();
		if moves.len() == 0 {
			if state.foundations.iter().map(|f| f.len()).sum::<usize>() == 52 {
				kudchuet::GameOutcome::PLAYER1
			} else {
				kudchuet::GameOutcome::PLAYER2
			}
		} else {
			kudchuet::GameOutcome::OnGoing
		}
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s = state.clone();
		s.play_unchecked(m);
		Some(s)
	}

	fn get_outcome(state: &Self::S) -> kudchuet::GameOutcome {
		let moves = state.legal_moves();
		if moves.len() == 0 {
			if state.foundations.iter().map(|f| f.len()).sum::<usize>() == 52 {
				kudchuet::GameOutcome::PLAYER1
			} else {
				kudchuet::GameOutcome::PLAYER2
			}
		} else {
			kudchuet::GameOutcome::OnGoing
		}
	}
	fn get_hash(state: &Self::S) -> u64 {
		let mut hasher = DefaultHasher::new();
		state.hash(&mut hasher);
		hasher.finish()
		//state.h
	}

	fn get_current_player(_state: &Self::S) -> kudchuet::Player {
		Player::PLAYER1
	}
	fn get_next_player(_state: &Self::S) -> Player {
		Player::PLAYER1
	}
}
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
			Move::TableauToTableau { from, to } => {
				let from_col = &state.tableau[*from];
				let to_col = &state.tableau[*to];

				if let Some(card_from) = from_col.iter().last() {
					let target = if let Some(card) = to_col.iter().last() {
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

				if let Some(card_from) = from_col.iter().last() {
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

					if let Some(card) = to_col.iter().last() {
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
impl CardMove<Klondike> for Move {
	fn click(&self) -> Option<CardGameClick<<Klondike as CardGame>::Card>> {
		todo!()
	}
}
impl CardGame for Klondike {
	type Card = PlayingCard54;

	fn build_board(&self) -> CardBoard<OrderedCardSet54, PlayingCard54> {
		let mut zones = vec![];

		zones.push(CardZone {
			id: 0,
			set: self.stock.clone(),
			layout: CardSetLayout::Stack,
			origin: pos2(0.05, 0.05),
			rect: Rect::from_min_max(pos2(0.05, 0.05), pos2(0.10, 0.35)),
			rotation: 0.0,
			card_spacing: 0.0,
			face_up: false,
			draw_empty: true,
			zone_only: true,
		});

		zones.push(CardZone {
			id: 1,
			set: self.revealed_stock.clone(),
			layout: CardSetLayout::Stack,
			origin: pos2(0.15, 0.05),
			rect: Rect::from_min_max(pos2(0.15, 0.05), pos2(0.25, 0.35)),
			rotation: 0.0,
			card_spacing: 30.0,
			face_up: true,
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
				origin: pos2(0.5 + i as f32 * table_width, 0.05),
				rect: Rect::from_min_max(
					pos2(0.5 + i as f32 * table_width, 0.05),
					pos2(0.5 + (i + 1) as f32 * table_width, 0.35),
				),
				rotation: 0.0,
				card_spacing: 20.0,
				face_up: true,
				draw_empty: true,
				zone_only: true,
			});
		}
		let len = self.tableau.len();
		let table_width = 0.9 / len as f32;
		for (i, col) in self.tableau.iter().enumerate() {
			zones.push(CardZone {
				id: i as u8 + 6,
				set: col.clone(),
				layout: CardSetLayout::Vertical,
				origin: pos2(0.05 + i as f32 * 0.12, 0.40),
				rect: Rect::from_min_max(
					pos2(0.05 + i as f32 * table_width, 0.40),
					pos2(0.05 + (i + 1) as f32 * table_width, 1.0),
				),
				rotation: 0.0,
				card_spacing: 20.0,
				face_up: true,
				draw_empty: true,
				zone_only: false,
			});
		}

		CardBoard { zones }
	}
}
#[cfg(test)]
mod tests {

	#[test]
	#[cfg(not(target_arch = "wasm32"))]
	fn test_gui() -> eframe::Result<()> {
		use crate::{gui::card_app::CardApp, klondike::Klondike};
		use kudchuet::ai::AIEngineProvider;
		use winit::platform::windows::EventLoopBuilderExtWindows;

		let engines: Vec<Box<dyn AIEngineProvider<Klondike>>> = vec![];
		//vec![Box::new(MoveSearcherBuilder::new(
		//	"Simple",
		//	FreecellEval,
		//	5,
		//))];
		let mut board = CardApp::new(Klondike::new(), engines);
		board.max_depth = 13;
		board.depth = 8;
		let mut options = eframe::NativeOptions::default();
		options.event_loop_builder = Some(Box::new(|builder| {
			builder.with_any_thread(true);
		}));
		eframe::run_native("Klondike", options, Box::new(|_cc| Ok(Box::new(board))))
	}
}
