mod bitboard;
pub mod gui;
pub mod game;
use std::fmt;

use ::bitboard::BitIter;

use crate::chinese_checkers::bitboard::ChineseCheckerBoard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ChineseCheckersPlayer {
	#[default]
	Red,
	Blue,
	Green,
	Yellow,
	Black,
	White,
}
impl ChineseCheckersPlayer {
	pub fn from_idx(idx: u8) -> Self {
		match idx {
			0 => Self::Red,
			1 => Self::Blue,
			2 => Self::Green,
			3 => Self::Yellow,
			4 => Self::Black,
			5 => Self::White,
			_ => panic!("Invalid Player Index")
		}
	}
	pub fn idx(&self) -> u8 {
		match self {
			Self::Red => 0,
			Self::Blue => 1,
			Self::Green => 2,
			Self::Yellow => 3,
			Self::Black => 4,
			Self::White => 5,
		}
	}
}
#[derive(Debug, Clone, Hash)]
pub struct ChineseCheckers {
	red: ChineseCheckerBoard,
	blue: ChineseCheckerBoard,
	green: ChineseCheckerBoard,
	yellow: ChineseCheckerBoard,
	black: ChineseCheckerBoard,
	white: ChineseCheckerBoard,
	nb_players: u8,
	turn: ChineseCheckersPlayer
}
impl ChineseCheckers {
	pub fn active_players(nb_players: u8) -> Vec<ChineseCheckersPlayer> {
		match nb_players {
			2 => vec![ChineseCheckersPlayer::Red, ChineseCheckersPlayer::Blue],
			3 => vec![ChineseCheckersPlayer::Red, ChineseCheckersPlayer::Yellow, ChineseCheckersPlayer::Black],
			4 => vec![ChineseCheckersPlayer::Red, ChineseCheckersPlayer::Blue, ChineseCheckersPlayer::Green, ChineseCheckersPlayer::Yellow],
			6 => vec![ChineseCheckersPlayer::Red, ChineseCheckersPlayer::Blue, ChineseCheckersPlayer::Green, ChineseCheckersPlayer::Yellow, ChineseCheckersPlayer::Black, ChineseCheckersPlayer::White],
			_ => panic!("Unsupported number of players"),
		}
	}
}
impl Default for ChineseCheckers {
	fn default() -> Self {
		Self::new(2)
	}
}
impl ChineseCheckers {
	pub fn new(nb_players: u8) -> Self {
		let mut game = Self {
			red: ChineseCheckerBoard::EMPTY,
			blue: ChineseCheckerBoard::EMPTY,
			green: ChineseCheckerBoard::EMPTY,
			yellow: ChineseCheckerBoard::EMPTY,
			black: ChineseCheckerBoard::EMPTY,
			white: ChineseCheckerBoard::EMPTY,
			nb_players,
			turn: ChineseCheckersPlayer::Red
		};

		for player in Self::active_players(nb_players) {
			match player {
				ChineseCheckersPlayer::Red => game.red = ChineseCheckerBoard::initial_red(),
				ChineseCheckersPlayer::Blue => game.blue = ChineseCheckerBoard::initial_blue(),
				ChineseCheckersPlayer::Green => game.green = ChineseCheckerBoard::initial_green(),
				ChineseCheckersPlayer::Yellow => game.yellow = ChineseCheckerBoard::initial_yellow(),
				ChineseCheckersPlayer::Black => game.black = ChineseCheckerBoard::initial_black(),
				ChineseCheckersPlayer::White => game.white = ChineseCheckerBoard::initial_white(),
			}
		}

		game
	}
	const INITIAL_RED: ChineseCheckerBoard = ChineseCheckerBoard::initial_red();
	const INITIAL_BLUE: ChineseCheckerBoard = ChineseCheckerBoard::initial_blue();
	const INITIAL_GREEN: ChineseCheckerBoard = ChineseCheckerBoard::initial_green();
	const INITIAL_YELLOW: ChineseCheckerBoard = ChineseCheckerBoard::initial_yellow();
	const INITIAL_WHITE: ChineseCheckerBoard = ChineseCheckerBoard::initial_white();
	const INITIAL_BLACK: ChineseCheckerBoard = ChineseCheckerBoard::initial_black();
	const FINAL_RED: ChineseCheckerBoard = Self::INITIAL_BLUE;
	const FINAL_BLUE: ChineseCheckerBoard = Self::INITIAL_RED;
	const FINAL_GREEN: ChineseCheckerBoard = Self::INITIAL_YELLOW;
	const FINAL_YELLOW: ChineseCheckerBoard = Self::INITIAL_GREEN;
	const FINAL_BLACK: ChineseCheckerBoard = Self::INITIAL_WHITE;
	const FINAL_WHITE: ChineseCheckerBoard = Self::INITIAL_BLACK;
	
