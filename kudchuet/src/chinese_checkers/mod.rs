mod bitboard;
pub mod gui;
pub mod game;
pub mod fen;
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
	hash: u64,
	turn: ChineseCheckersPlayer
}
impl ChineseCheckers {
	pub fn active_players(nb_players: u8) -> &'static [ChineseCheckersPlayer] {
	match nb_players {
		2 => &[ChineseCheckersPlayer::Red, ChineseCheckersPlayer::Blue],
		3 => &[ChineseCheckersPlayer::Red, ChineseCheckersPlayer::Yellow, ChineseCheckersPlayer::Black],
		4 => &[ChineseCheckersPlayer::Red, ChineseCheckersPlayer::Blue, ChineseCheckersPlayer::Green, ChineseCheckersPlayer::Yellow],
		6 => &[
			ChineseCheckersPlayer::Red,
			ChineseCheckersPlayer::Blue,
			ChineseCheckersPlayer::Green,
			ChineseCheckersPlayer::Yellow,
			ChineseCheckersPlayer::Black,
			ChineseCheckersPlayer::White
		],
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
			hash:0,
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
		game.hash=game.compute_zobrist();
		game
	}
	pub fn empty() -> Self {
		let mut s = Self {
			red: ChineseCheckerBoard::EMPTY,
			blue: ChineseCheckerBoard::EMPTY,
			green: ChineseCheckerBoard::EMPTY,
			yellow: ChineseCheckerBoard::EMPTY,
			black: ChineseCheckerBoard::EMPTY,
			white: ChineseCheckerBoard::EMPTY,
			nb_players:0,
			hash:0,
			turn: ChineseCheckersPlayer::Red
		};
		s.hash=s.compute_zobrist();
		s
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

	pub fn is_empty_at_index(&self, index:usize) -> bool {
		!self.all().get_at_index(index)
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
pub static JUMPS: [ChineseCheckerBoard;ChineseCheckerBoard::NB_SQUARES] = ChineseCheckerBoard::generate_jump_table();
pub static MIDDLE_TABLE: [[u16;ChineseCheckerBoard::NB_SQUARES];ChineseCheckerBoard::NB_SQUARES] = ChineseCheckerBoard::generate_middle_table();
impl ChineseCheckers {
	fn jumps_from(&self, from: usize, empty: &ChineseCheckerBoard, visited: &mut ChineseCheckerBoard) {
		visited.set_at_index(from);

		let jumps_board = JUMPS[from];

		for to_bit in jumps_board.iter_bits() {
			let to_idx = to_bit as usize;
			let mid_idx = MIDDLE_TABLE[from][to_idx] as usize;
			//let mid_idx = ChineseCheckerBoard::compute_middle(from.0, from.1, tx, ty, to_idx, from) as usize;

			if !visited.get_at_index(to_idx) && !empty.get_at_index(mid_idx) && empty.get_at_index(to_idx) {
				self.jumps_from(to_idx, empty, visited);
			}
		}
	}

	pub fn generate_moves(&self, moves: &mut Vec<Move>) {
		self.generate_moves_for_player(self.turn, moves)
	}
	pub fn generate_moves_for_player(&self, player: ChineseCheckersPlayer, moves: &mut Vec<Move>) {
		let board = self.board(player);
		let unowned = ChineseCheckerBoard::unowned_zones_board(player);
		// for home reenter restriction
		let home = ChineseCheckerBoard::initial_zone(player);
		let empty = !self.all();
		for bit in board.iter_bits() {
			// Simple Moves
			let neighbours = ChineseCheckerBoard::NEIGHBOURS[bit as usize] & !unowned;
			for n_bit in neighbours.iter_bits() {
				if empty.get_at_index(n_bit as usize) &&
					(home.get_at_index(bit as usize) || !home.get_at_index(n_bit as usize)) {
					moves.push(Move{from:bit as u8, to:n_bit as u8});
				}
			}

			// Jump moves
			let mut visited = ChineseCheckerBoard::EMPTY;
			self.jumps_from(bit as usize, &empty, &mut visited);
			for jump_to in visited.iter_bits() {
				if !unowned.get_at_index(jump_to as usize) && bit != jump_to &&
					(home.get_at_index(bit as usize) || !home.get_at_index(jump_to as usize)) {
					moves.push(Move{from:bit as u8, to:jump_to as u8});
				}
			}
		}
	}
}
impl ChineseCheckers {

	pub fn check_legality(
		&mut self,
		player: ChineseCheckersPlayer,
		mv: Move,
	) -> Result<(), &'static str> {
		if !ChineseCheckerBoard::is_playable_index(mv.from) {
			return Err("Invalid from square");
		}
		if !ChineseCheckerBoard::is_playable_index(mv.to) {
			return Err("Invalid to square");
		}

		let all_before = self.all();
		if all_before.get_at_index(mv.to as usize) {
			return Err("to square is not empty");
		}

		let board = self.board_mut(player);

		if !board.get_at_index(mv.from as usize) {
			return Err("No Pawn on from square");
		}

		Ok(())
	}
	fn has_won(&self, player: &ChineseCheckerBoard, target: &ChineseCheckerBoard) -> bool {
		let outside_target = player.clone() & !target.clone();
		//println!("Pions hors cible: {:#?}", outside_target.to_u128());
		outside_target == ChineseCheckerBoard::EMPTY
	}
}

impl ChineseCheckers {
	pub fn winner(&self) -> Option<ChineseCheckersPlayer> {
		let active = Self::active_players(self.nb_players);

		for &player in active {
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

		self.check_legality(self.turn, mv)?;
		self.play_unchecked(mv);

		Ok(())
	}
	pub fn play_unchecked(&mut self, mv: Move) {
		self.play_unchecked_for_player(self.turn, mv);

	}
	pub fn undo_unchecked(&mut self, mv: Move) {
		let p = self.turn;
		self.previous_turn();
		self.update_hash_turn(self.turn, p);
		self.undo_unchecked_for_player(self.turn, mv);
		self.update_hash(self.turn, &mv);

	}
	pub(crate) fn play_unchecked_for_player(&mut self, p: ChineseCheckersPlayer, mv: Move) {
		let board = self.board_mut(p);
		board.reset_at_index(mv.from as usize);
		board.set_at_index(mv.to as usize);
		self.update_hash(p, &mv);
		self.next_turn();
		self.update_hash_turn(p, self.turn);
	}
	pub(crate) fn undo_unchecked_for_player(&mut self, p: ChineseCheckersPlayer, mv: Move) {
		let board = self.board_mut(p);
		board.set_at_index(mv.from as usize);
		board.reset_at_index(mv.to as usize);
	}
	
	fn next_turn(&mut self) {
		let active = Self::active_players(self.nb_players);
		let current_index = active.iter().position(|p| *p == self.turn).unwrap();
		let next_index = (current_index + 1) % active.len();
		self.turn = active[next_index];
	}
	fn previous_turn(&mut self) {
		let active = Self::active_players(self.nb_players);
		let current_index = active.iter().position(|p| *p == self.turn).unwrap();
		let previous_index = (current_index + active.len() - 1) % active.len();
		self.turn = active[previous_index];
	}
}
impl fmt::Display for ChineseCheckers {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for y in 0..17 {

			for x in 0..13 {
				if !ChineseCheckerBoard::is_playable(x, y) {
					write!(f, "  ")?;
					continue;
				}
				if y % 2 == 0 {
					write!(f, " ")?;
				}
				let c = if self.red.get(x, y) { 'R' }
						else if self.blue.get(x, y) { 'B' }
						else if self.green.get(x, y) { 'G' }
						else if self.yellow.get(x, y) { 'Y' }
						else if self.black.get(x, y) { 'b' }
						else if self.white.get(x, y) { 'w' }
						else { '.' };

				write!(f, "{}", c)?;
				if y % 2 == 1 {
					write!(f, " ")?;
				}
			}
			writeln!(f)?;
		}
		Ok(())
	}
}

pub struct Zobrist {
	pub pieces: [[u64; 6]; ChineseCheckerBoard::NB_SQUARES],
	pub turn: [u64; 6],
}

impl Zobrist {
	pub const fn new(seed: u64) -> Self {
		let mut rng = crate::utils::Rng::from_seed(seed);
		let mut pieces = [[0u64; 6]; ChineseCheckerBoard::NB_SQUARES];
		let mut turn = [0u64; 6];
		
		let mut i = 0;
		while i < ChineseCheckerBoard::NB_SQUARES {
			pieces[i][0] = rng.u64();
			pieces[i][1] = rng.u64();
			pieces[i][2] = rng.u64();
			pieces[i][3] = rng.u64();
			pieces[i][4] = rng.u64();
			pieces[i][5] = rng.u64();
			i += 1;
		}
		turn[0] = rng.u64();
		turn[1] = rng.u64();
		turn[2] = rng.u64();
		turn[3] = rng.u64();
		turn[4] = rng.u64();
		turn[5] = rng.u64();
		
		Self {
			pieces,
			turn,
		}
	}
}

impl ChineseCheckers {
	pub const ZOBRIST_KEYS: Zobrist = Zobrist::new(0xA19F784);
	fn compute_zobrist(&self) -> u64 {
		let mut h = 0u64;
		for i in self.red.iter_bits() { h ^= Self::ZOBRIST_KEYS.pieces[i as usize][0]; }
		for i in self.blue.iter_bits() { h ^= Self::ZOBRIST_KEYS.pieces[i as usize][1]; }
		for i in self.green.iter_bits() { h ^= Self::ZOBRIST_KEYS.pieces[i as usize][2]; }
		for i in self.yellow.iter_bits() { h ^= Self::ZOBRIST_KEYS.pieces[i as usize][3]; }
		for i in self.black.iter_bits() { h ^= Self::ZOBRIST_KEYS.pieces[i as usize][4]; }
		for i in self.white.iter_bits() { h ^= Self::ZOBRIST_KEYS.pieces[i as usize][5]; }
		h ^= Self::ZOBRIST_KEYS.turn[self.turn.idx() as usize];
		h
	}
	fn update_hash(&mut self, p: ChineseCheckersPlayer, m: &Move) {
		self.hash ^= Self::ZOBRIST_KEYS.pieces[m.from as usize][p.idx() as usize];
		self.hash ^= Self::ZOBRIST_KEYS.pieces[m.to as usize][p.idx() as usize];
	}
	fn update_hash_turn(&mut self, previous: ChineseCheckersPlayer, next:ChineseCheckersPlayer) {
		self.hash ^= Self::ZOBRIST_KEYS.turn[previous.idx() as usize];
		self.hash ^= Self::ZOBRIST_KEYS.turn[next.idx() as usize];
	}
}
#[cfg(test)]
mod tests {
	use super::{ChineseCheckers, ChineseCheckersPlayer, Move};
	use super::bitboard::ChineseCheckerBoard;
	use crate::common::{GameResult, gui::BoardGame};
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
	fn test_play() {
		let mut game = ChineseCheckers::new(6);
		let mut i = 0;
		let mut rng = crate::utils::Rng::new();
		while i < 10000 && game.winner().is_none() {
			let moves = game.legal_moves();
			let m = moves[rng.range(0, moves.len())];
			game.play_unchecked(m);
			assert_eq!(game.hash, game.compute_zobrist());
			game.undo_unchecked(m);
			assert_eq!(game.hash, game.compute_zobrist());
			game.play_unchecked(m);
			//println!("{}", game.hash);
			i+=1;
		}
		println!("{}", game);
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
		
		let mut moves=vec![];
		game.generate_moves_for_player(ChineseCheckersPlayer::Red, &mut moves);

		let from = ChineseCheckerBoard::index_from_coords(6, 8);
		let to   = ChineseCheckerBoard::index_from_coords(8, 8);

		let jump_move = Move { from: from as u8, to: to as u8 };

		assert!(
			moves.contains(&jump_move),
			"Simple jump error ! Moves: {:?}",
			moves
		);
	}
	#[test]
	fn test_over() {
		let mut game = ChineseCheckers::from_fen(
	r##"turn=R
             B            
            B B           
           . B B          
          B B B B         
 . . . . B . . . Y . Y Y Y
  . . . . . . . . . Y Y Y 
   . . . . . . . . . Y Y  
    . . . . . . . . . Y   
     G . . . . . . . .    
    . . . . . . . . . .   
   G G . . . . . . . . .  
  G G G . . . . . . . . . 
 G G G G . . . . R . . . .
          R R R R         
           R R .          
            B R           
             R            
"##).unwrap();
		assert!(game.result() == GameResult::OnGoing);
		let legals = game.legal_moves();
		println!("{legals:?}");
		let _=game.play(Move{from:164, to: 189});
		println!("{}", game);
		println!("{:?}", game.result());
		assert!(game.result() == GameResult::Player(0));
		
	}
}