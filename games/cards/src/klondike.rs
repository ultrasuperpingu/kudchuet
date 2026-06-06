use egui::pos2;

use crate::{
	freecell::{are_alternate, foundation_index, rank},
	gui::card_view::{CardSetLayout, CardZone},
	ordered_card_set::OrderedCardSet54,
	playing_cards::PlayingCard,
	playing_cards54::PlayingCard54,
	unordered_card_set::UnorderedCardSet54,
};
pub struct Klondike {
	pub stock: OrderedCardSet54,
	pub revealed_stock: OrderedCardSet54,
	pub tableau: [OrderedCardSet54; 7],
	pub foundations: [UnorderedCardSet54; 4],
	h: u64,
}
pub enum Move {
	StockToReveal,
	RecycleStock,

	RevealToTableau { to: usize },
	RevealToFoundation { foundation: usize },

	TableauToTableau { from: usize, to: usize },
	TableauToFoundation { from: usize, foundation: usize },
}
impl Klondike {
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
	pub stock: [[u64; 52]; 24], // stock + revealed (approx max 24 visibles)
	pub tableau: [[[u64; 52]; 20]; 7], // 7 colonnes max depth ~20
	pub foundations: [[u64; 14]; 4], // 13 cartes max + safe margin
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
		let idx = card.index() as usize;

		if add {
			self.h ^= Self::ZOBRIST.foundations[f][idx];
		} else {
			self.h ^= Self::ZOBRIST.foundations[f][idx];
		}
	}
	fn test_ui(&self) -> Vec<CardZone<OrderedCardSet54, PlayingCard54>> {
		let game = self;
		let mut zones = vec![];
		zones.push(CardZone {
			id: 0,
			set: game.stock.clone(),
			layout: CardSetLayout::Stack,
			origin: pos2(20.0, 20.0),
			rotation: 0.0,
			card_spacing: 0.0,
			face_up: false,
			draw_empty: true,
		});

		zones.push(CardZone {
			id: 1,
			set: game.revealed_stock.clone(),
			layout: CardSetLayout::Stack,
			origin: pos2(140.0, 20.0),
			rotation: 0.0,
			card_spacing: 30.0,
			face_up: true,
			draw_empty: true,
		});

		for (i, col) in game.tableau.iter().enumerate() {
			zones.push(CardZone {
				id: i as u8 + 2,
				set: col.clone(),
				layout: CardSetLayout::Vertical,
				origin: pos2(20.0 + i as f32 * 110.0, 180.0),
				rotation: 0.0,
				card_spacing: 25.0,
				face_up: true,
				draw_empty: true,
			});
		}
		zones
	}
}
