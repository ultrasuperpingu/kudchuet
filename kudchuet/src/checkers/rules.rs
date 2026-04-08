#![allow(clippy::uninlined_format_args)]
#![allow(clippy::too_many_arguments)]

use std::fmt::{Debug, Display};
use std::hash::Hash;

use bitboard::{BitIter, Bitboard};

use crate::checkers::bitboards::Bitboard5x10Checkers10;
use crate::common::Player;

pub type Coord = (u8, u8);
pub type BitPos = u8;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Cell {
	Empty,
	WhitePawn,
	BlackPawn,
	WhiteQueen,
	BlackQueen,
}
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub struct Move {
	pub(crate) start:u8,
	pub(crate) dest:u8,
	pub(crate) pawn_takes:Bitboard5x10Checkers10,
	pub(crate) queen_takes:Bitboard5x10Checkers10
}
impl Default for Move {
	fn default() -> Self {
		Move {
			start: 0, dest: 0,
			pawn_takes: Bitboard5x10Checkers10::EMPTY,
			queen_takes: Bitboard5x10Checkers10::EMPTY
		}
	}
}
impl Display for Move {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_simple() {
			write!(f, "{}-{}", self.start, self.dest)
		} else {
			write!(f, "{}x{}", self.start, self.dest)
		}
	}
}
impl Move {
	#[inline(always)]
	pub fn from(&self) -> u8{
		self.start
	}
	#[inline(always)]
	pub fn to(&self) -> u8 {
		self.dest
	}
	#[inline(always)]
	pub fn is_take(&self) -> bool {
		!self.is_simple()
	}
	#[inline(always)]
	pub fn is_simple(&self) -> bool {
		self.pawn_takes.is_empty() && self.queen_takes.is_empty()
	}
	#[inline(always)]
	pub fn simple_from_manoury(start: u8, dest: u8) -> Self {
		Self { start:start-1, dest:dest-1, pawn_takes: Bitboard5x10Checkers10::EMPTY, queen_takes: Bitboard5x10Checkers10::EMPTY }
	}
	#[inline(always)]
	pub fn take_from_indices(start: u8, dest: u8, board: Checkers10) -> Option<Self> {
		board.legal_moves_for_piece(start-1).into_iter().find(|m| m.to() == dest)
	}
	#[inline(always)]
	pub fn nb_takes(&self) -> u32 {
		(self.pawn_takes|self.queen_takes).count()
	}
	#[inline]
	fn merge(&self, m: &Move) -> Move {
		debug_assert_eq!(self.dest, m.start);
		debug_assert!(m.is_take() && self.is_take());

		let new_pawn_takes = self.pawn_takes | m.pawn_takes;
		let new_queen_takes = self.queen_takes | m.queen_takes;


		Self {
			start: self.start,
			dest: m.dest,
			pawn_takes: new_pawn_takes,
			queen_takes: new_queen_takes,
		}
	}

}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Rules {
	pub queen_long_moves: bool,
	pub first_player: Player,
	pub pawns_takes_backward: bool,
	pub queen_en_passant: bool,
	pub maximal_take_mandatory: bool,
	pub maximal_queens_take_mandatory: bool,
	pub force_takes: bool,
	pub allow_no_takes: bool,
	pub huffing_takes: bool,
}
impl Default for Rules {
	fn default() -> Self {
		Self::international()
	}
}
impl Rules {
	#[inline(always)]
	pub fn international() -> Self {
		Self {
			queen_long_moves: true,
			first_player: Player::PLAYER1,
			pawns_takes_backward: true,
			queen_en_passant: false,
			maximal_take_mandatory: true,
			maximal_queens_take_mandatory: false,
			force_takes: true,
			allow_no_takes: false,
			huffing_takes: false,
		}
	}
	#[inline(always)]
	pub fn english_draughts() -> Self {
		Self {
			queen_long_moves: false,
			first_player: Player::PLAYER2,
			pawns_takes_backward: false,
			queen_en_passant: false,
			maximal_take_mandatory: false,
			maximal_queens_take_mandatory: false,
			force_takes: true,
			allow_no_takes: true,
			huffing_takes: false,
		}
	}
}
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct Checkers10
{
	pub(super) current_player:Player,
	pub(super) white_pawns: Bitboard5x10Checkers10,
	pub(super) black_pawns: Bitboard5x10Checkers10,
	pub(super) white_queens: Bitboard5x10Checkers10,
	pub(super) black_queens: Bitboard5x10Checkers10,
	pub(super) rules: Rules
}
impl Default for Checkers10 {
	fn default() -> Self {
		#[allow(clippy::let_unit_value)]
		
		let mut white_pawns = 0u64;
		let mut black_pawns = 0u64;

		let pawns_per_player = Bitboard5x10Checkers10::WIDTH * (Bitboard5x10Checkers10::HEIGHT / 2 - 1) ;
		println!("{}", pawns_per_player);
		let mut count = 0;
		for row in 0..Bitboard5x10Checkers10::HEIGHT {
			for col in 0..Bitboard5x10Checkers10::WIDTH*2 {
				if let Some(index) = Self::coords_to_index(col, row) {
					if count < pawns_per_player {
						black_pawns |= 1u64 << index;
						count += 1;
					}
				}
			}
		}

		let mut count = 0;
		for row in (0..Bitboard5x10Checkers10::HEIGHT).rev() {
			for col in 0..Bitboard5x10Checkers10::WIDTH*2 {
				if let Some(index) = Self::coords_to_index(col, row) {
					if count < pawns_per_player {
						white_pawns |= 1u64 << index;
						count += 1;
					}
				}
			}
		}

		Self {
			current_player: Player::PLAYER1,
			white_pawns: Bitboard5x10Checkers10(white_pawns),
			black_pawns: Bitboard5x10Checkers10(black_pawns),
			white_queens: Bitboard5x10Checkers10(0),
			black_queens: Bitboard5x10Checkers10(0),
			rules: Rules::default()
		}
	}
}
impl Display for Checkers10 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "  ")?;
		for c in 0..Bitboard5x10Checkers10::WIDTH {
			write!(f, "  {:2}", c+1)?;
		}
		writeln!(f)?;
		for row in 0..Bitboard5x10Checkers10::HEIGHT {
			if row % 2 == 1 {
				write!(f, "{:2}", (row)*Bitboard5x10Checkers10::WIDTH+1)?;
			} else {
				write!(f, "  ")?;
			}
			for col in 0..Bitboard5x10Checkers10::WIDTH*2 {
				if (row+col) % 2 == 1 {
					let char = match self.cell_from_coords(col, row) {
						Cell::Empty => ' ',
						Cell::WhitePawn => 'o',
						Cell::BlackPawn => 'x',
						Cell::WhiteQueen => 'O',
						Cell::BlackQueen => 'X',
					};
					write!(f, "|{}", char)?;
				} else {
					write!(f, "| ")?;
				}
			}
			if row % 2 == 0 {
				writeln!(f, "|{}", (row+1)*Bitboard5x10Checkers10::WIDTH)?;
			} else {
				writeln!(f, "|")?;
			}
			
		}
		write!(f, "  ")?;
		for c in Bitboard5x10Checkers10::WIDTH * (Bitboard5x10Checkers10::HEIGHT-1)..Bitboard5x10Checkers10::WIDTH * (Bitboard5x10Checkers10::HEIGHT-1)+Bitboard5x10Checkers10::WIDTH {
			write!(f, " {:2} ", c+1)?;
		}
		writeln!(f)
	}
}
pub fn international() -> Checkers10 {
	Checkers10::default()
}

