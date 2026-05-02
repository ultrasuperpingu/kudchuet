//! A definition of the game Tic-Tac-Toe using the library, for use in tests.
//!
//! For example, playing a correctly-implemented strategy against itself should
//! always result in a draw; and playing such a strategy against one that picks
//! moves randomly should always result in a win or draw.
#![allow(dead_code)]

extern crate kudchuet;

use std::default::Default;
use std::fmt::{Display, Formatter, Result};

use kudchuet::ai::minimax::gametree::GameTree;
use kudchuet::{GameOutcome, Player, utils};
use kudchuet::ai::minimax::{Evaluation, Evaluator, ExpectiMinimax};
use kudchuet::ai::minimax::{Game, Strategy};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Board {
	nb: u8,
	to_move: Player,
}

impl Board {
	pub fn new(nb: u8) -> Self {
		Self { nb, to_move: Player::PLAYER1 }
	}
	fn just_moved(&self) -> Player {
		self.to_move.opponent()
	}
}

impl Default for Board {
	fn default() -> Board {
		Board {
			nb: 20,
			to_move: Player::PLAYER1,
		}
	}
}

impl Display for Board {
	fn fmt(&self, f: &mut Formatter) -> Result {
		let board = " | ".repeat(self.nb as usize);
		writeln!(
			f,
			"To play: {}\n{board}",
			self.to_move
		)?;
		Ok(())
	}
}
#[derive(Debug)]
pub struct NimGame;

impl Game for NimGame {
	type S = Board;
	type M = Place;

	fn generate_moves(b: &Board, ms: &mut Vec<Place>) -> GameOutcome {
		if b.nb == 0 {
			return b.to_move.into();
		}
		for i in 1..=3 {
			if b.nb >= i {
				ms.push(Place { i: i as u8 });
			}
		}
		GameOutcome::OnGoing
	}

	fn get_outcome(b: &Board) -> GameOutcome {
		if b.nb == 0 {
			return b.to_move.into();
		} else {
			GameOutcome::OnGoing
		}
	}

	fn apply(b: &mut Board, m: Place) -> Option<Board> {
		b.nb -= m.i;
		b.to_move = b.to_move.opponent();
		None
	}
	fn undo(b: &mut Board, m: Place) {
		b.nb += m.i;
		b.to_move = b.to_move.opponent();
	}
	fn get_current_player(state: &Self::S) -> Player {
		state.to_move
	}
	fn get_hash(state: &Self::S) -> u64 {
		utils::splitmix64(state.nb as u64 | ((state.to_move.0 as u64) << 9))
	}
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Place {
	pub(crate) i: u8,
}

impl Display for Place {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "@{}", self.i)
	}
}

pub struct TTTEvaluator;

impl Default for TTTEvaluator {
	fn default() -> Self {
		Self {}
	}
}

impl Evaluator for TTTEvaluator {
	type G = NimGame;
	fn evaluate_for(&self, _b: &Board, _p: Player) -> Evaluation {
		0
	}
}

fn main() {
	let nb_sticks = 10;
	let mut b = Board::new(nb_sticks);
	let mut strategies = vec![
		ExpectiMinimax::new(TTTEvaluator::default(), 10, true),
		ExpectiMinimax::new(TTTEvaluator::default(), 10, true),
	];
	let mut s = 0;
	while !NimGame::get_outcome(&b).is_ended() {
		println!("{}", b);
		let ref mut strategy = strategies[s];
		match strategy.choose_move(&mut b) {
			Some(m) => NimGame::apply(&mut b, m),
			None => break,
		};
		s = 1 - s;
	}
	println!("{}", b);
	let outcome = NimGame::get_outcome(&b);
	println!("{:?}", outcome);


	let b = Board::new(nb_sticks);
	let mut tree = GameTree::<NimGame>::from(b);
	let outcome2 = tree.expand_all(0, false);
	println!("{:?}", outcome2);
	println!("{}", tree);
	tree.set_root_id(38);
	tree.cleanup();
	println!("{}", tree);
	assert_eq!(outcome, outcome2);
}
