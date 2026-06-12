//use std::hash::{DefaultHasher, Hasher};
use std::hash::Hash;

use egui::{Rect, pos2};
use kudchuet::{
	Player,
	ai::move_search::{Evaluator, Game},
	gui::{BoardStyle, DefaultSettings, GUIGame, GUIMove},
};

use crate::freecell::{are_alternate, foundation_index, rank};
use kudchuet::cards::{
	ordered_card_set::OrderedCardSet54,
	playing_cards::{CardSet, PlayingCard},
	playing_cards54::PlayingCard54,
	unordered_card_set::UnorderedCardSet54,
};
use kudchuet::gui::{
	CardGame,
	card_view::{CardBoard, CardGameClick, CardSetLayout, CardZone},
};
#[derive(Debug, Clone, Hash)]
pub struct Klondike {
	pub stock: OrderedCardSet54,
	pub revealed_stock: OrderedCardSet54,
	pub tableau: [(OrderedCardSet54, u8); 7],
	pub foundations: [UnorderedCardSet54; 4],
	h: u64,
}
impl Default for Klondike {
	fn default() -> Self {
		Self::new()
	}
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Move {
	StockToReveal,
	RecycleStock,

	RevealToTableau {
		to: usize,
	},
	RevealToFoundation {
		foundation: usize,
	},

	TableauToTableau {
		from: usize,
		to: usize,
		count: usize,
	},
	TableauToFoundation {
		from: usize,
		foundation: usize,
	},
}
impl Klondike {
	pub fn new() -> Self {
		let mut deck = UnorderedCardSet54::ALL_BUT_JOKERS;

		let mut s = Self {
			stock: OrderedCardSet54::EMPTY,
			revealed_stock: OrderedCardSet54::EMPTY,
			tableau: [
				(
					CardSet::draw_random(1, &mut deck)
						.expect("Should work")
						.into(),
					0,
				),
				(
					CardSet::draw_random(2, &mut deck)
						.expect("Should work")
						.into(),
					1,
				),
				(
					CardSet::draw_random(3, &mut deck)
						.expect("Should work")
						.into(),
					2,
				),
				(
					CardSet::draw_random(4, &mut deck)
						.expect("Should work")
						.into(),
					3,
				),
				(
					CardSet::draw_random(5, &mut deck)
						.expect("Should work")
						.into(),
					4,
				),
				(
					CardSet::draw_random(6, &mut deck)
						.expect("Should work")
						.into(),
					5,
				),
				(
					CardSet::draw_random(7, &mut deck)
						.expect("Should work")
						.into(),
					6,
				),
			],
			foundations: [UnorderedCardSet54::EMPTY; 4],
			h: 0,
		};
		for t in &mut s.tableau {
			fastrand::shuffle(&mut t.0.0);
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
				if can_place_on_tableau(card, &col.0) {
					moves.push(Move::RevealToTableau { to });
				}
			}
		}

		// TABLEAU
		for (from, src) in self.tableau.iter().enumerate() {
			let Some(&card) = src.0.iter().last() else {
				continue;
			};

			let max_len = max_ordered_suffix(&src.0, src.1);

			for count in (1..=max_len).rev() {
				let first_card = src.0.iter().rev().nth(count - 1).copied().unwrap();

				for (to, dst) in self.tableau.iter().enumerate() {
					if from == to {
						continue;
					}

					if can_place_on_tableau(first_card, &dst.0) {
						moves.push(Move::TableauToTableau { from, to, count });
					}
				}
			}

			let f = foundation_index(card.color());

			if can_place_on_foundation(card, self.foundations[f]) {
				moves.push(Move::TableauToFoundation {
					from,
					foundation: f,
				});
			}
		}