impl Checkers10
{
	pub fn empty() -> Self {
		Self {
			current_player: Player::PLAYER1,
			white_pawns: Bitboard5x10Checkers10::EMPTY,
			black_pawns: Bitboard5x10Checkers10::EMPTY,
			white_queens: Bitboard5x10Checkers10::EMPTY,
			black_queens: Bitboard5x10Checkers10::EMPTY,
			rules: Rules::international(),
		}
	}
	pub fn set_cell_from_index(&mut self, index: u8, piece: Cell) {
		match piece {
			Cell::Empty => {
				self.white_pawns.reset_at_index(index as usize);
				self.black_pawns.reset_at_index(index as usize);
				self.white_queens.reset_at_index(index as usize);
				self.black_queens.reset_at_index(index as usize);
			},
			Cell::WhitePawn => {
				self.white_pawns.set_at_index(index as usize);
				self.black_pawns.reset_at_index(index as usize);
				self.white_queens.reset_at_index(index as usize);
				self.black_queens.reset_at_index(index as usize);
			},
			Cell::BlackPawn => {
				self.white_pawns.reset_at_index(index as usize);
				self.black_pawns.set_at_index(index as usize);
				self.white_queens.reset_at_index(index as usize);
				self.black_queens.reset_at_index(index as usize);
			},
			Cell::WhiteQueen => {
				self.white_pawns.reset_at_index(index as usize);
				self.black_pawns.reset_at_index(index as usize);
				self.white_queens.set_at_index(index as usize);
				self.black_queens.reset_at_index(index as usize);
			},
			Cell::BlackQueen => {
				self.white_pawns.reset_at_index(index as usize);
				self.black_pawns.reset_at_index(index as usize);
				self.white_queens.reset_at_index(index as usize);
				self.black_queens.set_at_index(index as usize);
			},
		}
	}
	pub fn coords_to_index(x: u8, y: u8) -> Option<u8> {
		if (x + y).is_multiple_of(2) {
			return None;
		}

		let darks_per_row = Bitboard5x10Checkers10::WIDTH;

		let index = if y.is_multiple_of(2) {
			(x - 1) / 2
		} else {
			x / 2
		};

		Some(y * darks_per_row + index)
	}
	#[inline(always)]
	pub fn index_to_coords(index: u8) -> (u8, u8) {
		let darks_per_row = Bitboard5x10Checkers10::WIDTH;

		let y = index / darks_per_row;
		let index_in_row = index % darks_per_row;

		let x = if y.is_multiple_of(2) {
			// On even rows, dark cells are odd: 1,3,5,...
			1 + index_in_row * 2
		} else {
			// On odd rows, dark cells are even = 0,2,4,...
			index_in_row * 2
		};

		(x, y)
	}

