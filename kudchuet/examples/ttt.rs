//! A definition of the game Tic-Tac-Toe using the library, for use in tests.
//!
//! For example, playing a correctly-implemented strategy against itself should
//! always result in a draw; and playing such a strategy against one that picks
//! moves randomly should always result in a win or draw.
#![allow(dead_code)]

extern crate kudchuet;

use std::default::Default;
use std::fmt::{Display, Formatter, Result};
use std::hash::{DefaultHasher, Hash, Hasher};

use kudchuet::{GameOutcome, Player};
use kudchuet::ai::minimax::{Evaluation, Evaluator, ExpectiMinimax};
use kudchuet::ai::minimax::{Game, Strategy};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum Square {
	Empty,
	X,
	O,
}

impl Square {
	fn invert(&self) -> Self {
		match *self {
			Square::Empty => Square::Empty,
			Square::X => Square::O,
			Square::O => Square::X,
		}
	}
}

impl Default for Square {
	fn default() -> Square {
		Square::Empty
	}
}

impl Display for Square {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(
			f,
			"{}",
			match *self {
				Square::Empty => ' ',
				Square::X => 'X',
				Square::O => 'O',
			}
		)
	}
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Board {
	squares: [Square; 9],
	to_move: Square,
}

impl Board {
	fn just_moved(&self) -> Square {
		self.to_move.invert()
	}
}

impl Default for Board {
	fn default() -> Board {
		Board {
			squares: [Square::default(); 9],
			to_move: Square::X,
		}
	}
}

impl Display for Board {
	fn fmt(&self, f: &mut Formatter) -> Result {
		writeln!(
			f,
			"{} | {} | {}",
			self.squares[0], self.squares[1], self.squares[2]
		)?;
		writeln!(
			f,
			"{} | {} | {}",
			self.squares[3], self.squares[4], self.squares[5]
		)?;
		writeln!(
			f,
			"{} | {} | {}",
			self.squares[6], self.squares[7], self.squares[8]
		)?;
		Ok(())
	}
}
#[derive(Debug)]
pub struct TTTGame;

impl Game for TTTGame {
	type S = Board;
	type M = Place;

	fn generate_moves(b: &Board, ms: &mut Vec<Place>) -> GameOutcome {
		let res = Self::get_outcome(b);
		if res.is_ended() {
			return res;
		}
		for i in 0..b.squares.len() {
			if b.squares[i] == Square::Empty {
				ms.push(Place { i: i as u8 });
			}
		}
		GameOutcome::OnGoing
	}

	fn get_outcome(b: &Board) -> GameOutcome {
		// A player can only cause themselves to win on their turn, so only check for that.

		// horizontal wins
		if b.squares[0] == b.just_moved()
			&& b.squares[0] == b.squares[1]
			&& b.squares[1] == b.squares[2]
		{
			return square_to_winner(b.just_moved());
		}
		if b.squares[3] == b.just_moved()
			&& b.squares[3] == b.squares[4]
			&& b.squares[4] == b.squares[5]
		{
			return square_to_winner(b.just_moved());
		}
		if b.squares[6] == b.just_moved()
			&& b.squares[6] == b.squares[7]
			&& b.squares[7] == b.squares[8]
		{
			return square_to_winner(b.just_moved());
		}
		// vertical wins
		if b.squares[0] == b.just_moved()
			&& b.squares[0] == b.squares[3]
			&& b.squares[3] == b.squares[6]
		{
			return square_to_winner(b.just_moved());
		}
		if b.squares[1] == b.just_moved()
			&& b.squares[1] == b.squares[4]
			&& b.squares[4] == b.squares[7]
		{
			return square_to_winner(b.just_moved());
		}
		if b.squares[2] == b.just_moved()
			&& b.squares[2] == b.squares[5]
			&& b.squares[5] == b.squares[8]
		{
			return square_to_winner(b.just_moved());
		}
		// diagonal wins
		if b.squares[0] == b.just_moved()
			&& b.squares[0] == b.squares[4]
			&& b.squares[4] == b.squares[8]
		{
			return square_to_winner(b.just_moved());
		}
		if b.squares[2] == b.just_moved()
			&& b.squares[2] == b.squares[4]
			&& b.squares[4] == b.squares[6]
		{
			return square_to_winner(b.just_moved());
		}
		// draws
		if b.squares.iter().all(|s| *s != Square::Empty) {
			GameOutcome::Draw
		} else {
			// non-terminal state
			GameOutcome::OnGoing
		}
	}

	fn apply(b: &mut Board, m: Place) -> Option<Board> {
		b.squares[m.i as usize] = b.to_move;
		b.to_move = b.to_move.invert();
		None
	}
	fn undo(b: &mut Board, m: Place) {
		b.squares[m.i as usize] = Square::Empty;
		b.to_move = b.to_move.invert();
	}
	fn get_current_player(state: &Self::S) -> Player {
		square_to_player(state.to_move)
	}
	fn get_hash(state: &Self::S) -> u64 {
		let mut hasher = DefaultHasher::new();
		state.hash(&mut hasher);
		hasher.finish()
	}
}
fn square_to_player(square: Square) -> Player {
	match square {
		Square::Empty => unreachable!(),
		Square::X => Player::PLAYER1,
		Square::O => Player::PLAYER2,
	}
}
fn square_to_winner(square: Square) -> GameOutcome {
	match square {
		Square::Empty => unreachable!(),
		Square::X => GameOutcome::PLAYER1,
		Square::O => GameOutcome::PLAYER2,
	}
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Place {
	i: u8,
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
	type G = TTTGame;
	// adapted from http://www.cs.olemiss.edu/~dwilkins/CSCI531/tic.c
	fn evaluate_for(&self, b: &Board, p: Player) -> Evaluation {
		let mut score = 0;

		// 3rd: check for doubles
		for i in 0..3 {
			let line = i * 3;
			if b.squares[line + 0] == b.squares[line + 1] {
				if b.squares[line + 0] == Square::X {
					score += 5;
				} else if b.squares[line + 0] == Square::O {
					score -= 5;
				}
			}
			if b.squares[line + 1] == b.squares[line + 2] {
				if b.squares[line + 1] == Square::X {
					score += 5;
				} else if b.squares[line + 1] == Square::O {
					score += 5;
				}
			}
			if b.squares[i] == b.squares[3 + i] {
				if b.squares[i] == Square::X {
					score += 5;
				} else if b.squares[i] == Square::O {
					score -= 5;
				}
			}
			if b.squares[3 + i] == b.squares[6 + i] {
				if b.squares[3 + i] == Square::X {
					score += 5;
				} else if b.squares[3 + i] == Square::O {
					score -= 5;
				}
			}
		}
		// 2nd: check for the middle square
		if b.squares[4] == Square::X {
			score += 5;
		}
		if b.squares[4] == Square::O {
			score -= 5;
		}
		if p == Player::PLAYER1 { score } else { -score }
	}
}

fn main() {
	let mut b = Board::default();
	let mut strategies = vec![
		ExpectiMinimax::new(TTTEvaluator::default(), 10, true),
		ExpectiMinimax::new(TTTEvaluator::default(), 10, true),
	];
	let mut s = 0;
	while !TTTGame::get_outcome(&b).is_ended() {
		println!("{}", b);
		let ref mut strategy = strategies[s];
		match strategy.choose_move(&mut b) {
			Some(m) => TTTGame::apply(&mut b, m),
			None => break,
		};
		s = 1 - s;
	}
	println!("{}", b);
	println!("{:?}", TTTGame::get_outcome(&b));
}