		moves
	}
	pub fn play_unchecked(&mut self, m: Move) {
		match m {
			Move::StockToReveal => {
				let card = self.stock.pop().unwrap();
				let from = self.stock.len();
				let to = self.revealed_stock.len();

				self.revealed_stock.push(card);

				self.h ^= Self::ZOBRIST.stock[0][from][card.index() as usize];
				self.h ^= Self::ZOBRIST.stock[1][to][card.index() as usize];
			}

			Move::RecycleStock => {
				while let Some(card) = self.revealed_stock.pop() {
					self.stock.push(card);
				}
				self.h = self.compute_hash();
			}

			Move::RevealToTableau { to } => {
				let card = self.revealed_stock.pop().unwrap();

				let from = self.revealed_stock.len();
				let to_idx = self.tableau[to].0.len();

				self.tableau[to].0.push(card);

				self.h ^= Self::ZOBRIST.stock[1][from][card.index() as usize];
				self.h ^= Self::ZOBRIST.tableau[to][to_idx][card.index() as usize];
			}

			Move::RevealToFoundation { foundation } => {
				let card = self.revealed_stock.pop().unwrap();

				let from = self.revealed_stock.len();

				self.foundations[foundation].insert(card);

				self.h ^= Self::ZOBRIST.stock[1][from][card.index() as usize];
				self.h ^=
					Self::ZOBRIST.foundations[foundation][self.foundations[foundation].len() - 1];
				self.h ^= Self::ZOBRIST.foundations[foundation][self.foundations[foundation].len()];
			}

			Move::TableauToTableau { from, to, count } => {
				let mut cards = OrderedCardSet54::new(vec![]);
				for _ in 0..count {
					let card = self.tableau[from].0.pop().unwrap();
					let index = self.tableau[from].0.len();
					cards.push(card);
					self.h ^= Self::ZOBRIST.tableau[from][index][card.index() as usize];
				}
				for _ in 0..count {
					let card = cards.pop().unwrap();
					let index = self.tableau[to].0.len();
					self.h ^= Self::ZOBRIST.tableau[to][index][card.index() as usize];
					self.tableau[to].0.push(card);
				}

				let from_i = self.tableau[from].0.len();
				if from_i > 0 && self.tableau[from].1 >= from_i as u8 {
					// face up last card;
					self.h ^= Self::ZOBRIST.tableau_hidden[from][self.tableau[from].1 as usize];
					self.tableau[from].1 -= 1;
					self.h ^= Self::ZOBRIST.tableau_hidden[from][self.tableau[from].1 as usize];
				}
			}

			Move::TableauToFoundation { from, foundation } => {
				let card = self.tableau[from].0.pop().unwrap();

				let from_i = self.tableau[from].0.len();
				if from_i > 0 && self.tableau[from].1 >= from_i as u8 {
					// face up last card;
					self.h ^= Self::ZOBRIST.tableau_hidden[from][self.tableau[from].1 as usize];
					self.tableau[from].1 -= 1;
					self.h ^= Self::ZOBRIST.tableau_hidden[from][self.tableau[from].1 as usize];
				}
				self.foundations[foundation].insert(card);

				self.h ^= Self::ZOBRIST.tableau[from][from_i][card.index() as usize];
				self.h ^=
					Self::ZOBRIST.foundations[foundation][self.foundations[foundation].len() - 1];
				self.h ^= Self::ZOBRIST.foundations[foundation][self.foundations[foundation].len()];
			}
		}
	}
}
fn can_place_on_tableau(card: PlayingCard54, col: &OrderedCardSet54) -> bool {
	col.iter()
		.last()
		.map(|top| are_alternate(card.color(), top.color()) && rank(card) + 1 == rank(*top))
		.unwrap_or(rank(card) == 13) // kings
}

fn can_place_on_foundation(card: PlayingCard54, f: UnorderedCardSet54) -> bool {
	rank(card) == f.len() as u8 + 1
}