	#[inline(always)]
	pub fn cell_from_coords(&self, x:u8, y:u8) -> Cell {
		if let Some(index) = Self::coords_to_index(x, y) {
			self.cell_from_index(index)
		} else {
			Cell::Empty
		}
	}
	#[inline(always)]
	pub fn cell_from_index(&self, index: u8) -> Cell {
		if self.white_pawns.get_at_index(index as usize) {
			Cell::WhitePawn
		} else if self.black_pawns.get_at_index(index as usize) {
			Cell::BlackPawn
		} else if self.white_queens.get_at_index(index as usize) {
			Cell::WhiteQueen
		} else if self.black_queens.get_at_index(index as usize) {
			Cell::BlackQueen
		} else {
			Cell::Empty
		}
	}
	#[inline(always)]
	pub fn player_turn(&self) -> Player {
		self.current_player
	}
	#[inline]
	pub fn play_unchecked(&mut self, m: &Move) {
		debug_assert!(self.white_pawns.0 & self.black_pawns.0 & self.white_queens.0 & self.black_queens.0 == 0);

		self.move_piece(m.start, m.dest);

		// promotion
		self.try_promote();

		// remove captured pieces
		match self.current_player {
			Player::PLAYER1 => {
				self.black_pawns &= !m.pawn_takes;
				self.black_queens &= !m.queen_takes;
			}
			Player::PLAYER2 => {
				self.white_pawns &= !m.pawn_takes;
				self.white_queens &= !m.queen_takes;
			}
			_ => unreachable!()
		}

		self.current_player = self.current_player.opponent();
	}

