use kudchuet::{
	gui::{BoardStyle, DefaultSettings, GUIGame},
	utils::fibo_hash_64,
};

use kudchuet::cards::{
	ordered_card_set::OrderedCardSet54,
	playing_cards::{CardSet, CardSuit, PlayingCard},
	playing_cards54::PlayingCard54,
	unordered_card_set::UnorderedCardSet54,
};
use kudchuet::gui::card_game_drawer::CardGameClick;
use std::fmt;
//use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone, Debug, Hash)]
pub struct Freecell {
	pub free_cells: [Option<PlayingCard54>; 4],
	pub tableau: [OrderedCardSet54; 8],
	pub foundations: [UnorderedCardSet54; 4],
	saved_tableau: Option<[OrderedCardSet54; 8]>,
	saved_foundation: Option<[UnorderedCardSet54; 4]>,
	saved_free_cells: Option<[Option<PlayingCard54>; 4]>,
	pub(crate) h: u64,
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
			saved_tableau: None,
			saved_foundation: None,
			saved_free_cells: None,
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
	Finish,
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
		let mut all_ordered = true;
		for (from, src) in self.tableau.iter().enumerate() {
			if all_ordered && !is_well_ordered(src) {
				all_ordered = false;
			}
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
		if all_ordered && self.foundations.iter().any(|f| f.len() < 13) {
			moves.push(Move::Finish);
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
			Move::Finish => {
				let mut saved = [OrderedCardSet54::EMPTY; 8];
				std::mem::swap(&mut saved, &mut self.tableau);
				self.saved_tableau = Some(saved);
				let mut saved = [
					UnorderedCardSet54::by_color(CardSuit::Spades),
					UnorderedCardSet54::by_color(CardSuit::Hearts),
					UnorderedCardSet54::by_color(CardSuit::Diamonds),
					UnorderedCardSet54::by_color(CardSuit::Clubs),
				];
				std::mem::swap(&mut saved, &mut self.foundations);
				self.saved_foundation = Some(saved);
				let mut saved = [None, None, None, None];
				std::mem::swap(&mut saved, &mut self.free_cells);
				self.saved_free_cells = Some(saved);
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
			Move::Finish => {
				self.tableau = self.saved_tableau.take().unwrap();
				self.foundations = self.saved_foundation.take().unwrap();
				self.free_cells = self.saved_free_cells.take().unwrap();
			}
		}
	}
	pub fn max_movables_cards(&self) -> usize {
		let free_cell_count = self.free_cells.iter().filter(|c| c.is_none()).count();
		let empty_columns = self.tableau.iter().filter(|c| c.is_empty()).count();

		(free_cell_count + 1) * (1usize << empty_columns)
	}
	pub fn are_all_cards_unlocked(&self) -> bool {
		for column in &self.tableau {
			if !is_well_ordered(column) {
				return false;
			}
		}
		true
	}
}
fn can_place_on_column(card: PlayingCard54, column: &OrderedCardSet54) -> bool {
	match column.iter().last().copied() {
		None => true,
		Some(top) => are_alternate(card.color(), top.color()) && rank(card) + 1 == rank(top),
	}
}

pub fn is_well_ordered(column: &OrderedCardSet54) -> bool {
	if column.is_empty() {
		return true;
	}

	max_ordered_suffix(column) == column.len()
}
pub fn max_ordered_suffix(column: &OrderedCardSet54) -> usize {
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
		for content in self.foundations.iter() {
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

impl GUIGame for Freecell {
	type Click = CardGameClick<PlayingCard54>;
	type Settings = DefaultSettings;

	type Style = BoardStyle;
	fn nb_players(&self) -> u8 {
		1
	}
}
#[cfg(test)]
mod tests {

	use std::time::Duration;

	use kudchuet::{
		ai::move_search::{IterativeOptions, IterativeSearch, Strategy, gametree::GameTree},
		cards::{ordered_card_set::OrderedCardSet54, playing_cards54::PlayingCard54},
		gui::GUIGame,
		utils::{fibo_hash_64, inv_fibo_hash_64},
	};

	use crate::{
		game::FreecellEval,
		rules::{Freecell, Move, max_ordered_suffix},
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
}