	pub fn all(&self) -> ChineseCheckerBoard {
		self.red.clone()
			| self.blue.clone()
			| self.green.clone()
			| self.yellow.clone()
			| self.black.clone()
			| self.white.clone()
	}

	pub fn is_empty(&self, x: u8, y: u8) -> bool {
		!self.all().get(x, y)
	}

	pub fn board_mut(&mut self, player: ChineseCheckersPlayer) -> &mut ChineseCheckerBoard {
		match player {
			ChineseCheckersPlayer::Red => &mut self.red,
			ChineseCheckersPlayer::Blue => &mut self.blue,
			ChineseCheckersPlayer::Green => &mut self.green,
			ChineseCheckersPlayer::Yellow => &mut self.yellow,
			ChineseCheckersPlayer::Black => &mut self.black,
			ChineseCheckersPlayer::White => &mut self.white,
		}
	}

	pub fn board(&self, player: ChineseCheckersPlayer) -> &ChineseCheckerBoard {
		match player {
			ChineseCheckersPlayer::Red => &self.red,
			ChineseCheckersPlayer::Blue => &self.blue,
			ChineseCheckersPlayer::Green => &self.green,
			ChineseCheckersPlayer::Yellow => &self.yellow,
			ChineseCheckersPlayer::Black => &self.black,
			ChineseCheckersPlayer::White => &self.white,
		}
	}
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Move {
	pub from:u8,
	pub to:u8,
}
impl ChineseCheckers {
	fn jumps_from(&self, from: (u8, u8), visited: &mut ChineseCheckerBoard) -> Vec<(u8, u8)> {
		let mut results = Vec::new();
		visited.set(from.0, from.1);

		const DELTAS_EVEN: [(i8, i8); 6] = [(-1, 0), (1, 0), (0, -1), (1, -1), (0, 1), (1, 1)];
		const DELTAS_ODD: [(i8, i8); 6]  = [(-1, 0), (1, 0), (-1, -1), (0, -1), (-1, 1), (0, 1)];
		let deltas = if from.1 % 2 == 0 { DELTAS_EVEN } else { DELTAS_ODD };

		for (delta_index, (dx,dy)) in deltas.iter().enumerate() {
			let nx = from.0 as i8 + dx;
			let ny = from.1 as i8 + dy;

			if nx < 0 || nx >= 13 || ny < 0 || ny >= 17 {
				continue;
			}

			let nx = nx as u8;
			let ny = ny as u8;

			if !ChineseCheckerBoard::is_playable(nx, ny) || self.is_empty(nx, ny) {
				continue;
			}
			let delta_jump = if ny % 2 == 0 { DELTAS_EVEN } else { DELTAS_ODD }[delta_index];

			let jump_x_i8 = nx as i8 + delta_jump.0;
			let jump_y_i8 = ny as i8 + delta_jump.1;

			if jump_x_i8 < 0 || jump_x_i8 >= 13 || jump_y_i8 < 0 || jump_y_i8 >= 17 {
				continue;
			}

			let jump_x = jump_x_i8 as u8;
			let jump_y = jump_y_i8 as u8;

			if ChineseCheckerBoard::is_playable(jump_x, jump_y) &&
			self.is_empty(jump_x, jump_y) &&
			!visited.get(jump_x, jump_y) 
			{
				results.push((jump_x, jump_y));
				results.extend(self.jumps_from((jump_x, jump_y), visited));
			}
		}

		results
	}

	pub fn generate_moves(&self) -> Vec<Move> {
		self.generate_moves_for_player(self.turn)
	}
	pub fn generate_moves_for_player(&self, player: ChineseCheckersPlayer) -> Vec<Move> {
		let mut all_moves = Vec::new();
		let board = self.board(player);

		for bit in board.iter_bits() {
			let (x, y) = ChineseCheckerBoard::coords_from_index(bit as usize);

			// Mouvements simples
			let neighbours = ChineseCheckerBoard::NEIGHBOURS[y as usize][x as usize];
			for n_bit in neighbours.iter_bits() {
				let (nx, ny) = ChineseCheckerBoard::coords_from_index(n_bit as usize);
				if self.is_empty(nx, ny) {
					all_moves.push(Move{from:bit as u8, to:n_bit as u8});
				}
			}

			// Mouvements par saut
			let mut visited = ChineseCheckerBoard::EMPTY;
			for jump_to in self.jumps_from((x, y), &mut visited) {
				let jump_to_index = ChineseCheckerBoard::index_from_coords(jump_to.0, jump_to.1);
				all_moves.push(Move{from:bit as u8, to:jump_to_index as u8});
			}
		}
		all_moves
	}
}
impl ChineseCheckers {