	#[inline]
	fn try_promote(&mut self) {
		
		match self.current_player {
			Player::PLAYER1 => {
				let promoted_white = self.white_pawns & Bitboard5x10Checkers10::SOUTH_BORDER;
				self.white_queens |= promoted_white;
				self.white_pawns &= !promoted_white;
			}
			Player::PLAYER2 => {
				let promoted_black = self.black_pawns & Bitboard5x10Checkers10::NORTH_BORDER;
				self.black_queens |= promoted_black;
				self.black_pawns &= !promoted_black;
			}
			_ => unreachable!()
		}
	}

	pub fn legal_moves(&self) -> Vec<Move> {
		let mut moves = vec![];
		for index in self.self_pieces().iter_bits() {
			self.legal_moves_for_piece_inplace(&mut moves, index as u8);
		}
		self.filter_moves(&mut moves);
		moves
	}
	pub fn legal_moves_for_piece(&self, cell_index:u8) -> Vec<Move> {
		let mut moves = vec![];
		self.legal_moves_for_piece_inplace(&mut moves, cell_index);
		moves
	}
	pub fn legal_moves_for_piece_inplace(&self, moves: &mut Vec<Move>, cell_index:u8) {
		debug_assert!(self.white_pawns.0 & self.black_pawns.0 & self.white_queens.0 & self.black_queens.0 == 0);
		let full_mask = self.all_pieces();
		if self.current_player == Player::PLAYER1 {
			if self.white_pawns.get_at_index(cell_index as usize) {
				self.pawn_moves(cell_index, moves, full_mask,self.black_pawns, self.black_queens, -1);
			} else {
				self.queen_moves(cell_index, moves, full_mask, self.white_queens, self.black_pawns, self.black_queens);
			}
		} else { //if self.current_player == Player::Black
			if self.black_pawns.get_at_index(cell_index as usize) {
				self.pawn_moves(cell_index, moves, full_mask, self.white_pawns, self.white_queens, 1);
			} else {
				self.queen_moves(cell_index, moves, full_mask, self.black_queens, self.white_pawns, self.white_queens);
			}
		}
	}

	#[inline]
	fn pawn_moves(&self,
		cell_index: u8,
		moves: &mut Vec<Move>,
		all_pieces: Bitboard5x10Checkers10,
		opponent_pawns: Bitboard5x10Checkers10,
		opponent_queens: Bitboard5x10Checkers10,
		advance_way:i8)
	{
		let before_count = moves.len();
		let already_taken = Bitboard5x10Checkers10::EMPTY;
		self.pawn_takes_moves(cell_index, cell_index, moves,all_pieces, opponent_pawns, opponent_queens, advance_way, already_taken);
		if before_count < moves.len() && !self.rules.allow_no_takes {
			return;
		}
		// Simple pawn moves
		let adv_mask = if advance_way > 0 {
			Bitboard5x10Checkers10::BLACK_ADVANCE[cell_index as usize]
		} else {
			Bitboard5x10Checkers10::WHITE_ADVANCE[cell_index as usize]
		};
		for dest in (adv_mask&!all_pieces).iter_bits() {
			moves.push(Move { start: cell_index, dest: dest as u8, pawn_takes: Bitboard5x10Checkers10::EMPTY, queen_takes: Bitboard5x10Checkers10::EMPTY });
		}
	}
	
