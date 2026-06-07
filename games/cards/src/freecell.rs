use egui::{Rect, pos2};
use kudchuet::{
	Player,
	ai::move_search::{Evaluator, Game},
	gui::{BoardStyle, DefaultSettings, GUIGame, GUIMove},
	utils::fibo_hash_64,
};

use crate::{
	gui::{
		CardGame, CardMove,
		card_view::{CardBoard, CardGameClick, CardSetLayout, CardZone},
	},
	ordered_card_set::OrderedCardSet54,
	playing_cards::{CardSet, CardSuit, PlayingCard},
	playing_cards54::PlayingCard54,
	unordered_card_set::UnorderedCardSet54,
};
use std::fmt;
//use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone, Debug, Hash)]
pub struct Freecell {
	pub free_cells: [Option<PlayingCard54>; 4],
	pub tableau: [OrderedCardSet54; 8],
	pub foundations: [UnorderedCardSet54; 4],
	h: u64,
}
impl Default for Freecell {
	fn default() -> Self {
		Self::new()
	}
}
impl Freecell {
	pub fn new() -> Self {
		let mut deck = UnorderedCardSet54::ALL_BUT_JOKERS;

		let mut s = Self {
			free_cells: [None; 4],
			tableau: [
				CardSet::draw_random(7, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(7, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(7, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(7, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(6, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(6, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(6, &mut deck)
					.expect("Should work")
					.into(),
				CardSet::draw_random(6, &mut deck)
					.expect("Should work")
					.into(),
			],
			foundations: [UnorderedCardSet54::EMPTY; 4],
			h: 0,
		};
		for t in &mut s.tableau {
			fastrand::shuffle(&mut t.0);
		}
		s.h = s.compute_hash();
		s
	}
}

impl fmt::Display for Freecell {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "Free Cells:")?;
		for cell in &self.free_cells {
			match cell {
				Some(card) => write!(f, "[{}] ", card)?,
				None => write!(f, "[ ] ")?,
			}
		}
		writeln!(f)?;

		writeln!(f, "\nFoundations:")?;
		for foundation in &self.foundations {
			write!(f, "[{}] ", foundation)?;
		}
		writeln!(f)?;

		writeln!(f, "\nTableau:")?;
		for (i, column) in self.tableau.iter().enumerate() {
			writeln!(f, "{i}: {column}")?;
		}

		Ok(())
	}
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Move {
	TableauToTableau {
		from: usize,
		to: usize,
		count: usize,
	},
	TableauToFreeCell {
		from: usize,
		cell: usize,
	},
	FreeCellToTableau {
		cell: usize,
		to: usize,
	},
	TableauToFoundation {
		from: usize,
		foundation: usize,
	},
	FreeCellToFoundation {
		cell: usize,
		foundation: usize,
	},
}
impl Freecell {
	pub fn legal_moves_inplace(&self, moves: &mut Vec<Move>) {
		let free_cell_count = self.free_cells.iter().filter(|c| c.is_none()).count();
		let empty_columns = self.tableau.iter().filter(|c| c.is_empty()).count();

		let max_movable = (free_cell_count + 1) * (1usize << empty_columns);
		let max_movable_on_empty = if empty_columns == 0 {
			max_movable
		} else {
			(free_cell_count + 1) * (1usize << (empty_columns - 1))
		};

		// Tableau -> Foundation
		for (from, column) in self.tableau.iter().enumerate() {
			let Some(card) = column.iter().last().copied() else {
				continue;
			};
			let foundation = foundation_index(card.color());
			if can_move_to_foundation(card, self.foundations[foundation]) {
				moves.push(Move::TableauToFoundation { from, foundation });
			}
		}

		// FreeCell -> Foundation
		for (cell, card) in self.free_cells.iter().enumerate() {
			let Some(card) = card else { continue };

			let foundation = foundation_index(card.color());
			if can_move_to_foundation(*card, self.foundations[foundation]) {
				moves.push(Move::FreeCellToFoundation { cell, foundation });
			}
		}

		// Tableau -> Tableau
		for (from, src) in self.tableau.iter().enumerate() {
			if src.is_empty() {
				continue;
			}

			let max_len = max_ordered_suffix(src).min(max_movable);

			for count in (1..=max_len).rev() {
				let first_card = src.iter().rev().nth(count - 1).copied().unwrap();

				for (to, dst) in self.tableau.iter().enumerate() {
					if from == to {
						continue;
					}
					if dst.is_empty() && count > max_movable_on_empty {
						//println!("continue {}-{} ({}) with max_len:{} {}\n{}", from, to, count, max_len, max_movable_on_empty, self);
						continue;
					}
					//println!("can_place_on_column({}, {}) from {}", first_card, dst, src);

					if can_place_on_column(first_card, dst) {
						/*if count > 1 {
							println!("{}-{} ({})", from, to, count);
						}*/
						moves.push(Move::TableauToTableau { from, to, count });
					}
				}
			}
		}

		// FreeCell -> Tableau
		for (cell, card) in self.free_cells.iter().enumerate() {
			let Some(card) = card else { continue };

			for (to, column) in self.tableau.iter().enumerate() {
				if can_place_on_column(*card, column) {
					moves.push(Move::FreeCellToTableau { cell, to });
				}
			}
		}

		// Tableau -> FreeCell
		for (from, column) in self.tableau.iter().enumerate() {
			if column.is_empty() {
				continue;
			}

			for (cell, slot) in self.free_cells.iter().enumerate() {
				if slot.is_none() {
					moves.push(Move::TableauToFreeCell { from, cell });
				}
			}
		}
	}
	pub fn play_unchecked(&mut self, m: Move) {
		match m {
			Move::TableauToTableau { from, to, count } => {
				let mut cards = OrderedCardSet54::new(vec![]);
				for _ in 0..count {
					let card = self.tableau[from].pop().unwrap();
					let index = self.tableau[from].len();
					cards.push(card);
					self.update_hash_tableau(from, index, card);
				}
				for _ in 0..count {
					let card = cards.pop().unwrap();
					let index = self.tableau[to].len();
					self.update_hash_tableau(to, index, card);
					self.tableau[to].push(card);
				}
			}
			Move::TableauToFreeCell { from, cell } => {
				let card = self.tableau[from].pop().unwrap();
				self.free_cells[cell] = Some(card);
				self.update_hash_tableau(from, self.tableau[from].len(), card);
				self.update_hash_freecell(cell, card);
			}
			Move::FreeCellToTableau { cell, to } => {
				let card = self.free_cells[cell].take().unwrap();
				self.update_hash_tableau(to, self.tableau[to].len(), card);
				self.update_hash_freecell(cell, card);
				self.tableau[to].push(card);
			}
			Move::TableauToFoundation { from, foundation } => {
				let old_foundation = self.foundations[foundation].0;
				let card = self.tableau[from].pop().unwrap();
				self.foundations[foundation].insert(card);
				self.update_hash_tableau(from, self.tableau[from].len(), card);
				self.update_hash_foundation(
					foundation,
					old_foundation,
					self.foundations[foundation].0,
				);
			}
			Move::FreeCellToFoundation { cell, foundation } => {
				let old_foundation = self.foundations[foundation].0;
				let card = self.free_cells[cell].take().unwrap();
				self.foundations[foundation].insert(card);
				self.update_hash_freecell(cell, card);
				self.update_hash_foundation(
					foundation,
					old_foundation,
					self.foundations[foundation].0,
				);
			}
		}
	}
	pub fn undo_unchecked(&mut self, m: Move) {
		match m {
			Move::TableauToTableau { from, to, count } => {
				let mut cards = OrderedCardSet54::new(vec![]);
				for _ in 0..count {
					let card = self.tableau[to].pop().unwrap();
					let index = self.tableau[to].len();
					cards.push(card);
					self.update_hash_tableau(to, index, card);
				}
				for _ in 0..count {
					let card = cards.pop().unwrap();
					let index = self.tableau[from].len();
					self.update_hash_tableau(from, index, card);
					self.tableau[from].push(card);
				}
			}
			Move::TableauToFreeCell { from, cell } => {
				let card = self.free_cells[cell].take().unwrap();
				let index = self.tableau[from].len();
				self.tableau[from].push(card);
				self.update_hash_tableau(from, index, card);
				self.update_hash_freecell(cell, card);
			}
			Move::FreeCellToTableau { cell, to } => {
				self.free_cells[cell] = self.tableau[to].pop();
				let card = self.free_cells[cell].unwrap();
				self.update_hash_tableau(to, self.tableau[to].len(), card);
				self.update_hash_freecell(cell, card);
			}
			Move::TableauToFoundation { from, foundation } => {
				let old_foundation = self.foundations[foundation].0;
				let card = self.foundations[foundation].pop().unwrap();
				let index = self.tableau[from].len();
				self.tableau[from].push(card);
				self.update_hash_tableau(from, index, card);
				self.update_hash_foundation(
					foundation,
					old_foundation,
					self.foundations[foundation].0,
				);
			}
			Move::FreeCellToFoundation { cell, foundation } => {
				let old_foundation = self.foundations[foundation].0;
				let card = self.foundations[foundation].pop().unwrap();
				self.free_cells[cell] = Some(card);
				self.update_hash_freecell(cell, card);
				self.update_hash_foundation(
					foundation,
					old_foundation,
					self.foundations[foundation].0,
				);
			}
		}
	}
	fn max_movables_cards(&self) -> usize {
		let free_cell_count = self.free_cells.iter().filter(|c| c.is_none()).count();
		let empty_columns = self.tableau.iter().filter(|c| c.is_empty()).count();

		(free_cell_count + 1) * (1usize << empty_columns)
	}
}
fn can_place_on_column(card: PlayingCard54, column: &OrderedCardSet54) -> bool {
	match column.iter().last().copied() {
		None => true,
		Some(top) => are_alternate(card.color(), top.color()) && rank(card) + 1 == rank(top),
	}
}
fn max_ordered_suffix(column: &OrderedCardSet54) -> usize {
	let cards = &column.0;

	if cards.is_empty() {
		return 0;
	}

	let mut len = 1;

	for pair in cards.windows(2).rev() {
		let upper = pair[0];
		let lower = pair[1];

		if are_alternate(lower.color(), upper.color()) && rank(lower) + 1 == rank(upper) {
			len += 1;
		} else {
			break;
		}
	}

	len
}
fn can_move_to_foundation(card: PlayingCard54, foundation: UnorderedCardSet54) -> bool {
	use PlayingCard54::*;

	match foundation.len() {
		0 => matches!(card, AceOfSpades | AceOfHearts | AceOfDiamonds | AceOfClubs),
		n => rank(card) == n as u8 + 1,
	}
}
pub fn foundation_index(color: CardSuit) -> usize {
	match color {
		CardSuit::Spades => 0,
		CardSuit::Hearts => 1,
		CardSuit::Diamonds => 2,
		CardSuit::Clubs => 3,
		CardSuit::Joker => unreachable!(),
	}
}

pub fn rank(card: PlayingCard54) -> u8 {
	match card {
		PlayingCard54::AceOfSpades => 1,
		PlayingCard54::TwoOfSpades => 2,
		PlayingCard54::ThreeOfSpades => 3,
		PlayingCard54::FourOfSpades => 4,
		PlayingCard54::FiveOfSpades => 5,
		PlayingCard54::SixOfSpades => 6,
		PlayingCard54::SevenOfSpades => 7,
		PlayingCard54::EightOfSpades => 8,
		PlayingCard54::NineOfSpades => 9,
		PlayingCard54::TenOfSpades => 10,
		PlayingCard54::JackOfSpades => 11,
		PlayingCard54::QueenOfSpades => 12,
		PlayingCard54::KingOfSpades => 13,
		PlayingCard54::AceOfHearts => 1,
		PlayingCard54::TwoOfHearts => 2,
		PlayingCard54::ThreeOfHearts => 3,
		PlayingCard54::FourOfHearts => 4,
		PlayingCard54::FiveOfHearts => 5,
		PlayingCard54::SixOfHearts => 6,
		PlayingCard54::SevenOfHearts => 7,
		PlayingCard54::EightOfHearts => 8,
		PlayingCard54::NineOfHearts => 9,
		PlayingCard54::TenOfHearts => 10,
		PlayingCard54::JackOfHearts => 11,
		PlayingCard54::QueenOfHearts => 12,
		PlayingCard54::KingOfHearts => 13,
		PlayingCard54::AceOfDiamonds => 1,
		PlayingCard54::TwoOfDiamonds => 2,
		PlayingCard54::ThreeOfDiamonds => 3,
		PlayingCard54::FourOfDiamonds => 4,
		PlayingCard54::FiveOfDiamonds => 5,
		PlayingCard54::SixOfDiamonds => 6,
		PlayingCard54::SevenOfDiamonds => 7,
		PlayingCard54::EightOfDiamonds => 8,
		PlayingCard54::NineOfDiamonds => 9,
		PlayingCard54::TenOfDiamonds => 10,
		PlayingCard54::JackOfDiamonds => 11,
		PlayingCard54::QueenOfDiamonds => 12,
		PlayingCard54::KingOfDiamonds => 13,
		PlayingCard54::AceOfClubs => 1,
		PlayingCard54::TwoOfClubs => 2,
		PlayingCard54::ThreeOfClubs => 3,
		PlayingCard54::FourOfClubs => 4,
		PlayingCard54::FiveOfClubs => 5,
		PlayingCard54::SixOfClubs => 6,
		PlayingCard54::SevenOfClubs => 7,
		PlayingCard54::EightOfClubs => 8,
		PlayingCard54::NineOfClubs => 9,
		PlayingCard54::TenOfClubs => 10,
		PlayingCard54::JackOfClubs => 11,
		PlayingCard54::QueenOfClubs => 12,
		PlayingCard54::KingOfClubs => 13,
		PlayingCard54::BlackJoker => 0,
		PlayingCard54::RedJoker => 0,
	}
}
pub fn are_alternate(c1: CardSuit, c2: CardSuit) -> bool {
	match (c1, c2) {
		(CardSuit::Spades, CardSuit::Hearts) => true,
		(CardSuit::Spades, CardSuit::Diamonds) => true,
		(CardSuit::Spades, CardSuit::Joker) => true,
		(CardSuit::Hearts, CardSuit::Spades) => true,
		(CardSuit::Hearts, CardSuit::Clubs) => true,
		(CardSuit::Hearts, CardSuit::Joker) => true,
		(CardSuit::Diamonds, CardSuit::Spades) => true,
		(CardSuit::Diamonds, CardSuit::Clubs) => true,
		(CardSuit::Diamonds, CardSuit::Joker) => true,
		(CardSuit::Clubs, CardSuit::Hearts) => true,
		(CardSuit::Clubs, CardSuit::Diamonds) => true,
		(CardSuit::Clubs, CardSuit::Joker) => true,
		(CardSuit::Joker, _) => true,
		(_, _) => false,
	}
}

pub struct Zobrist {
	pub tableau: [[[u64; 52]; 21]; 8],
	pub free_cells: [[u64; 52]; 4],
	//pub foundations: [u64; 4],
}

impl Zobrist {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);
		let mut tableau = [[[0; 52]; 21]; 8];
		let mut free_cells = [[0; 52]; 4];
		//let mut foundations = [0; 4];
		let mut i = 0;
		while i < tableau.len() {
			let mut j = 0;
			while j < tableau[i].len() {
				let mut k = 0;
				while k < tableau[i][j].len() {
					tableau[i][j][k] = rng.u64();
					k += 1;
				}
				j += 1;
			}
			i += 1;
		}
		let mut i = 0;
		while i < free_cells.len() {
			let mut j = 0;
			while j < free_cells[i].len() {
				free_cells[i][j] = rng.u64();
				j += 1;
			}
			i += 1;
		}
		//let mut i = 0;
		//while i < foundations.len() {
		//	foundations[i] = rng.u64();
		//	i += 1;
		//}
		Self {
			tableau,
			free_cells,
			//foundations,
		}
	}
}
impl Freecell {
	const ZOBRIST: Zobrist = Zobrist::new(1254);
	fn compute_hash(&self) -> u64 {
		let mut h = 0u64;
		for (i, column) in self.tableau.iter().enumerate() {
			for (j, c) in column.iter().enumerate() {
				h ^= Self::ZOBRIST.tableau[i][j][c.index() as usize];
			}
		}
		for (i, cell) in self.free_cells.iter().enumerate() {
			if let Some(c) = cell {
				h ^= Self::ZOBRIST.free_cells[i][c.index() as usize];
			}
		}
		for (_i, content) in self.foundations.iter().enumerate() {
			//h ^= Self::ZOBRIST.foundations[i] ^ content.0;
			h ^= fibo_hash_64(content.0);
		}
		h
	}
	fn update_hash_tableau(&mut self, tableau: usize, index: usize, c: PlayingCard54) {
		self.h ^= Self::ZOBRIST.tableau[tableau][index][c.index() as usize];
	}
	fn update_hash_freecell(&mut self, cell: usize, c: PlayingCard54) {
		self.h ^= Self::ZOBRIST.free_cells[cell][c.index() as usize];
	}
	fn update_hash_foundation(
		&mut self,
		_foundation: usize,
		old_foundation: u64,
		new_foundation: u64,
	) {
		self.h ^= fibo_hash_64(old_foundation);
		self.h ^= fibo_hash_64(new_foundation);
	}
}

impl Game for Freecell {
	type S = Self;

	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> kudchuet::GameOutcome {
		state.legal_moves_inplace(moves);
		if moves.len() == 0 {
			if state
				.tableau
				.iter()
				.all(|a| a.is_empty() && state.free_cells.iter().all(|c| c.is_none()))
			{
				kudchuet::GameOutcome::PLAYER1
			} else {
				kudchuet::GameOutcome::PLAYER2
			}
		} else {
			kudchuet::GameOutcome::OnGoing
		}
	}
	fn filter_moves(_state: &Self::S, moves: &mut Vec<Self::M>) {
		// todo: retain only one move to empty column
		let mut seen = [false; 8];

		moves.retain(|m| match m {
			Move::TableauToFreeCell { from, .. } => {
				if seen[*from] {
					false
				} else {
					seen[*from] = true;
					true
				}
			}
			_ => true,
		});
	}
	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		state.play_unchecked(m);
		None
	}
	fn undo(state: &mut Self::S, m: Self::M) {
		state.undo_unchecked(m);
	}

	fn get_outcome(state: &Self::S) -> kudchuet::GameOutcome {
		let moves = state.legal_moves();
		if moves.len() == 0 {
			if state
				.tableau
				.iter()
				.all(|a| a.is_empty() && state.free_cells.iter().all(|c| c.is_none()))
			{
				kudchuet::GameOutcome::PLAYER1
			} else {
				kudchuet::GameOutcome::PLAYER2
			}
		} else {
			kudchuet::GameOutcome::OnGoing
		}
	}
	fn get_hash(state: &Self::S) -> u64 {
		//let mut hasher = DefaultHasher::new();
		//state.hash(&mut hasher);
		//hasher.finish()
		state.h
	}

	fn get_current_player(_state: &Self::S) -> kudchuet::Player {
		Player::PLAYER1
	}
	fn get_next_player(_state: &Self::S) -> Player {
		Player::PLAYER1
	}
	fn notation(_state: &Self::S, _move: Self::M) -> Option<String> {
		match _move {
			Move::TableauToTableau { from, to, count } => {
				Some(format!("{}-{} ({})", from, to, count))
			}
			Move::TableauToFreeCell { from, cell } => Some(format!("{}-free({})", from, cell)),
			Move::FreeCellToTableau { cell, to } => Some(format!("free({})-{}", cell, to)),
			Move::TableauToFoundation {
				from,
				foundation: _,
			} => Some(format!("{}-f", from)),
			Move::FreeCellToFoundation {
				cell,
				foundation: _,
			} => Some(format!("free({})-f", cell)),
		}
	}
}
impl GUIGame for Freecell {
	type Click = CardGameClick<PlayingCard54>;
	type Settings = DefaultSettings;

	type Style = BoardStyle;
	fn nb_players(&self) -> u8 {
		1
	}
}
impl CardGame for Freecell {
	type Card = PlayingCard54;

	fn build_board(&self) -> CardBoard<OrderedCardSet54, PlayingCard54> {
		let mut zones = vec![];

		let x_step = 0.08;
		let base_y_top = 0.05;
		//let base_y_mid = 0.15;
		let base_y_bottom = 0.55;

		for (i, col) in self.foundations.iter().enumerate() {
			zones.push(CardZone {
				id: i as u8 + 4,
				set: OrderedCardSet54::from_iter(*col),
				layout: CardSetLayout::Stack,
				origin: pos2(0.60 + i as f32 * x_step, base_y_top),
				rect: Rect::from_min_max(
					pos2(0.60 + i as f32 * x_step, base_y_top),
					pos2(0.60 + (i + 1) as f32 * x_step, 0.35),
				),
				rotation: 0.0,
				card_spacing: 0.0,
				face_up: true,
				draw_empty: true,
				zone_only: true,
			});
		}

		for (i, cell) in self.free_cells.iter().enumerate() {
			zones.push(CardZone {
				id: i as u8,
				set: OrderedCardSet54::from_iter(cell.clone()),
				layout: CardSetLayout::Stack,
				origin: pos2(0.05 + i as f32 * x_step, base_y_top),
				rect: Rect::from_min_max(
					pos2(0.05 + i as f32 * x_step, base_y_top),
					pos2(0.05 + (i + 1) as f32 * x_step, 1.0),
				),
				rotation: 0.0,
				card_spacing: 0.03,
				face_up: true,
				draw_empty: true,
				zone_only: true,
			});
		}

		for (i, col) in self.tableau.iter().enumerate() {
			zones.push(CardZone {
				id: i as u8 + 8,
				set: col.clone(),
				layout: CardSetLayout::Vertical,
				origin: pos2(0.05 + i as f32 * 0.08, base_y_bottom),
				rect: Rect::from_min_max(
					pos2(0.05 + i as f32 * 0.08, base_y_bottom),
					pos2(0.05 + (i + 1) as f32 * 0.08, 0.35),
				),
				rotation: 0.0,
				card_spacing: 25.0,
				face_up: true,
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
				let card_from = state.tableau[*from].iter().last().unwrap();
				res.push(CardGameClick::Card(*card_from));
				res.push(CardGameClick::CardZone(*foundation as u8 + 4));
			}
			Move::FreeCellToFoundation { cell, foundation } => {
				res.push(CardGameClick::CardZone(*cell as u8));
				res.push(CardGameClick::CardZone(*foundation as u8 + 4));
			}
		}
		res
	}
}
impl CardMove<Freecell> for Move {
	fn click(&self) -> Option<CardGameClick<<Freecell as CardGame>::Card>> {
		todo!()
	}
}
#[derive(Eq, PartialEq, Debug, Default, Copy, Clone)]
pub struct FreecellEval;
impl Evaluator for FreecellEval {
	type G = Freecell;

	fn evaluate_for(
		&self,
		s: &<Self::G as Game>::S,
		_p: Player,
	) -> kudchuet::ai::move_search::Evaluation {
		let foundation_count: usize = s.foundations.iter().map(|c| c.len()).sum();
		let foundation_mean = foundation_count as f32 / 4.0;

		let mut foundation_variability: f32 = s
			.foundations
			.iter()
			.map(|c| {
				let l = c.len() as f32;
				(l - foundation_mean) * (l - foundation_mean)
			})
			.sum();
		foundation_variability /= 4.0;
		let max_movable = s.max_movables_cards();
		let mut misorder: i16 = 0;
		let mut ordered_bonus: i16 = 0;
		for column in &s.tableau {
			if column.is_empty() {
				continue;
			}
			let ordered = max_ordered_suffix(column);
			let unordered_count = column.len() - ordered;
			misorder += unordered_count as i16 * 11;
			ordered_bonus += ordered as i16 * 5;
			if unordered_count == 0 {
				let rank = rank(column.0[0]);
				ordered_bonus += rank as i16 * 13;
			}
			let unordered = &column.0[0..unordered_count];
			for (i, c) in unordered.iter().enumerate() {
				let rank = rank(*c);
				if rank < 6 {
					misorder += (ordered + i) as i16;
				}
			}
		}
		if misorder == 0 {
			ordered_bonus = 0;
		}
		max_movable.min(12) as i16 * 71 + foundation_count as i16 * 191
			- (foundation_variability * 23.0) as i16
			- misorder as i16 * 3
			+ ordered_bonus
	}
}

#[cfg(test)]
mod tests {

	use std::time::Duration;

	use kudchuet::{
		ai::move_search::{IterativeOptions, IterativeSearch, Strategy, gametree::GameTree},
		gui::GUIGame,
		utils::{fibo_hash_64, inv_fibo_hash_64},
	};

	use crate::{
		freecell::{Freecell, FreecellEval, Move, max_ordered_suffix},
		ordered_card_set::OrderedCardSet54,
		playing_cards54::PlayingCard54,
	};

	#[test]
	fn test_legal_moves() {
		let mut freecell = Freecell::new();
		println!("{}", freecell);
		let mut moves = freecell.legal_moves();
		println!("{:?}", moves);
		let mut i = 0;
		while i < 500000 && !moves.is_empty() {
			let mv = fastrand::choice(&moves).unwrap();
			if i % 1000 != 0
				&& let Move::TableauToFreeCell { from: _, cell: _ } = mv
			{
				i -= 1;
				continue;
			}
			freecell.play_unchecked(*mv);
			assert_eq!(freecell.compute_hash(), freecell.h);
			moves = freecell.legal_moves();
			i += 1;
		}

		println!("{}", i);
		println!("{:?}", freecell.result());
		println!("{}", freecell);
		println!("{:?}", moves);
	}
	#[test]
	fn test_play() {
		let mut freecell = Freecell::new();
		println!("{}", freecell);
		let mut i = 0;
		let mut strategy = IterativeSearch::new(
			FreecellEval,
			IterativeOptions::new().with_shuffle_moves(false),
		);
		strategy.set_depth_or_timeout(12, Duration::from_secs(1));
		while i < 500000 {
			let mv = strategy.choose_move(&mut freecell);
			assert_eq!(freecell.compute_hash(), freecell.h);
			if let Some(mv) = mv {
				freecell.play(mv);
				println!("{}", freecell);
			} else {
				break;
			}
			i += 1;
		}

		println!("{}", i);
		println!("{:?}", freecell.result());
	}
	#[test]
	fn test_max_ordered() {
		let column = OrderedCardSet54::new(vec![
			PlayingCard54::AceOfSpades,
			PlayingCard54::TwoOfSpades,
			PlayingCard54::KingOfSpades,
			PlayingCard54::QueenOfDiamonds,
			PlayingCard54::JackOfClubs,
		]);
		let max_len = max_ordered_suffix(&column);
		assert_eq!(3, max_len);
		for count in (1..=max_len).rev() {
			let first_card = column.iter().rev().nth(count - 1).copied().unwrap();
			if count == 3 {
				assert_eq!(first_card, PlayingCard54::KingOfSpades);
			} else if count == 2 {
				assert_eq!(first_card, PlayingCard54::QueenOfDiamonds);
			} else if count == 1 {
				assert_eq!(first_card, PlayingCard54::JackOfClubs);
			}
		}
		let column = OrderedCardSet54::new(vec![
			PlayingCard54::AceOfSpades,
			PlayingCard54::TenOfHearts,
			PlayingCard54::QueenOfDiamonds,
			PlayingCard54::JackOfClubs,
			PlayingCard54::TenOfHearts,
		]);
		let max_len = max_ordered_suffix(&column);
		assert_eq!(3, max_len);
		for count in (1..=max_len).rev() {
			let first_card = column.iter().rev().nth(count - 1).copied().unwrap();
			if count == 3 {
				assert_eq!(first_card, PlayingCard54::QueenOfDiamonds);
			} else if count == 2 {
				assert_eq!(first_card, PlayingCard54::JackOfClubs);
			} else if count == 1 {
				assert_eq!(first_card, PlayingCard54::TenOfHearts);
			} else {
				panic!()
			}
		}
	}
	#[test]
	fn test_perfect() {
		let test_input = 2;
		println!(
			"{} -> {} -> {}",
			test_input,
			fibo_hash_64(test_input),
			inv_fibo_hash_64(fibo_hash_64(test_input))
		);
		let freecell = Freecell::new();
		println!("{}", freecell);
		let mut tree = GameTree::<Freecell, ()>::from(freecell);
		tree.expand_to_depth(0, 80);
		//println!("{:?}", freecell.result());
	}

	#[test]
	#[cfg(not(target_arch = "wasm32"))]
	fn test_gui() -> eframe::Result<()> {
		use crate::gui::card_app::CardApp;
		use kudchuet::ai::{AIEngineProvider, MoveSearcherBuilder};
		use winit::platform::windows::EventLoopBuilderExtWindows;

		let engines: Vec<Box<dyn AIEngineProvider<Freecell>>> = vec![Box::new(
			MoveSearcherBuilder::new("Simple", FreecellEval, 5),
		)];
		let mut board = CardApp::new(Freecell::new(), engines);
		board.max_depth = 13;
		board.depth = 8;
		let mut options = eframe::NativeOptions::default();
		options.event_loop_builder = Some(Box::new(|builder| {
			builder.with_any_thread(true);
		}));
		eframe::run_native("Freecell", options, Box::new(|_cc| Ok(Box::new(board))))
	}
}