pub fn max_ordered_suffix(column: &OrderedCardSet54, nb_hidden: u8) -> usize {
	let cards = &column.0[nb_hidden as usize..];
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
pub struct ZobristKlondike {
	pub stock: [[[u64; 52]; 24]; 2],
	pub tableau: [[[u64; 52]; 20]; 7],
	pub foundations: [[u64; 14]; 4],
	pub tableau_hidden: [[u64; 7]; 7],
}
impl ZobristKlondike {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);

		let mut stock = [[[0; 52]; 24]; 2];
		let mut tableau = [[[0; 52]; 20]; 7];
		let mut foundations = [[0; 14]; 4];
		let mut tableau_hidden = [[0; 7]; 7];

		let mut i = 0;
		while i < 24 {
			let mut c = 0;
			while c < 52 {
				stock[0][i][c] = rng.u64();
				stock[1][i][c] = rng.u64();
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
			let mut j = 0;
			while j < 7 {
				tableau_hidden[i][j] = rng.u64();
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
			tableau_hidden,
		}
	}
}
impl Klondike {
	pub const ZOBRIST: ZobristKlondike = ZobristKlondike::new(0x15A4CDE);

	pub fn compute_hash(&self) -> u64 {
		let mut h = 0u64;

		// STOCK
		for (i, card) in self.stock.iter().enumerate() {
			h ^= Self::ZOBRIST.stock[0][i][card.index() as usize];
		}

		// REVEALED STOCK
		for (i, card) in self.revealed_stock.iter().enumerate() {
			h ^= Self::ZOBRIST.stock[1][i][card.index() as usize];
		}

		// TABLEAU
		for (col, column) in self.tableau.iter().enumerate() {
			for (row, card) in column.0.iter().enumerate() {
				h ^= Self::ZOBRIST.tableau[col][row][card.index() as usize];
			}
			h ^= Self::ZOBRIST.tableau_hidden[col][column.1 as usize];
		}

		// FOUNDATIONS
		for (f, _foundation) in self.foundations.iter().enumerate() {
			h ^= Self::ZOBRIST.foundations[f][self.foundations[f].len()];
		}

		h
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

#[derive(Eq, PartialEq, Debug, Default, Copy, Clone)]
pub struct KlondikeEval;
impl Evaluator for KlondikeEval {
	type G = Klondike;

	fn evaluate_for(
		&self,
		s: &<Self::G as Game>::S,
		_p: Player,
	) -> kudchuet::ai::move_search::Evaluation {
		let foundation_count: usize = s.foundations.iter().map(|c| c.len()).sum();

		let foundation_variability_sum = s
			.foundations
			.iter()
			.map(|c| {
				let d = c.len() as i16 * 4 - foundation_count as i16;
				d * d
			})
			.sum::<i16>();
		let hidden_cards: i16 = s.tableau.iter().map(|c| c.1 as i16).sum::<i16>();
		let empty_columns = s.tableau.iter().filter(|c| c.0.is_empty()).count() as i16;
		let stock_count = s.revealed_stock.len() as i16 + s.stock.len() as i16;

		foundation_count as i16 * 103 - (foundation_variability_sum * 7) as i16 - hidden_cards * 191
			+ empty_columns * 51
			- stock_count * 17
	}
}

impl std::fmt::Display for Klondike {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(
			f,
			"Revealed Stock ({} remaining): {}",
			self.stock.len(),
			self.revealed_stock
				.iter()
				.last()
				.map(|c| c.to_string())
				.unwrap_or("None".to_string())
		)?;

		writeln!(f, "\nFoundations:")?;
		for foundation in &self.foundations {
			write!(f, "[{}] ", foundation)?;
		}
		writeln!(f)?;

		writeln!(f, "\nTableau:")?;
		for (i, column) in self.tableau.iter().enumerate() {
			writeln!(f, "{i}: {}", column.0)?;
		}

		Ok(())
	}
}
#[cfg(test)]
mod tests {
	use std::time::Duration;

	use crate::klondike::{Klondike, KlondikeEval};
	use kudchuet::ai::move_search::Strategy;
	use kudchuet::{
		ai::move_search::{IterativeOptions, IterativeSearch},
		gui::GUIGame,
	};

	#[test]
	fn test_legal_moves() {
		let mut klondike = Klondike::new();
		println!("{}", klondike);
		let mut moves = klondike.legal_moves();
		println!("{:?}", moves);
		let mut i = 0;
		while i < 500000 && !moves.is_empty() {
			let mv = fastrand::choice(&moves).unwrap();
			klondike.play_unchecked(*mv);
			assert_eq!(klondike.compute_hash(), klondike.h);
			moves = klondike.legal_moves();
			i += 1;
		}

		println!("{}", i);
		println!("{:?}", klondike.result());
		println!("{}", klondike);
		println!("{:?}", moves);
	}
	#[test]
	fn test_play() {
		let mut klondike = Klondike::new();
		println!("{}", klondike);
		let mut i = 0;
		let mut strategy = IterativeSearch::new(
			KlondikeEval,
			IterativeOptions::new()
				.with_shuffle_moves(false)
				.without_verbose(),
		);
		strategy.set_depth_or_timeout(12, Duration::from_secs(10));
		while i < 50000 {
			let mv = strategy.choose_move(&mut klondike);
			assert_eq!(klondike.compute_hash(), klondike.h);
			if let Some(mv) = mv {
				klondike.play(mv);
				println!("{}", klondike);
			} else {
				break;
			}
			i += 1;
		}

		println!("{}", i);
		println!("{:?}", klondike.result());
	}
	#[test]
	#[cfg(not(target_arch = "wasm32"))]
	fn test_gui() -> eframe::Result<()> {
		use crate::klondike::Klondike;
		use kudchuet::{
			ai::{AIEngineProvider, MoveSearcherBuilder},
			gui::{board_app::GenericGameApp, card_view::DefaultCardGameDrawer},
		};
		use winit::platform::windows::EventLoopBuilderExtWindows;

		let engines: Vec<Box<dyn AIEngineProvider<Klondike>>> = vec![Box::new(
			MoveSearcherBuilder::new("Cheating AI", KlondikeEval, 8),
		)];
		let mut board = GenericGameApp::new(Klondike::new(), engines);
		board.game_drawer = Box::new(DefaultCardGameDrawer::default());
		board.max_depth = 13;
		board.depth = 8;
		let mut options = eframe::NativeOptions::default();
		options.event_loop_builder = Some(Box::new(|builder| {
			builder.with_any_thread(true);
		}));
		eframe::run_native("Klondike", options, Box::new(|_cc| Ok(Box::new(board))))
	}
}