	fn pawn_takes_moves(
		&self,
		start_cell_index: u8,
		cell_index: u8,
		moves: &mut Vec<Move>,
		all_pieces: Bitboard5x10Checkers10,
		opponent_pawns: Bitboard5x10Checkers10,
		opponent_queens: Bitboard5x10Checkers10,
		advance_way: i8,
		already_taken: Bitboard5x10Checkers10,
	) {
		let mut local_takes = [Move::default(); 4];
		let take_count = Self::pawn_one_take_moves(
				start_cell_index,
				cell_index,
				all_pieces,
				opponent_pawns,
				opponent_queens,
				if self.rules.pawns_takes_backward {0..4} else if advance_way < 0 { 0..2 } else {2..4 },
				already_taken,
				&mut local_takes,
			);

		if take_count == 0 {
			return;
		}

		for i in 0..take_count {
			let m = local_takes[i];
			let mut next_taken  = already_taken.clone();

			next_taken |= m.pawn_takes|m.queen_takes;
			let before_count = moves.len();
			self.pawn_takes_moves(
				start_cell_index,
				m.to(),
				moves,
				all_pieces,
				opponent_pawns,
				opponent_queens,
				advance_way,
				next_taken,
			);

			if moves.len() == before_count {
				moves.push(m);
			} else {
				for j in before_count..moves.len() {
					moves[j] = m.merge(&moves[j]);
				}
			}
		}

		//self.filter_moves(moves);
	}

	#[inline]
	fn pawn_one_take_moves(
		start_cell_index: u8,
		cell_index: u8,
		all_pieces: Bitboard5x10Checkers10,
		opponent_pawns: Bitboard5x10Checkers10,
		opponent_queens: Bitboard5x10Checkers10,
		range: std::ops::Range<usize>,
		already_taken: Bitboard5x10Checkers10,
		takes_out: &mut [Move; 4]
	) -> usize {
		let mut nb_takes = 0;
		let jumps = Bitboard5x10Checkers10::JUMPS[cell_index as usize];
		for i in range {
			let jump = jumps[i];
			if jump.1 == Bitboard5x10Checkers10::NB_SQUARES as u8 { continue; }
			if ((jump.0 & all_pieces).is_empty() || start_cell_index == jump.1) && !already_taken.get_at_index(jump.2 as usize){
				let is_queen = opponent_queens.get_at_index(jump.2 as usize);
				let is_pawn = opponent_pawns.get_at_index(jump.2 as usize);
				if is_queen || is_pawn {

					takes_out[nb_takes] = Move {
						start: cell_index,
						dest: jump.1,
						pawn_takes: if is_pawn { Bitboard5x10Checkers10::from_index(jump.2 as usize)} else { Bitboard5x10Checkers10::EMPTY },
						queen_takes: if is_queen { Bitboard5x10Checkers10::from_index(jump.2 as usize)} else { Bitboard5x10Checkers10::EMPTY },
					};
					nb_takes += 1;
				}
			}
		}
		nb_takes
	}