	pub fn move_piece(
		&mut self,
		player: ChineseCheckersPlayer,
		mv: Move,
	) -> Result<(), &'static str> {
		if !ChineseCheckerBoard::is_playable_index(mv.from) {
			return Err("La case de départ n'est pas jouable");
		}
		if !ChineseCheckerBoard::is_playable_index(mv.to) {
			return Err("La case d'arrivée n'est pas jouable");
		}

		let all_before = self.all();
		if all_before.get_at_index(mv.to as usize) {
			return Err("La case d'arrivée est occupée");
		}

		let board = self.board_mut(player);

		if !board.get_at_index(mv.from as usize) {
			return Err("Aucun pion du joueur sur la case de départ");
		}

		board.reset_at_index(mv.from as usize);
		board.set_at_index(mv.to as usize);

		Ok(())
	}
	pub fn has_won(&self, player: &ChineseCheckerBoard, target: &ChineseCheckerBoard) -> bool {
		let outside_target = player.clone() & !target.clone();
		//println!("Pions hors cible: {:#?}", outside_target.to_u128());
		outside_target == ChineseCheckerBoard::EMPTY
	}
}

impl ChineseCheckers {
	pub fn winner(&self) -> Option<ChineseCheckersPlayer> {
		let active = Self::active_players(self.nb_players);

		for &player in &active {
			let board = self.board(player);
			let target = match player {
				ChineseCheckersPlayer::Red => ChineseCheckers::FINAL_RED,
				ChineseCheckersPlayer::Blue => ChineseCheckers::FINAL_BLUE,
				ChineseCheckersPlayer::Green => ChineseCheckers::FINAL_GREEN,
				ChineseCheckersPlayer::Yellow => ChineseCheckers::FINAL_YELLOW,
				ChineseCheckersPlayer::Black => ChineseCheckers::FINAL_BLACK,
				ChineseCheckersPlayer::White => ChineseCheckers::FINAL_WHITE,
			};
			if self.has_won(board, &target) {
				return Some(player);
			}
		}
		None
	}
}
impl ChineseCheckers {
	pub fn play(&mut self, mv: Move) -> Result<(), &'static str> {
		//let player_board = self.board_mut(self.turn);

		self.move_piece(self.turn, mv)?;

		self.next_turn();

		Ok(())
	}
	pub fn play_unchecked(&mut self, mv: Move) {
		let board = self.board_mut(self.turn);
		board.reset_at_index(mv.from as usize);
		board.set_at_index(mv.to as usize);
		self.next_turn();

	}

	fn next_turn(&mut self) {
		let active = Self::active_players(self.nb_players);
		let current_index = active.iter().position(|p| *p == self.turn).unwrap();
		let next_index = (current_index + 1) % active.len();
		self.turn = active[next_index];
	}
}
impl fmt::Display for ChineseCheckers {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for y in 0..17 {

			if y % 2 == 0 {
				write!(f, " ")?;
			}

			for x in 0..13 {
				if !ChineseCheckerBoard::is_playable(x, y) {
					write!(f, "  ")?;
					continue;
				}

				let c = if self.red.get(x, y) { 'R' }
						else if self.blue.get(x, y) { 'B' }
						else if self.green.get(x, y) { 'G' }
						else if self.yellow.get(x, y) { 'Y' }
						else if self.black.get(x, y) { 'b' }
						else if self.white.get(x, y) { 'w' }
						else { '.' };

				write!(f, "{} ", c)?;
			}
			writeln!(f)?;
		}
		Ok(())
	}
}

#[test]
fn test_display() {
	let board = ChineseCheckers::new(2);
	println!("{}", board);
	let board = ChineseCheckers::new(3);
	println!("{}", board);
	let board = ChineseCheckers::new(4);
	println!("{}", board);
	let board = ChineseCheckers::new(6);
	println!("{}", board);
	let board = ChineseCheckers::default();
	println!("{}", board);
}
#[test]
fn test_jump_simple() {
	let mut game = ChineseCheckers::new(2);

	game.red = ChineseCheckerBoard::EMPTY;
	game.blue = ChineseCheckerBoard::EMPTY;

	game.red.set(6, 8);

	game.blue.set(7, 8);
	println!("{}", game);
	println!("{}", ChineseCheckerBoard::neighbours(8, 8));
	let from_index = ChineseCheckerBoard::index_from_coords(6, 8);
	let mid_index  = ChineseCheckerBoard::index_from_coords(7, 8);
	let to_index   = ChineseCheckerBoard::index_from_coords(8, 8);
	println!("from={}, mid={}, to={}", from_index, mid_index, to_index);
	
	let moves = game.generate_moves_for_player(ChineseCheckersPlayer::Red);

	let from = ChineseCheckerBoard::index_from_coords(6, 8);
	let to   = ChineseCheckerBoard::index_from_coords(8, 8);

	let jump_move = Move { from: from as u8, to: to as u8 };

	assert!(
		moves.contains(&jump_move),
		"Jump simple non généré ! Moves: {:?}",
		moves
	);
}