	fn queen_moves(&self, cell_index: u8,
		moves: &mut Vec<Move>,
		all_pieces: Bitboard5x10Checkers10,
		self_queens: Bitboard5x10Checkers10,
		opponent_pawns: Bitboard5x10Checkers10,
		opponent_queens: Bitboard5x10Checkers10) {
		if self_queens.get_at_index(cell_index as usize) {
			let already_taken = Bitboard5x10Checkers10::EMPTY;
			let before_count = moves.len();
			self.queen_takes_moves(cell_index, cell_index, moves, all_pieces, opponent_pawns, opponent_queens, already_taken);
			let took = before_count < moves.len();
			
			let need_eval_simple = !took || self.rules.allow_no_takes;
			if need_eval_simple {
				if self.rules.queen_long_moves {
					self.queen_simple_moves_long(cell_index, moves, all_pieces);
				} else {
					self.queen_simple_moves_short(cell_index, moves, all_pieces);
				}
			}
		}
	}
	#[inline]
	fn queen_simple_moves_long(
		&self,
		cell_index: u8,
		moves: &mut Vec<Move>,
		all_pieces: Bitboard5x10Checkers10,
	) {
		let sq = cell_index as usize;
		// On all 4 directions
		for ray in [
			Bitboard5x10Checkers10::RAY_NE[sq], Bitboard5x10Checkers10::RAY_SE[sq], 
			Bitboard5x10Checkers10::RAY_NW[sq], Bitboard5x10Checkers10::RAY_SW[sq]
		] {
			let blockers = ray & all_pieces;
			
			let landing_mask = if blockers.is_empty() {
				ray
			} else {
				let is_forward = ray == Bitboard5x10Checkers10::RAY_NE[sq] || ray == Bitboard5x10Checkers10::RAY_SE[sq];
				let first_blocker = if is_forward {
					blockers.lsb() as usize
				} else {
					blockers.msb() as usize
				};
				Bitboard5x10Checkers10::BETWEEN_MASKS[sq][first_blocker]
			};

			for dest in landing_mask.iter_bits() {
				moves.push(Move { start: cell_index, dest: dest as u8, ..Default::default() });
			}
		}
	}
	#[inline]
	fn queen_simple_moves_short(
		&self,
		cell_index: u8,
		moves: &mut Vec<Move>,
		all_pieces: Bitboard5x10Checkers10,
	) {
		for dest in (Bitboard5x10Checkers10::QUEEN_NEIGHBORS[cell_index as usize]&!all_pieces).iter_bits() {
			moves.push(Move { start: cell_index, dest: dest as u8, ..Default::default() });
		}
	}
	fn queen_takes_moves(
		&self,
		start_cell_index: u8,
		cell_index: u8,
		moves: &mut Vec<Move>,
		all_pieces: Bitboard5x10Checkers10,
		opponent_pawns: Bitboard5x10Checkers10,
		opponent_queens: Bitboard5x10Checkers10,
		already_taken: Bitboard5x10Checkers10,
	) {
		let mut local_takes = [Move { start: 0, dest: 0, ..Default::default() }; 16];
		let take_count = Self::queen_one_take_moves(
			start_cell_index,
			cell_index,
			all_pieces,
			opponent_pawns | opponent_queens,
			opponent_queens,
			already_taken,
			&mut local_takes
		);
		

		if take_count == 0 {
			return;
		}

		for i in 0..take_count {
			let m = local_takes[i];
			// update already taken
			let mut next_taken = already_taken;
			// move was generated by queen_one_take_moves so (*pawn_takes|*queen_takes).count() == 1
			debug_assert!((m.pawn_takes|m.queen_takes).count() == 1);
			next_taken.set_at_index((m.pawn_takes|m.queen_takes).lsb() as usize);
			
			let dest = m.to();
			let before_count = moves.len();
			// recurse
			self.queen_takes_moves(
				start_cell_index,
				dest,
				moves,
				all_pieces,
				opponent_pawns,
				opponent_queens,
				next_taken,
			);

			if moves.len() == before_count {
				moves.push(m);
			} else {
				for j in before_count..moves.len() {
					moves[j]=m.merge(&moves[j]);
				}
			}
		}
	}

	#[inline]
	fn queen_one_take_moves(
		start_cell_index: u8,
		cell_index: u8,
		all_pieces: Bitboard5x10Checkers10,
		opponent_all: Bitboard5x10Checkers10,
		opponent_queens: Bitboard5x10Checkers10,
		already_taken: Bitboard5x10Checkers10,
		takes: &mut [Move; 16]
	) -> usize {
		let mut takes_count = 0;
		let sq = cell_index as usize;

		let directions = [
			(Bitboard5x10Checkers10::RAY_NE, false),
			(Bitboard5x10Checkers10::RAY_SE, true),
			(Bitboard5x10Checkers10::RAY_NW, false),
			(Bitboard5x10Checkers10::RAY_SW, true),
		];

		for (ray_table, is_decreasing_ray) in directions {
			let ray = ray_table[sq];
			let blockers = ray & all_pieces;
			
			if blockers.is_empty() { continue; }

			let first_blocker_idx = if is_decreasing_ray {
				blockers.0.trailing_zeros() as usize
			} else {
				63 - blockers.0.leading_zeros() as usize
			};
			//println!("ray:\n{}\nblockers:\n{}\n{}", ray, blockers, first_blocker_idx);

			let victim = Bitboard5x10Checkers10::from_index(first_blocker_idx);

			if !(victim & opponent_all).is_empty() && (victim & already_taken).is_empty() {
				
				let ray_from_victim = ray_table[first_blocker_idx];

				let obstacles_behind = ray_from_victim & all_pieces &! Bitboard5x10Checkers10::from_index(start_cell_index as usize);
				
				let landing_mask = if obstacles_behind.is_empty() {
					ray_from_victim
				} else {
					let next_blocker_idx = if is_decreasing_ray {
						obstacles_behind.0.trailing_zeros() as usize
					} else {
						63 - obstacles_behind.0.leading_zeros() as usize
					};
					
					Bitboard5x10Checkers10::BETWEEN_MASKS[first_blocker_idx][next_blocker_idx]
				};

				for dest_idx in  landing_mask.iter_bits() {
					let dest_idx = dest_idx as u8;
					
					let (pawn_takes, queen_takes) = if opponent_queens.get_at_index(first_blocker_idx) {
						(Bitboard5x10Checkers10::EMPTY, Bitboard5x10Checkers10::from_index(first_blocker_idx))
					} else {
						(Bitboard5x10Checkers10::from_index(first_blocker_idx), Bitboard5x10Checkers10::EMPTY)
					};

					takes[takes_count] = Move {
						start: cell_index,
						dest: dest_idx,
						pawn_takes,
						queen_takes,
					};
					takes_count += 1;
				}
			}
		}

		takes_count
	}

	#[inline]
	fn filter_moves(&self, moves: &mut Vec<Move>) {
		if self.rules.allow_no_takes || moves.is_empty() {
			return;
		}
		let has_take = moves.iter().any(|m| m.is_take());
		if has_take {
			moves.retain(|m| m.is_take());

			if self.rules.maximal_take_mandatory {
				let max_len = moves.iter()
					.map(|m| m.nb_takes())
					.max()
					.unwrap_or(0);

				moves.retain(|m| (m.nb_takes()) == max_len);
			}
		}
	}

	#[inline]
	pub fn all_pieces(&self) -> Bitboard5x10Checkers10 {
		self.white_pawns | self.black_pawns | self.white_queens | self.black_queens
	}

	#[inline]
	pub fn blacks(&self) -> Bitboard5x10Checkers10 {
		self.black_pawns | self.black_queens
	}
	#[inline]
	pub fn whites(&self) -> Bitboard5x10Checkers10 {
		self.white_pawns | self.white_queens
	}


	fn self_pieces(&self) -> Bitboard5x10Checkers10 {
		if self.current_player == Player::PLAYER1 {
			self.whites()
		} else {
			self.blacks()
		}
	}
	fn move_piece(&mut self, start: u8, dest: u8) {
		if self.current_player == Player::PLAYER1 {
			if self.white_pawns.get_at_index(start as usize) {
				self.white_pawns.reset_at_index(start as usize);
				self.white_pawns.set_at_index(dest as usize);
			} else if self.white_queens.get_at_index(start as usize) {
				self.white_queens.reset_at_index(start as usize);
				self.white_queens.set_at_index(dest as usize);
			}
		} else {
			if self.black_pawns.get_at_index(start as usize) {
				self.black_pawns.reset_at_index(start as usize);
				self.black_pawns.set_at_index(dest as usize);
			} else if self.black_queens.get_at_index(start as usize) {
				self.black_queens.reset_at_index(start as usize);
				self.black_queens.set_at_index(dest as usize);
			}
		}
	}

	pub fn is_victory(&self) -> bool {
		if self.current_player == Player::PLAYER1 {
			self.whites().is_empty() || self.legal_moves().is_empty()
		} else {
			self.blacks().is_empty() || self.legal_moves().is_empty()
		}
	}
	pub fn is_loose(&self) -> bool {
		if self.current_player == Player::PLAYER2 {
			self.whites().is_empty() || self.legal_moves().is_empty()
		} else {
			self.blacks().is_empty() || self.legal_moves().is_empty()
		}
	}
	pub fn is_over(&self) -> bool {
		self.blacks().is_empty() || self.whites().is_empty() || self.legal_moves().is_empty()
	}
	
}


#[cfg(test)]
mod tests {
	//use std::hash::{DefaultHasher, Hash, Hasher};

	use bitboard::Bitboard;

use super::{Bitboard5x10Checkers10, Checkers10, Move};

	#[test]
	fn test_checkers() {
		let mut board = Checkers10::default();
		println!("{}", board);
		println!("{:?}", board.legal_moves_for_piece(30));
		board.play_unchecked(&&Move::simple_from_manoury(31, 26));
		println!("{}", board);
		board.play_unchecked(&&Move::simple_from_manoury(17, 21));
		println!("{}", board);
		println!("{:?}", board.legal_moves_for_piece(25));
	}
	#[test]
	fn test_checkers2() {
		let mut board = Checkers10 {
			current_player: super::Player::PLAYER1,
			white_pawns: Bitboard5x10Checkers10(0b00100_00000_00000_00000_00000_00000_00000_00000_00000_00000u64),
			black_pawns: Bitboard5x10Checkers10(0b00000_00100_00000_01000_00000_01110_00000_01110_00000_00000u64),
			white_queens: Bitboard5x10Checkers10(0),
			black_queens: Bitboard5x10Checkers10(0),
			..Default::default()
		};
		println!("{}", board);
		let mvs = board.legal_moves();
		println!("{:?}", mvs);
		if mvs.len() >= 1 {
			board.play_unchecked(&mvs[0]);
		}
		println!("{}", board);
	}
	#[test]
	fn test_checkers3() {
		let mut board = Checkers10 {
			current_player: super::Player::PLAYER1,
			white_pawns: Bitboard5x10Checkers10(0),
			black_pawns: Bitboard5x10Checkers10(0b00000_00011_00000_00000_00100_01000_00110_00000_01000_00000u64),
			white_queens: Bitboard5x10Checkers10(0b00000_00000_00100_00000_00000_00000_00000_00000_00000_00000u64),
			black_queens: Bitboard5x10Checkers10(0),
			..Default::default()
		};
		println!("{}", board);
		let mvs = board.legal_moves();
		println!("{:?}", mvs);
		if mvs.len() >= 1 {
			board.play_unchecked(&mvs[0]);
		}
		println!("{}", board);
	}
	#[test]
	fn test_bug() {
		
		let board = Checkers10::from_fen("W:WK50:B33,22,12,34,24,14").unwrap();
		println!("{}", board);
		let mvs = board.legal_moves();
		println!("{:?}", mvs);
		assert_eq!(mvs.len(), 2);
		assert_eq!(mvs[0].nb_takes(), 6);

		let board = Checkers10::from_fen("W:W47:B41,31,21,22,12,23,34").unwrap();
		println!("{}", board);
		let mvs = board.legal_moves();
		println!("{:?}", mvs);
		assert_eq!(mvs.len(), 1);
		assert_eq!(mvs[0].nb_takes(), 5);

	
		let mut board = Checkers10::from_fen("W:W41:B37,31,28,19,10").unwrap();
		println!("{}", board);
		let mvs = board.legal_moves();
		println!("{:?}", mvs);
		assert_eq!(mvs.len(), 1);
		assert_eq!(mvs[0].nb_takes(), 4);
		board.play_unchecked(&mvs[0]);
		assert!(board.white_queens.count() == 1);

		let mut board = Checkers10::from_fen("W:W41:B37,31,28,19,9,8").unwrap();
		println!("{}", board);
		let mvs = board.legal_moves();
		println!("{:?}", mvs);
		assert_eq!(mvs.len(), 1);
		assert_eq!(mvs[0].nb_takes(), 5);
		board.play_unchecked(&mvs[0]);
		assert!(board.white_queens.count() == 0);
		
	}
}
