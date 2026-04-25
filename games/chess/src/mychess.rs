use std::hash::Hash;

use bitboard::{BitIter, Bitboard};
use crate::bitboard::Bitboard8x8;

use super::rules::{Color, Square, Piece, CastlingRights, Move};

use kudchuet::Player;
use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

use kudchuet::GameOutcome;
use super::pext_tables;
//use super::magic_tables::{MagicEntry, ROOK_MOVES, ROOK_MAGICS};


#[derive(Copy, Clone, Debug, Hash)]
pub struct ChessBoard {
	pub(crate) whites:Bitboard8x8,
	pub(crate) blacks:Bitboard8x8,
	pub(crate) pawns:Bitboard8x8,
	pub(crate) rooks:Bitboard8x8,
	pub(crate) knights:Bitboard8x8,
	pub(crate) bishops:Bitboard8x8,
	pub(crate) queens:Bitboard8x8,
	pub(crate) kings:Bitboard8x8,
	pub(crate) turn: Color,
	pub(crate) castling_rights: CastlingRights,
	pub(crate) ep_square: Option<Square>,
	pub(crate) hash: u64,
}

impl Default for ChessBoard {
	fn default() -> Self {
		// Rangées de départ
		let rank1 = 0x00000000000000FFu64; // a1–h1
		let rank2 = 0x000000000000FF00u64; // a2–h2
		let rank7 = 0x00FF000000000000u64; // a7–h7
		let rank8 = 0xFF00000000000000u64; // a8–h8

		let mut board = ChessBoard {
			// Couleurs
			whites: Bitboard8x8::from_storage(rank1 | rank2),
			blacks: Bitboard8x8::from_storage(rank7 | rank8),

			// Types de pièces
			pawns: Bitboard8x8::from_storage(rank2 | rank7),
			rooks: Bitboard8x8::from_storage(
				(1u64 << 0) | (1u64 << 7) | (1u64 << 56) | (1u64 << 63)
			),
			knights: Bitboard8x8::from_storage(
				(1u64 << 1) | (1u64 << 6) | (1u64 << 57) | (1u64 << 62)
			),
			bishops: Bitboard8x8::from_storage(
				(1u64 << 2) | (1u64 << 5) | (1u64 << 58) | (1u64 << 61)
			),
			queens: Bitboard8x8::from_storage(
				(1u64 << 3) | (1u64 << 59)
			),
			kings: Bitboard8x8::from_storage(
				(1u64 << 4) | (1u64 << 60)
			),
			// Trait
			turn: Color::White,
			castling_rights: CastlingRights::default(),
			ep_square: None,
			hash: 0,
		};
		board.hash = board.compute_zobrist();
		board
	}
}

impl std::fmt::Display for ChessBoard {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for rank in (0..8).rev() {
			write!(f, "{}  ", rank + 1)?;

			for file in 0..8 {
				let sq = (rank * 8 + file) as u8;
				let bb = Bitboard8x8::from_index(sq.into());

				let c = if (self.pawns & bb).any() {
					if (self.whites & bb).any() { 'P' } else { 'p' }
				} else if (self.rooks & bb).any() {
					if (self.whites & bb).any() { 'R' } else { 'r' }
				} else if (self.knights & bb).any() {
					if (self.whites & bb).any() { 'N' } else { 'n' }
				} else if (self.bishops & bb).any() {
					if (self.whites & bb).any() { 'B' } else { 'b' }
				} else if (self.queens & bb).any() {
					if (self.whites & bb).any() { 'Q' } else { 'q' }
				} else if (self.kings & bb).any() {
					if (self.whites & bb).any() { 'K' } else { 'k' }
				} else {
					'.'
				};

				write!(f, "{} ", c)?;
			}

			writeln!(f)?;
		}

		writeln!(f, "\n   a b c d e f g h")?;
		writeln!(f, "Side to move: {:?}", self.turn)?;

		Ok(())
	}
}

impl ChessBoard {
	#[inline]
	pub fn turn(&self) -> Color {
		self.turn
	}
	#[inline]
	pub fn colors(&self, color: Color) -> Bitboard8x8 {
		match color {
			Color::White => self.whites,
			Color::Black => self.blacks,
		}
	}
	#[inline]
	pub fn pieces(&self, piece: Piece) -> Bitboard8x8 {
		match piece {
			Piece::Pawn => self.pawns,
			Piece::Rook => self.rooks,
			Piece::Knight => self.knights,
			Piece::Bishop => self.bishops,
			Piece::Queen => self.queens,
			Piece::King => self.kings,
		}
	}
	#[inline]
	pub fn all(&self) -> Bitboard8x8 {
		self.blacks | self.whites
	}
	#[inline]
	pub fn free(&self) -> Bitboard8x8 {
		!(self.blacks | self.whites)
	}
}

/*
#[inline(always)]
fn magic_index(entry: &MagicEntry, blockers: u64) -> usize {
	let blockers = blockers & entry.mask;
	let hash = blockers.wrapping_mul(entry.magic);
	(hash >> (64 - entry.index_bits)) as usize
}

fn get_rook_moves(square: Square, blockers: u64) -> Bitboard8x8 {
	let sq = square.0 as usize; 
	let entry = &ROOK_MAGICS[sq];
	let moves = ROOK_MOVES[sq];

	let relevant = blockers & entry.mask;
	let index = magic_index(entry, relevant);

	Bitboard8x8::from_storage(moves[index])
}

fn get_bishop_moves(square: Square, blockers: u64) -> Bitboard8x8 {
	let sq = square.0 as usize; 
	let entry = &BISHOP_MAGICS[sq];
	let moves = BISHOP_MOVES[sq];

	let relevant = blockers & entry.mask;
	let index = magic_index(entry, relevant);

	Bitboard8x8::from_storage(moves[index])
}*/
#[inline(always)]
fn get_rook_moves(square: Square, blockers: u64) -> Bitboard8x8 {
	let sq = square.0 as usize; 
	let mask = pext_tables::ROOK_MASKS[sq];
	let moves = pext_tables::ROOK_MOVES[sq];

	//let index = Bitboard8x8::from_storage(blockers).pext(&Bitboard8x8::from_storage(mask));
	let index = Bitboard8x8::from_storage(blockers).pext(&mask);

	//Bitboard8x8::from_storage(moves[index as usize])
	moves[index as usize]
}
#[inline(always)]
fn get_bishop_moves(square: Square, blockers: u64) -> Bitboard8x8 {
	let sq = square.0 as usize; 
	let mask = pext_tables::BISHOP_MASKS[sq];
	let moves = pext_tables::BISHOP_MOVES[sq];

	//let index = Bitboard8x8::from_storage(blockers).pext(&Bitboard8x8::from_storage(mask));
	let index = Bitboard8x8::from_storage(blockers).pext(&mask);

	//Bitboard8x8::from_storage(moves[index as usize])
	moves[index as usize]
}
impl ChessBoard {
	#[inline(always)]
	pub fn attacks_of<const IS_WHITE: bool>(
		&self,
		piece: Piece,
		square: Square,
		blockers: u64
	) -> Bitboard8x8 {
		match piece {
			Piece::Rook   => get_rook_moves(square, blockers),
			Piece::Bishop => get_bishop_moves(square, blockers),
			Piece::Queen  => get_rook_moves(square, blockers) | get_bishop_moves(square, blockers),
			Piece::Knight => Self::knight_attacks(square),
			Piece::King   => Self::king_attacks(square),
			Piece::Pawn   => Self::pawn_attacks::<IS_WHITE>(square),
		}
	}
}
impl ChessBoard {
	#[inline]
	pub fn compute_pins<const IS_WHITE: bool>(&self) -> (Bitboard8x8, [i8; 64]) {
		let mut pinned = Bitboard8x8::empty();
		let mut pin_dir = [0i8; 64];

		let king_sq = self.king_square::<IS_WHITE>();
		let king_idx = king_sq.0 as usize;

		let us = if IS_WHITE { self.whites } else { self.blacks };
		let enemy_rooks_queens =
			(if IS_WHITE { self.blacks } else { self.whites }) & (self.rooks | self.queens);
		let enemy_bishops_queens =
			(if IS_WHITE { self.blacks } else { self.whites }) & (self.bishops | self.queens);

		let all = self.all(); // Bitboard8x8

		// 4 directions rook-like
		Self::scan_ray(
			king_idx,
			&all,
			&us,
			&enemy_rooks_queens,
			&mut pinned,
			&mut pin_dir,
			direction_index(0, 1),  // N
			Bitboard8x8::ray_n_mask,
		);
		Self::scan_ray(
			king_idx,
			&all,
			&us,
			&enemy_rooks_queens,
			&mut pinned,
			&mut pin_dir,
			direction_index(0, -1), // S
			Bitboard8x8::ray_s_mask,
		);
		Self::scan_ray(
			king_idx,
			&all,
			&us,
			&enemy_rooks_queens,
			&mut pinned,
			&mut pin_dir,
			direction_index(1, 0),  // E
			Bitboard8x8::ray_e_mask,
		);
		Self::scan_ray(
			king_idx,
			&all,
			&us,
			&enemy_rooks_queens,
			&mut pinned,
			&mut pin_dir,
			direction_index(-1, 0), // W
			Bitboard8x8::ray_w_mask,
		);

		// 4 directions bishop-like
		Self::scan_ray(
			king_idx,
			&all,
			&us,
			&enemy_bishops_queens,
			&mut pinned,
			&mut pin_dir,
			direction_index(1, 1),  // NE
			Bitboard8x8::ray_ne_mask,
		);
		Self::scan_ray(
			king_idx,
			&all,
			&us,
			&enemy_bishops_queens,
			&mut pinned,
			&mut pin_dir,
			direction_index(-1, 1), // NW
			Bitboard8x8::ray_nw_mask,
		);
		Self::scan_ray(
			king_idx,
			&all,
			&us,
			&enemy_bishops_queens,
			&mut pinned,
			&mut pin_dir,
			direction_index(1, -1), // SE
			Bitboard8x8::ray_se_mask,
		);
		Self::scan_ray(
			king_idx,
			&all,
			&us,
			&enemy_bishops_queens,
			&mut pinned,
			&mut pin_dir,
			direction_index(-1, -1), // SW
			Bitboard8x8::ray_sw_mask,
		);

		(pinned, pin_dir)
	}

	#[inline]
	fn move_respects_pin(from: usize, to: usize, dir: i8) -> bool {
		let (fx, fy) = Bitboard8x8::coords_from_index(from);
		let (tx, ty) = Bitboard8x8::coords_from_index(to);

		let dx = (tx as i8 - fx as i8).signum();
		let dy = (ty as i8 - fy as i8).signum();

		direction_index(dx, dy) == dir || direction_index(-dx, -dy) == dir
	}
	#[inline]
	fn scan_ray(
		king_idx: usize,
		all: &Bitboard8x8,
		us: &Bitboard8x8,
		enemy_sliders: &Bitboard8x8,
		pinned: &mut Bitboard8x8,
		pin_dir: &mut [i8; 64],
		dir_code: i8,
		ray_mask_fn: fn(usize) -> Bitboard8x8,
	) {
		// Masque du rayon à partir du roi
		let ray_mask = ray_mask_fn(king_idx);
		// Pièces présentes sur ce rayon
		let ray_occ = ray_mask & *all;

		if ray_occ.is_empty() {
			return;
		}

		// On va parcourir les cases du rayon dans l'ordre à partir du roi.
		// Comme on n’a pas l’ordre garanti par les bits, on fait un petit
		// stepping linéaire à partir de king_idx en utilisant le mask.
		let (dx, dy) = dir_from_code(dir_code);
		let mut idx = king_idx as isize;

		let mut found_ally: Option<usize> = None;

		loop {
			idx = step_index(idx, dx, dy);
			if idx < 0 || idx >= Bitboard8x8::NB_SQUARES as isize {
				break;
			}
			let i = idx as usize;
			if !ray_mask.get_at_index(i) {
				break;
			}

			if all.get_at_index(i) {
				let sq_bb = Bitboard8x8::from_index(i);

				if !(sq_bb & *us).is_empty() {
					if found_ally.is_none() {
						found_ally = Some(i);
						continue;
					} else {
						break;
					}
				}

				if !(sq_bb & *enemy_sliders).is_empty() {
					if let Some(p) = found_ally {
						pinned.set_at_index(p);
						pin_dir[p] = dir_code;
					}
					break;
				}

				// autre pièce ennemie non-slider → bloque le rayon
				break;
			}
		}
	}
}

#[inline]
fn direction_index(dx: i8, dy: i8) -> i8 {
	match (dx, dy) {
		(1, 0) => 1,
		(-1, 0) => 2,
		(0, 1) => 3,
		(0, -1) => 4,
		(1, 1) => 5,
		(1, -1) => 6,
		(-1, 1) => 7,
		(-1, -1) => 8,
		_ => 0,
	}
}

#[inline]
fn dir_from_code(code: i8) -> (i8, i8) {
	match code {
		1 => (1, 0),
		2 => (-1, 0),
		3 => (0, 1),
		4 => (0, -1),
		5 => (1, 1),
		6 => (1, -1),
		7 => (-1, 1),
		8 => (-1, -1),
		_ => (0, 0),
	}
}

#[inline]
fn step_index(idx: isize, dx: i8, dy: i8) -> isize {
	let (x, y) = Bitboard8x8::coords_from_index(idx as usize);
	let nx = x as isize + dx as isize;
	let ny = y as isize + dy as isize;
	if nx < 0 || ny < 0 || nx >= Bitboard8x8::WIDTH as isize || ny >= Bitboard8x8::HEIGHT as isize {
		-1
	} else {
		Bitboard8x8::index_from_coords(nx as u8, ny as u8) as isize
	}
}


impl ChessBoard {
	pub fn legal_moves(&self) -> Vec<Move> {
		if self.turn == Color::White {
			self.legal_moves_template::<true>()
		} else {
			self.legal_moves_template::<false>()
		}
	}
	pub fn legal_moves_template<const IS_WHITE: bool>(&self) -> Vec<Move> {
		let mut array = [Move{ from: Square(0), to: Square(0), promotion: None }; 256];
		let mut len = 0;
		self.legal_moves_inplace::<IS_WHITE>(&mut array, &mut len);
		array[0..len].to_vec()
	}
	
	#[inline]
	pub fn legal_moves_inplace<const IS_WHITE: bool>(&self, out: &mut [Move; 256], len: &mut usize) {
		*len = 0;

		let (in_check, checkers, block_mask) = self.compute_checkers::<IS_WHITE>();

		let us_bb = if IS_WHITE { self.whites } else { self.blacks };
		let not_us = !us_bb;

		// 3. Double échec → seul le roi peut bouger
		if in_check && checkers.count() >= 2 {
			self.generate_king_moves::<IS_WHITE>(out, len, us_bb, not_us);
			return;
		}
		// ---------------------------
		// KINGS
		// ---------------------------
		self.generate_king_moves::<IS_WHITE>(out, len, us_bb, not_us);

		// 5. Si échec simple → seules les pièces qui capturent ou bloquent
		let restrict_mask = if in_check { block_mask } else { Bitboard8x8::FULL };
		let (pinned, pin_dir) = self.compute_pins::<IS_WHITE>();

		let blockers = self.all().storage();
		let empty = Bitboard8x8::from_storage(!blockers);

		// ---------------------------
		// PAWNS
		// ---------------------------
		let pawns = self.pawns & us_bb;
		for from_sq in pawns.iter_bits() {
			let from = Square::from_index(from_sq as u8);
			let dir = pin_dir[from_sq as usize];

			let attacks = Self::pawn_attacks::<IS_WHITE>(from);
			let pushes  = Self::pawn_pushes::<IS_WHITE>(from, empty);

			let enemy_bb = if IS_WHITE { self.blacks } else { self.whites };
			// Captures
			for to_sq in (attacks & enemy_bb & !self.kings).iter_bits() {
				if !restrict_mask.get_at_index(to_sq as usize) {
					continue;
				}
				if dir != 0 && !Self::move_respects_pin(from_sq as usize, to_sq as usize, dir) {
					continue;
				}
				let to = Square::from_index(to_sq as u8);
				if ChessBoard::is_promotion_rank::<IS_WHITE>(to) {
					for promo in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
						out[*len] = Move { from, to, promotion: Some(promo) };
						*len += 1;
					}
				} else {
					out[*len] = Move { from, to, promotion: None };
					*len += 1;
				}
			}

			// Pushes
			for to_sq in pushes.iter_bits() {
				if !restrict_mask.get_at_index(to_sq as usize) {
					continue;
				}
				if dir != 0 && !Self::move_respects_pin(from_sq as usize, to_sq as usize, dir) {
					continue;
				}
				let to = Square::from_index(to_sq as u8);
				if ChessBoard::is_promotion_rank::<IS_WHITE>(to) {
					for promo in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
						out[*len] = Move { from, to, promotion: Some(promo) };
						*len += 1;
					}
				} else {
					out[*len] = Move { from, to, promotion: None };
					*len += 1;
				}
			}

			// En passant
			if dir == 0 {
				if let Some(ep_sq) = self.ep_square {
					if attacks.get_at_index(ep_sq.0 as usize) {
						out[*len] = Move { from, to: ep_sq, promotion: None };
						*len += 1;
					}
				}
			}
		}

		// ---------------------------
		// KNIGHTS
		// ---------------------------
		let knights = self.knights & us_bb;
		for from_sq in knights.iter_bits() {
			if pinned.get_at_index(from_sq as usize) {
				continue;
			}

			let from = Square::from_index(from_sq as u8);
			let attacks = Self::knight_attacks(from) & not_us;

			for to_sq in attacks.iter_bits() {
				if !restrict_mask.get_at_index(to_sq as usize) {
					continue;
				}
				out[*len] = Move { from, to: Square::from_index(to_sq as u8), promotion: None };
				*len += 1;
			}
		}

		// ---------------------------
		// BISHOPS
		// ---------------------------
		let bishops = self.bishops & us_bb;
		for from_sq in bishops.iter_bits() {
			let from = Square::from_index(from_sq as u8);
			let dir = pin_dir[from_sq as usize];

			let attacks = get_bishop_moves(from, blockers) & not_us;

			for to_sq in attacks.iter_bits() {
				if !restrict_mask.get_at_index(to_sq as usize) {
					continue;
				}
				if dir != 0 && !Self::move_respects_pin(from_sq as usize, to_sq as usize, dir) {
					continue;
				}
				out[*len] = Move { from, to: Square::from_index(to_sq as u8), promotion: None };
				*len += 1;
			}
		}

		// ---------------------------
		// ROOKS
		// ---------------------------
		let rooks = self.rooks & us_bb;
		for from_sq in rooks.iter_bits() {
			let from = Square::from_index(from_sq as u8);
			let dir = pin_dir[from_sq as usize];

			let attacks = get_rook_moves(from, blockers) & not_us;

			for to_sq in attacks.iter_bits() {
				if !restrict_mask.get_at_index(to_sq as usize) {
					continue;
				}
				if dir != 0 && !Self::move_respects_pin(from_sq as usize, to_sq as usize, dir) {
					continue;
				}
				out[*len] = Move { from, to: Square::from_index(to_sq as u8), promotion: None };
				*len += 1;
			}
		}

		// ---------------------------
		// QUEENS
		// ---------------------------
		let queens = self.queens & us_bb;
		for from_sq in queens.iter_bits() {
			let from = Square::from_index(from_sq as u8);
			let dir = pin_dir[from_sq as usize];

			let attacks = (get_rook_moves(from, blockers) | get_bishop_moves(from, blockers)) & not_us;

			for to_sq in attacks.iter_bits() {
				if !restrict_mask.get_at_index(to_sq as usize) {
					continue;
				}
				if dir != 0 && !Self::move_respects_pin(from_sq as usize, to_sq as usize, dir) {
					continue;
				}
				out[*len] = Move { from, to: Square::from_index(to_sq as u8), promotion: None };
				*len += 1;
			}
		}
		self.generate_legal_castling_moves::<IS_WHITE>(out, len);
	}
		
	fn generate_king_moves<const IS_WHITE:bool>(&self, out: &mut [Move; 256], len: &mut usize, us_bb: Bitboard8x8, not_us: Bitboard8x8) {
		let king = self.kings & us_bb;
		for from_sq in king.iter_bits() {
			let from = Square::from_index(from_sq as u8);
			let attacks = Self::king_attacks(from) & not_us;

			for to_sq in attacks.iter_bits() {
				if self.is_square_attacked::<IS_WHITE>( Square(to_sq as u8), (self.all() & !king).storage() ) {
					 continue; // illégal
				}
				out[*len] = Move { from, to: Square::from_index(to_sq as u8), promotion: None };
				*len += 1;
			}
		}
	}
	#[inline]
	pub fn status(&self) -> GameOutcome {
		if self.turn() == Color::Black {
			let no_legal_moves = self.legal_moves_template::<false>().is_empty();
			if !no_legal_moves {
				return GameOutcome::OnGoing;
			}
			let checkers = self.compute_checkers::<false>();
			if checkers.0 {
				GameOutcome::PLAYER1
			} else {
				GameOutcome::Draw
			}
		} else {
			let no_legal_moves = self.legal_moves_template::<true>().is_empty();
			if !no_legal_moves {
				return GameOutcome::OnGoing;
			}
			let checkers = self.compute_checkers::<true>();
			if checkers.0 {
				GameOutcome::PLAYER2
			} else {
				GameOutcome::Draw
			}
		}
	}
}

impl ChessBoard {
	#[inline]
	pub fn compute_checkers<const IS_WHITE: bool>(&self)
		-> (bool, Bitboard8x8, Bitboard8x8)
	{
		let them = if IS_WHITE { self.blacks } else { self.whites };

		let king_sq = self.king_square::<IS_WHITE>();
		let king_idx = king_sq.0 as usize;

		let blockers = self.all().storage();

		let mut checkers = Bitboard8x8::empty();
		let mut block_mask = Bitboard8x8::empty();

		// -------------------------
		// Pawns
		// -------------------------
		let enemy_pawns = self.pawns & them;
		checkers |= Self::pawn_attacks::<IS_WHITE>(king_sq) & enemy_pawns;

		// -------------------------
		// Knights
		// -------------------------
		let enemy_knights = self.knights & them;
		checkers |= Self::knight_attacks(king_sq) & enemy_knights;

		// -------------------------
		// Bishops / Queens (diagonales)
		// -------------------------
		let diag_sliders = (self.bishops | self.queens) & them;
		let diag_attackers = get_bishop_moves(king_sq, blockers) & diag_sliders;

		let d = diag_attackers.storage();
		if d != 0 {
			checkers |= diag_attackers;

			// un seul bit ?
			if d & (d - 1) == 0 {
				let a = d.trailing_zeros() as usize;
				block_mask |= Self::ray_between(king_idx, a);
			}
		}

		// -------------------------
		// 4. Rooks / Queens (line/col)
		// -------------------------
		let ortho_sliders = (self.rooks | self.queens) & them;
		let ortho_attackers = get_rook_moves(king_sq, blockers) & ortho_sliders;

		let o = ortho_attackers.storage();
		if o != 0 {
			checkers |= ortho_attackers;

			// un seul bit ?
			if o & (o - 1) == 0 {
				let a = o.trailing_zeros() as usize;
				block_mask |= Self::ray_between(king_idx, a);
			}
		}

		// -------------------------
		// 5. Checkers
		// -------------------------
		let c = checkers.storage();

		if c == 0 {
			return (false, Bitboard8x8::empty(), Bitboard8x8::empty());
		}

		// double échec → seul le roi peut bouger
		if c & (c - 1) != 0 {
			return (true, checkers, Bitboard8x8::empty());
		}

		// échec simple
		block_mask |= checkers; // capture possible
		(true, checkers, block_mask)
	}


	/// Retourne les cases entre deux cases (exclut les extrémités)
	#[inline]
	fn ray_between(from: usize, to: usize) -> Bitboard8x8 {
		Bitboard8x8::ray_between_mask(from, to)
		//perf are the same...
		//Bitboard8x8::compute_ray_between_mask(from, to)
	}
}


impl ChessBoard {
	#[inline(always)]
	fn is_promotion_rank<const IS_WHITE: bool>(to: Square) -> bool {
		let rank = to.rank();
		if IS_WHITE { rank == 7 } else { rank == 0 }
	}

	#[inline]
	fn generate_legal_castling_moves<const IS_WHITE: bool>(&self, moves: &mut [Move; 256], len: &mut usize) {
		const W_KING_SIDE_EMPTY: Bitboard8x8 = Bitboard8x8::from_storage((1 << 5) | (1 << 6));
		const B_KING_SIDE_EMPTY: Bitboard8x8 = Bitboard8x8::from_storage((1 << 61) | (1 << 62));
		const W_QUEEN_SIDE_EMPTY: Bitboard8x8 = Bitboard8x8::from_storage((1 << 3) | (1 << 2) | (1 << 1));
		const B_QUEEN_SIDE_EMPTY: Bitboard8x8 = Bitboard8x8::from_storage((1 << 59) | (1 << 58) | (1 << 57));
		let blockers = self.all().storage();
		let empty = Bitboard8x8::from_storage(!blockers);
		if IS_WHITE {
			// Petit roque
			if self.castling_rights.white_kingside() &&
				empty & W_KING_SIDE_EMPTY == W_KING_SIDE_EMPTY &&
				!self.is_square_attacked::<true>(Square(4), blockers) &&
				!self.is_square_attacked::<true>(Square(5), blockers) &&
				!self.is_square_attacked::<true>(Square(6), blockers)
			{
				moves[*len] = Move { from: Square(4), to: Square(6), promotion: None };
				*len += 1;
			}

			// Grand roque
			if self.castling_rights.white_queenside() &&
				empty & W_QUEEN_SIDE_EMPTY == W_QUEEN_SIDE_EMPTY &&
				!self.is_square_attacked::<true>(Square(4), blockers) &&
				!self.is_square_attacked::<true>(Square(3), blockers) &&
				!self.is_square_attacked::<true>(Square(2), blockers)

			{
				moves[*len] = Move { from: Square(4), to: Square(2), promotion: None };
				*len += 1;
			}
		}

		else {
			// Petit roque
			if self.castling_rights.black_kingside() &&
				empty & B_KING_SIDE_EMPTY == B_KING_SIDE_EMPTY &&
				!self.is_square_attacked::<false>(Square(60), blockers) &&
				!self.is_square_attacked::<false>(Square(61), blockers) &&
				!self.is_square_attacked::<false>(Square(62), blockers)

			{
				moves[*len] = Move { from: Square(60), to: Square(62), promotion: None };
				*len +=1;
			}

			// Grand roque
			if self.castling_rights.black_queenside() &&
				empty & B_QUEEN_SIDE_EMPTY == B_QUEEN_SIDE_EMPTY &&
				!self.is_square_attacked::<false>(Square(60), blockers) &&
				!self.is_square_attacked::<false>(Square(59), blockers) &&
				!self.is_square_attacked::<false>(Square(58), blockers)

			{
				moves[*len] = Move { from: Square(60), to: Square(58), promotion: None };
				*len += 1;
			}
		}
	}
}



impl ChessBoard {
	pub const KNIGHT_ATTACKS: &[Bitboard8x8;64] = &Bitboard8x8::generate_jump_attacks_table(&[
			(1,2),(2,1),(2,-1),(1,-2),
			(-1,-2),(-2,-1),(-2,1),(-1,2),
		]);
	pub const WPAWN_ATTACKS: &[Bitboard8x8;64] = &Bitboard8x8::generate_jump_attacks_table(&[
		(1,1),(-1,1)
	]);
	pub const BPAWN_ATTACKS: &[Bitboard8x8;64] = &Bitboard8x8::generate_jump_attacks_table(&[
		(1,-1),(-1,-1)
	]);
	#[inline(always)]
	pub fn knight_attacks(square: Square) -> Bitboard8x8 {
		Self::KNIGHT_ATTACKS[square.0 as usize]
	}
	#[inline(always)]
	pub fn king_attacks(square: Square) -> Bitboard8x8 {
		Bitboard8x8::neighbors_8_mask(square.0 as usize)
	}

	#[inline(always)]
	pub fn pawn_attacks<const IS_WHITE: bool>(square: Square) -> Bitboard8x8 {
		if IS_WHITE {
			Self::WPAWN_ATTACKS[square.0 as usize]
		} else {
			Self::BPAWN_ATTACKS[square.0 as usize]
		}
	}
}
impl ChessBoard {
	#[inline]
	pub fn piece_at(&self, sq: Square) -> Option<Piece> {
		let bb = Bitboard8x8::from_index(sq.0 as usize);

		if (self.pawns   & bb).any() { return Some(Piece::Pawn); }
		if (self.rooks   & bb).any() { return Some(Piece::Rook); }
		if (self.knights & bb).any() { return Some(Piece::Knight); }
		if (self.bishops & bb).any() { return Some(Piece::Bishop); }
		if (self.queens  & bb).any() { return Some(Piece::Queen); }
		if (self.kings   & bb).any() { return Some(Piece::King); }

		None
	}
	#[inline]
	pub fn color_at(&self, sq: Square) -> Option<Color> {
		let bb = Bitboard8x8::from_index(sq.0 as usize);

		if (self.whites   & bb).any() { return Some(Color::White); }
		if (self.blacks   & bb).any() { return Some(Color::Black); }
		None
	}
}
impl ChessBoard {
	pub fn pawn_pushes<const IS_WHITE: bool>(square: Square, free: Bitboard8x8) -> Bitboard8x8 {
		const RANK_2: Bitboard8x8 = Bitboard8x8::from_storage(0x000000000000FF00);
		const RANK_7: Bitboard8x8 = Bitboard8x8::from_storage(0x00FF000000000000);
		let sq = square.0 as usize;
		let bb = Bitboard8x8::from_index(sq);

		if IS_WHITE {
			let one = (bb << 8usize) & free;

			// double push depuis rangée 2
			if bb & RANK_2 != Bitboard8x8::EMPTY {
				let two = (bb << 16usize) & free & (free << 8usize);
				one | two
			} else {
				one
			}
		}
		else {
			let one = (bb >> 8usize) & free;

			// double push depuis rangée 7
			if bb & RANK_7 != Bitboard8x8::EMPTY {
				let two = (bb >> 16usize) & free & (free >> 8usize);
				one | two
			} else {
				one
			}
		}
	}
}


impl ChessBoard {
	#[inline]
	pub fn play(&mut self, mv: &Move) {
		if self.turn == Color::Black {
			self.play_template::<false>(mv);
		} else {
			self.play_template::<true>(mv);
		}
	}
	pub fn play_template<const IS_WHITE: bool>(&mut self, mv: &Move) {
		let from_bb = Bitboard8x8::from_index(mv.from.0 as usize);
		let to_bb   = Bitboard8x8::from_index(mv.to.0 as usize);
		let ep_square_prev = self.ep_square.take();
		// Reset previous move EP done by take
		//self.ep_square = None;

		debug_assert!( (self.whites & from_bb).any() == IS_WHITE);

		let piece = self.piece_at(mv.from)
			.expect("No piece on source square");
		let mut is_en_passant = false;
		let mut capture = None;
		// Castling
		if piece == Piece::King {
			match (mv.from.0, mv.to.0) {
				(4, 6) => { // white king
					// rook h1 → f1
					let rook_from = Bitboard8x8::from_index(7);
					let rook_to   = Bitboard8x8::from_index(5);

					self.rooks ^= rook_from;
					self.rooks |= rook_to;

					self.whites ^= rook_from;
					self.whites |= rook_to;
				}
				(4, 2) => { // white queen
					// rook a1 → d1
					let rook_from = Bitboard8x8::from_index(0);
					let rook_to   = Bitboard8x8::from_index(3);

					self.rooks ^= rook_from;
					self.rooks |= rook_to;

					self.whites ^= rook_from;
					self.whites |= rook_to;
				}

				(60, 62) => { // black king
					let rook_from = Bitboard8x8::from_index(63);
					let rook_to   = Bitboard8x8::from_index(61);

					self.rooks ^= rook_from;
					self.rooks |= rook_to;

					self.blacks ^= rook_from;
					self.blacks |= rook_to;
				}

				(60, 58) => { // black queen
					let rook_from = Bitboard8x8::from_index(56);
					let rook_to   = Bitboard8x8::from_index(59);

					self.rooks ^= rook_from;
					self.rooks |= rook_to;

					self.blacks ^= rook_from;
					self.blacks |= rook_to;
				}
				_ => {}
			}
		}
		
		// remove from origin
		match piece {
			Piece::Pawn   => self.pawns   &= !from_bb,
			Piece::Rook   => self.rooks   &= !from_bb,
			Piece::Knight => self.knights &= !from_bb,
			Piece::Bishop => self.bishops &= !from_bb,
			Piece::Queen  => self.queens  &= !from_bb,
			Piece::King   => self.kings   &= !from_bb,
		}

		if (self.all() & to_bb).any() {
			// capture
			if (self.pawns & to_bb).any()   {
				self.pawns   &= !to_bb;
				capture = Some(Piece::Pawn);
			} else if (self.rooks & to_bb).any() {
				self.rooks   &= !to_bb;
				capture = Some(Piece::Rook);
			} else if (self.knights & to_bb).any() {
				self.knights &= !to_bb;
				capture = Some(Piece::Knight);
			} else if (self.bishops & to_bb).any() {
				self.bishops &= !to_bb;
				capture = Some(Piece::Bishop);
			} else if (self.queens & to_bb).any() {
				self.queens  &= !to_bb;
				capture = Some(Piece::Queen);
			}
			//else if (self.kings & to_bb).any()   { self.kings   &= !to_bb; }
		}
		if piece == Piece::Pawn {
			let diff = mv.to.0 as i32 - mv.from.0 as i32;

			// Double pushes
			if diff == 16 {
				self.ep_square = Some(Square(mv.from.0 + 8));
			}
			if diff == -16 {
				self.ep_square = Some(Square(mv.from.0 - 8));
			}
			// En passant capture
			if let Some(ep_sq) = ep_square_prev {
				if mv.to.0 == ep_sq.0 {
					is_en_passant = true;
					capture = Some(Piece::Pawn);
					let captured_sq = if IS_WHITE {
						Square(ep_sq.0 - 8)
					} else {
						Square(ep_sq.0 + 8)
					};
					let cap_bb = Bitboard8x8::from_index(captured_sq.0 as usize);

					self.pawns &= !cap_bb;
					if IS_WHITE { self.blacks &= !cap_bb; }
					else        { self.whites &= !cap_bb; }
				}
			}
		}
		let old_castling_rights = self.castling_rights;
		self.castling_rights.0 |= CastlingRights::CASTLING_MASK_FROM[mv.from.0 as usize];
		self.castling_rights.0 |= CastlingRights::CASTLING_MASK_TO[mv.to.0 as usize];
		let castling_changed = CastlingRights(old_castling_rights.0 ^ self.castling_rights.0);
		match mv.promotion {
			Some(Piece::Queen)  => self.queens  |= to_bb,
			Some(Piece::Rook)   => self.rooks   |= to_bb,
			Some(Piece::Bishop) => self.bishops |= to_bb,
			Some(Piece::Knight) => self.knights |= to_bb,
			Some(Piece::King) => panic!("Promoting to King"),
			Some(Piece::Pawn) => panic!("Promoting to Pawn"),
			None => {
				// coup normal
				match piece {
					Piece::Pawn   => self.pawns   |= to_bb,
					Piece::Rook   => self.rooks   |= to_bb,
					Piece::Knight => self.knights |= to_bb,
					Piece::Bishop => self.bishops |= to_bb,
					Piece::Queen  => self.queens  |= to_bb,
					Piece::King   => self.kings   |= to_bb,
				}
			}
		}

		if IS_WHITE {
			self.whites ^= from_bb;
			self.whites |= to_bb;
			self.blacks &= !to_bb; // capture if needed
		} else {
			self.blacks ^= from_bb;
			self.blacks |= to_bb;
			self.whites &= !to_bb;
		}

		self.update_hash_move::<IS_WHITE>(mv, piece, capture, is_en_passant, castling_changed, ep_square_prev);
		
		self.turn = self.turn.opponent();
		self.update_hash_turn();
	}
	#[inline]
	pub fn is_square_attacked<const IS_WHITE: bool>(&self, sq: Square, blockers: u64) -> bool {
		let attackers = if IS_WHITE { self.blacks } else { self.whites };
		let all = blockers;
		if IS_WHITE {
			if (Self::pawn_attacks::<true>(sq) & self.pawns & attackers).any() {
				return true;
			}
		} else {
			if (Self::pawn_attacks::<false>(sq) & self.pawns & attackers).any() {
				return true;
			}
		}

		if (Self::knight_attacks(sq) & self.knights & attackers).any() {
			return true;
		}

		if (Self::king_attacks(sq) & self.kings & attackers).any() {
			return true;
		}

		//let bishop_like = self.attacks_of(Piece::Bishop, sq, by, all);
		let bishop_like = get_bishop_moves(sq, all);
		if (bishop_like & (self.bishops | self.queens) & attackers).any() {
			return true;
		}

		//let rook_like = self.attacks_of(Piece::Rook, sq, by, all);
		let rook_like = get_rook_moves(sq, all);
		if (rook_like & (self.rooks | self.queens) & attackers).any() {
			return true;
		}

		false
	}

	#[inline]
	pub fn king_square<const IS_WHITE: bool>(&self) -> Square {
		let bb = if IS_WHITE { self.kings & self.whites } else { self.kings & self.blacks };
		debug_assert!(bb.any(), "king_square: no king for {:?}", if IS_WHITE {Color::White} else {Color::Black});

		Square::from_index(bb.storage().trailing_zeros() as u8)
	}
	#[inline]
	pub fn is_in_check(&self, c: Color) -> bool {
		let checkers = if c == Color::White {
			self.compute_checkers::<true>()
		} else {
			self.compute_checkers::<false>()
		};
		checkers.0
	}
}

pub struct Zobrist {
	pub pawns: [[u64; 2]; Bitboard8x8::NB_SQUARES],
	pub rooks: [[u64; 2]; Bitboard8x8::NB_SQUARES],
	pub knights: [[u64; 2]; Bitboard8x8::NB_SQUARES],
	pub bishops: [[u64; 2]; Bitboard8x8::NB_SQUARES],
	pub queens: [[u64; 2]; Bitboard8x8::NB_SQUARES],
	pub kings: [[u64; 2]; Bitboard8x8::NB_SQUARES],
	pub castling_rights: [u64; 4],
	pub en_passant: [u64; 8],
	pub turn: u64,
}

impl Zobrist {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);
		let mut pawns = [[0u64; 2]; Bitboard8x8::NB_SQUARES];
		let mut rooks = [[0u64; 2]; Bitboard8x8::NB_SQUARES];
		let mut knights = [[0u64; 2]; Bitboard8x8::NB_SQUARES];
		let mut bishops = [[0u64; 2]; Bitboard8x8::NB_SQUARES];
		let mut queens = [[0u64; 2]; Bitboard8x8::NB_SQUARES];
		let mut kings = [[0u64; 2]; Bitboard8x8::NB_SQUARES];
		let mut castling_rights = [0u64; 4];
		let mut en_passant = [0u64; 8];
		
		let mut i = 0;
		while i < Bitboard8x8::NB_SQUARES {
			pawns[i][0] = rng.u64();
			pawns[i][1] = rng.u64();
			rooks[i][0] = rng.u64();
			rooks[i][1] = rng.u64();
			knights[i][0] = rng.u64();
			knights[i][1] = rng.u64();
			bishops[i][0] = rng.u64();
			bishops[i][1] = rng.u64();
			queens[i][0] = rng.u64();
			queens[i][1] = rng.u64();
			kings[i][0] = rng.u64();
			kings[i][1] = rng.u64();
			i += 1;
		}
		i = 0;
		while i < 4 {
			castling_rights[i] = rng.u64();
			i += 1;
		}
		i = 0;
		while i < 8 {
			en_passant[i] = rng.u64();
			i += 1;
		}
		
		Self {
			pawns,
			rooks,
			knights,
			bishops,
			queens,
			kings,
			castling_rights,
			en_passant,
			turn: rng.u64(),
		}
	}
}
impl ChessBoard {
	const ZOBRIST_KEYS: Zobrist = Zobrist::new(0x91F4A12);
	pub(crate) fn compute_zobrist(&self) -> u64 {
		let mut h = 0u64;
		for i in (self.pawns & self.whites).iter_bits() { h ^= Self::ZOBRIST_KEYS.pawns[i as usize][0]; }
		for i in (self.pawns & self.blacks).iter_bits() { h ^= Self::ZOBRIST_KEYS.pawns[i as usize][1]; }
		for i in (self.rooks & self.whites).iter_bits() { h ^= Self::ZOBRIST_KEYS.rooks[i as usize][0]; }
		for i in (self.rooks & self.blacks).iter_bits() { h ^= Self::ZOBRIST_KEYS.rooks[i as usize][1]; }
		for i in (self.knights & self.whites).iter_bits() { h ^= Self::ZOBRIST_KEYS.knights[i as usize][0]; }
		for i in (self.knights & self.blacks).iter_bits() { h ^= Self::ZOBRIST_KEYS.knights[i as usize][1]; }
		for i in (self.bishops & self.whites).iter_bits() { h ^= Self::ZOBRIST_KEYS.bishops[i as usize][0]; }
		for i in (self.bishops & self.blacks).iter_bits() { h ^= Self::ZOBRIST_KEYS.bishops[i as usize][1]; }
		for i in (self.queens & self.whites).iter_bits() { h ^= Self::ZOBRIST_KEYS.queens[i as usize][0]; }
		for i in (self.queens & self.blacks).iter_bits() { h ^= Self::ZOBRIST_KEYS.queens[i as usize][1]; }
		for i in (self.kings & self.whites).iter_bits() { h ^= Self::ZOBRIST_KEYS.kings[i as usize][0]; }
		for i in (self.kings & self.blacks).iter_bits() { h ^= Self::ZOBRIST_KEYS.kings[i as usize][1]; }
		if self.castling_rights.white_kingside() {
			h ^= Self::ZOBRIST_KEYS.castling_rights[0];
		}
		if self.castling_rights.white_queenside() {
			h ^= Self::ZOBRIST_KEYS.castling_rights[1];
		}
		if self.castling_rights.black_kingside() {
			h ^= Self::ZOBRIST_KEYS.castling_rights[2];
		}
		if self.castling_rights.black_queenside() {
			h ^= Self::ZOBRIST_KEYS.castling_rights[3];
		}
		// TODO: BUG ep_square is set even if no capture is possible
		if let Some(ep) = self.ep_square {
			h ^= Self::ZOBRIST_KEYS.en_passant[ep.file()];
		}
		if self.turn == Color::Black { h ^= Self::ZOBRIST_KEYS.turn; }
		h
	}
	fn zobrist_piece(piece: Piece, sq: usize, color: usize) -> u64 {
		match piece {
			Piece::Pawn   => Self::ZOBRIST_KEYS.pawns[sq][color],
			Piece::Rook   => Self::ZOBRIST_KEYS.rooks[sq][color],
			Piece::Knight => Self::ZOBRIST_KEYS.knights[sq][color],
			Piece::Bishop => Self::ZOBRIST_KEYS.bishops[sq][color],
			Piece::Queen  => Self::ZOBRIST_KEYS.queens[sq][color],
			Piece::King   => Self::ZOBRIST_KEYS.kings[sq][color],
		}
	}
	fn update_hash_move<const IS_WHITE: bool>(&mut self, m: &Move, piece: Piece, capture: Option<Piece>, is_en_passant: bool, castling_changed: CastlingRights, ep_square_prev: Option<Square>) {
		let from = m.from.0 as usize;
		let to = m.to.0 as usize;
		let (color, opp) = if IS_WHITE {(0, 1)} else {(1, 0)};
		//println!("{m}, {piece:?}, {capture:?}, {is_en_passant}, {castling_changed:?}, {ep_square_prev:?}, {:?}", self.ep_square);
		// --- 1. REMOVE PIECE FROM SOURCE ---
		self.hash ^= Self::zobrist_piece(piece, from, color);

		// --- 2. HANDLE CAPTURE ---
		if let Some(captured) = capture {
			let cap_sq = if is_en_passant {
				// pion capturé derrière
				if IS_WHITE { to - 8 } else { to + 8 }
			} else {
				to
			};
			self.hash ^= Self::zobrist_piece(captured, cap_sq, opp);
		}

		// --- 3. REMOVE OLD EN PASSANT ---
		if let Some(ep) = ep_square_prev {
			self.hash ^= Self::ZOBRIST_KEYS.en_passant[ep.file()];
		}

		// --- 4. REMOVE OLD CASTLING RIGHTS ---
		if !castling_changed.white_kingside() {
			self.hash ^= Self::ZOBRIST_KEYS.castling_rights[0];
		}
		if !castling_changed.white_queenside() {
			self.hash ^= Self::ZOBRIST_KEYS.castling_rights[1];
		}
		if !castling_changed.black_kingside() {
			self.hash ^= Self::ZOBRIST_KEYS.castling_rights[2];
		}
		if !castling_changed.black_queenside() {
			self.hash ^= Self::ZOBRIST_KEYS.castling_rights[3];
		}

		// --- 5. MOVE / PROMOTION ---
		let piece_to_place = if let Some(promo) = m.promotion {
			promo
		} else {
			piece
		};

		self.hash ^= Self::zobrist_piece(piece_to_place, to, color);

		// --- 6. CASTLING MOVE ---
		 if let Some(ct) = m.castling_type(piece) {
			match ct {
				0 => { // white king side
					self.hash ^= Self::zobrist_piece(Piece::Rook, 7, color);
					self.hash ^= Self::zobrist_piece(Piece::Rook, 5, color);
				}
				1 => { // white queen side
					self.hash ^= Self::zobrist_piece(Piece::Rook, 0, color);
					self.hash ^= Self::zobrist_piece(Piece::Rook, 3, color);
				}
				2 => { // black king side
					self.hash ^= Self::zobrist_piece(Piece::Rook, 63, color);
					self.hash ^= Self::zobrist_piece(Piece::Rook, 61, color);
				}
				3 => { // black queen side
					self.hash ^= Self::zobrist_piece(Piece::Rook, 56, color);
					self.hash ^= Self::zobrist_piece(Piece::Rook, 59, color);
				}
				_ => {}
			}
		}

		// --- 8. NEW EN PASSANT ---
		if let Some(ep) = self.ep_square {
			// TODO: check capture ok
			self.hash ^= Self::ZOBRIST_KEYS.en_passant[ep.file()];
		}
	}
	fn update_hash_turn(&mut self) {
		self.hash ^= Self::ZOBRIST_KEYS.turn
	}
}
impl Game for ChessBoard {
	type S = ChessBoard;
	type M = Move;

	#[inline]
	fn generate_moves(b: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		let mut array = [Move { from: Square(0), to: Square(0), promotion: None }; 256];
		let mut len = 0;
		if b.turn == Color::White {
			b.legal_moves_inplace::<true>(&mut array, &mut len);
		} else {
			b.legal_moves_inplace::<false>(&mut array, &mut len);
		}
		moves.extend_from_slice(&array[..len]);
		// TODO: check winner
		Self::get_winner(b)
	}

	#[inline]
	fn get_winner(b: &Self::S) -> GameOutcome {
		b.status()
	}

	#[inline]
	fn apply(b: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut copy = *b;
		copy.play(&m);
		
		Some(copy)
	}

	fn get_hash(b: &Self::S) -> u64 {
		b.hash
	}

	fn notation(_b: &Self::S, m: Self::M) -> Option<String> {
		_b.move_to_san(&m).ok()
	}
	fn get_current_player(b: &Self::S) -> Player {
		match b.turn() {
			Color::White => Player::PLAYER1,
			Color::Black => Player::PLAYER2,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChessMaterialEval;
impl ChessMaterialEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Default for ChessMaterialEval {
	fn default() -> Self {
		Self::new()
	}
}
impl Evaluator for ChessMaterialEval {
	type G = ChessBoard;
	fn evaluate_for(&self, state: &ChessBoard, p: Player) -> Evaluation {
		const P: i16 = 100;
		const N: i16 = 320;
		const B: i16 = 330;
		const R: i16 = 500;
		const Q: i16 = 900;
		let w = (state.pawns & state.whites).count() as i16 * P +
			(state.knights & state.whites).count() as i16 * N +
			(state.bishops & state.whites).count() as i16 * B +
			(state.rooks & state.whites).count() as i16 * R +
			(state.queens & state.whites).count() as i16 * Q;
		let b = (state.pawns & state.blacks).count() as i16 * P +
			(state.knights & state.blacks).count() as i16 * N +
			(state.bishops & state.blacks).count() as i16 * B +
			(state.rooks & state.blacks).count() as i16 * R +
			(state.queens & state.blacks).count() as i16 * Q;
		if p == Player::PLAYER1 {
			w - b
		} else {
			b - w
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChessPosEval;
impl ChessPosEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Default for ChessPosEval {
	fn default() -> Self {
		Self::new()
	}
}
impl Evaluator for ChessPosEval {
	type G = ChessBoard;

	fn evaluate_for(&self, state: &ChessBoard, p: Player) -> Evaluation {
		const P: i16 = 100;
		const N: i16 = 320;
		const B: i16 = 330;
		const R: i16 = 500;
		const Q: i16 = 900;

		const KNIGHT_TABLE: [i16; 64] = [
			-25,-20,-15,-15,-15,-15,-20,-25,
			-20,-10,  0,  0,  0,  0,-10,-20,
			-15,  0,  5,  8,  8,  5,  0,-15,
			-15,  5,  8, 10, 10,  8,  3,-15,
			-15,  0,  8, 10, 10,  8,  0,-15,
			-15,  5,  5,  8,  8,  5,  3,-15,
			-20,-10,  0,  3,  3,  0,-10,-20,
			-25,-40,-15,-15,-15,-15,-20,-25,
		];

		const BISHOP_TABLE: [i16; 64] = [
			-10,-10,-10,-10,-10,-10,-10,-10,
			-10,  0,  0,  0,  0,  0,  0,-10,
			-10,  0,  5, 10, 10,  5,  0,-10,
			-10,  5,  5, 10, 10,  5,  5,-10,
			-10,  0, 10, 10, 10, 10,  0,-10,
			-10, 10, 10, 10, 10, 10, 10,-10,
			-10,  5,  0,  0,  0,  0,  5,-10,
			-10,-10,-10,-10,-10,-10,-10,-10,
		];

		const ROOK_TABLE: [i16; 64] = [
			0, 0, 0, 0, 0, 0, 0, 0,
			5,10,10,10,10,10,10, 5,
			-5, 0, 0, 0, 0, 0, 0,-5,
			-5, 0, 0, 0, 0, 0, 0,-5,
			-5, 0, 0, 0, 0, 0, 0,-5,
			-5, 0, 0, 0, 0, 0, 0,-5,
			-5, 0, 0, 0, 0, 0, 0,-5,
			0, 0, 0, 5, 5, 0, 0, 0,
		];

		const PAWN_TABLE: [i16; 64] = [
			0, 0, 0, 0, 0, 0, 0, 0,
			50,50,50,50,50,50,50,50,
			10,10,20,30,30,20,10,10,
			5, 5,10,25,25,10, 5, 5,
			0, 0, 0,20,20, 0, 0, 0,
			5,-5,-10, 0, 0,-10,-5, 5,
			5,10,10,-20,-20,10,10, 5,
			0, 0, 0, 0, 0, 0, 0, 0,
		];

		let mut w_score = (state.pawns & state.whites).count() as i16 * P +
						(state.knights & state.whites).count() as i16 * N +
						(state.bishops & state.whites).count() as i16 * B +
						(state.rooks & state.whites).count() as i16 * R +
						(state.queens & state.whites).count() as i16 * Q;

		let mut b_score = (state.pawns & state.blacks).count() as i16 * P +
						(state.knights & state.blacks).count() as i16 * N +
						(state.bishops & state.blacks).count() as i16 * B +
						(state.rooks & state.blacks).count() as i16 * R +
						(state.queens & state.blacks).count() as i16 * Q;

		for sq in state.knights.iter_bits() {
			let sq = sq as usize;
			let bb = Bitboard8x8::from_index(sq);
			if (state.whites & bb).any() { w_score += KNIGHT_TABLE[sq]; }
			if (state.blacks & bb).any() { b_score += KNIGHT_TABLE[63 - sq]; }
		}

		for sq in state.bishops.iter_bits() {
			let sq = sq as usize;
			let bb = Bitboard8x8::from_index(sq);
			if (state.whites & bb).any() { w_score += BISHOP_TABLE[sq]; }
			if (state.blacks & bb).any() { b_score += BISHOP_TABLE[63 - sq]; }
		}

		for sq in state.rooks.iter_bits() {
			let sq = sq as usize;
			let bb = Bitboard8x8::from_index(sq);
			if (state.whites & bb).any() { w_score += ROOK_TABLE[sq]; }
			if (state.blacks & bb).any() { b_score += ROOK_TABLE[63 - sq]; }
		}

		for sq in state.pawns.iter_bits() {
			let sq = sq as usize;
			let bb = Bitboard8x8::from_index(sq);
			if (state.whites & bb).any() { w_score += PAWN_TABLE[sq] / 2; }
			if (state.blacks & bb).any() { b_score += PAWN_TABLE[63 - sq] / 2; }
		}

		let w_mobility = state.legal_moves_template::<true>().len() as i16;
		let b_mobility = state.legal_moves_template::<false>().len() as i16;
		w_score += w_mobility;
		b_score += b_mobility;

		//let w_files = (0..8).map(|f: u8| ((state.pawns & state.whites) >> f).count() as i16).collect::<Vec<_>>();
		//let b_files = (0..8).map(|f: u8| ((state.pawns & state.blacks) >> f).count() as i16).collect::<Vec<_>>();
		//w_score -= w_files.iter().filter(|&&c| c > 1).sum::<i16>() * 20;
		//b_score -= b_files.iter().filter(|&&c| c > 1).sum::<i16>() * 20;

		if p == Player::PLAYER1 { w_score - b_score } else { b_score - w_score }
	}
}
#[cfg(test)]
mod tests {

	use kudchuet::ai::minimax::util::perft;

use crate::bitboard::Bitboard8x8;

use super::ChessBoard;
	#[test]
	fn test_display() {
		let mut chess = ChessBoard::default();
		println!("{}", chess);
		let mut count = 0;
		let mut legal_moves = chess.legal_moves();
		while count < 30 && !legal_moves.is_empty() {
			chess.play(&legal_moves[0]);
			println!("{}", chess);
			legal_moves = chess.legal_moves();
			count+=1;
		}
		
	}
	#[test]
	fn test_play() {
		let mut rng = kudchuet::utils::Rng::new();
		let mut chess = ChessBoard::default();
		assert_eq!(chess.compute_zobrist(), chess.hash);
		println!("{}", chess);
		let mut count = 0;
		let mut legal_moves = chess.legal_moves();
		while count < 300 && !legal_moves.is_empty() {
			chess.play(&legal_moves[rng.range(0, legal_moves.len())]);
			println!("{}", chess);
			legal_moves = chess.legal_moves();
			assert_eq!(chess.compute_zobrist(), chess.hash);
			count+=1;
		}
		println!("{}", chess);
	}
	#[test]
	fn test_compute_pins_simple_diagonal_pin() {
		// Position :
		let board = ChessBoard::from_fen("4r3/8/8/4P3/8/2b5/3R4/4K3 w - - 0 1")
			.expect("FEN invalide");
		println!("{}", board);
		let (pinned, pin_dir) = board.compute_pins::<true>();
		println!("{}", pinned);
		let d2 = 11usize;
		let e5 = 36usize;

		assert!(
			pinned.get_at_index(d2),
			"Rook d2 should be pinned"
		);
		assert!(
			pinned.get_at_index(e5),
			"Pawn e5 should be pinned"
		);

		let dir = pin_dir[d2];
		assert!(
			dir != 0,
			"direction for d2 should not be 0"
		);
		println!("Direction détectée pour d2 = {}", dir);
		let dir = pin_dir[e5];
		assert!(
			dir != 0,
			"direction for e5 should not be 0"
		);
		println!("Direction détectée pour e5 = {}", dir);
	}
	#[test]
	fn test_compute_pins_first_failed() {
		let board = ChessBoard::from_fen("rnbqkbnr/1ppppppp/8/p7/Q7/2P5/PP1PPPPP/RNB1KBNR b KQkq - 0 1")
			.expect("FEN invalide");
		println!("{}", board);
		let (pinned, _pin_dir) = board.compute_pins::<false>();
		println!("{}", pinned);
		let ray = board.all().ray_sw(board.king_square::<false>().0 as usize);
		println!("{}", ray);
		let ray = Bitboard8x8::ray_sw_mask(board.king_square::<false>().0 as usize);
		println!("{}", ray);
		let ray = Bitboard8x8::ray_se_mask(board.king_square::<false>().0 as usize);
		println!("{}", ray);
		let ray = Bitboard8x8::ray_ne_mask(board.king_square::<true>().0 as usize);
		println!("{}", ray);
		let ray = Bitboard8x8::ray_nw_mask(board.king_square::<true>().0 as usize);
		println!("{}", ray);
	}

	#[test]
	fn test_compute_promote() {
		let board = ChessBoard::from_fen("8/2P5/8/8/2p5/1r6/K7/6rk w - - 0 1").expect("FEN invalide");
		println!("{}", board);
		let moves = board.legal_moves();
		println!("{:?}", moves);
		assert_eq!(moves.len(), 4);
		assert!(moves.iter().all(|m| m.promotion.is_some()));
	}

	#[test]
	fn test_compute_checkers() {
		// Position :
		let board = ChessBoard::from_fen("4r3/8/8/8/8/2b5/8/4K3 w - - 0 1")
			.expect("FEN invalid");
		println!("{}", board);
		let (check, checkers, blockers)=board.compute_checkers::<true>();
		println!("check: {}, checkers: {}, blockers: {}",check, checkers, blockers);
		
		let board = ChessBoard::from_fen("8/8/8/8/8/2b5/8/4K3 w - - 0 1")
			.expect("FEN invalide");
		println!("{}", board);
		let (check, checkers, blockers)=board.compute_checkers::<true>();
		println!("check: {}, checkers: {}, blockers: {}",check, checkers, blockers);

		let board = ChessBoard::from_fen("rnbqkbnr/pp1ppppp/8/8/8/P1pP4/1PPKPPPP/RNBQ1BNR w - - 0 1")
			.expect("FEN invalide");
		println!("{}", board);
		let (check, checkers, blockers)=board.compute_checkers::<true>();
		println!("check: {}, checkers: {}, blockers: {}",check, checkers, blockers);
	}
	// cargo test --release -p chess@0.1 mychess::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	//    0               1       4.2µs       238.1
	//    1              20      55.3µs       361.7
	//    2             400      27.6µs     14492.8
	//    3            8902     150.6µs     59110.2
	//    4          197281     856.5µs    230333.9
	//    5         4865609       9.5ms    510289.4
	//    6       119060324     197.4ms    603000.4
	//    7      3195901860        5.1s    631837.3
	//cargo flamegraph --unit-test abstract_strategy -- chess::mychess::tests::perft_test --perfdata perf.data
	
	#[test]
	fn perft_test() {
		println!("BMI1 enabled? {}", cfg!(target_feature = "bmi1"));
		println!("BMI2 enabled? {}", cfg!(target_feature = "bmi2"));
		let mut board = ChessBoard::default();
		let max_depth = 7;
		let nodes = perft::<ChessBoard>(&mut board, max_depth, true);
		assert!(nodes.len() == (max_depth+1) as usize);
		const NB_NODES: [u64; 13] = [
			1,                     // depth 0
			20,                    // depth 1
			400,                   // depth 2
			8_902,                 // depth 3
			197_281,               // depth 4
			4_865_609,             // depth 5
			119_060_324,           // depth 6
			3_195_901_860,         // depth 7
			84_998_978_956,        // depth 8
			2_357_039_609_552,     // depth 9
			69_352_859_712_417,    // depth 10
			2_097_651_003_696_806, // depth 11
			71_852_195_823_968_866 // depth 12
		];

		for (i, n) in nodes.iter().enumerate() {
			assert!(NB_NODES[i] == *n)
		}
	}
	#[test]
	fn middle_perft_test() {
		// Position Kiwipete (position de test standard)
		let mut board = ChessBoard::from_fen(
			"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
		).expect("FEN should be valid");

		let max_depth = 7;
		let nodes = perft::<ChessBoard>(&mut board, max_depth, true);
		assert!(nodes.len() == (max_depth + 1) as usize);

		const NB_NODES: [u64; 9] = [
			1,                     // depth 0
			48,                    // depth 1
			2039,                  // depth 2
			97_862,                // depth 3
			4_085_603,             // depth 4
			193_690_690,           // depth 5
			8_031_647_685,         // depth 6
			302_457_915_873,       // depth 7
			10_595_738_199_614,    // depth 8
		];

		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
	#[test]
	fn test_mat() {
		let mut board = ChessBoard::default();
		let m = board.san_to_move("f3");
		board.play(&m.unwrap());
		let m = board.san_to_move("e5");
		board.play(&m.unwrap());
		println!("{}", board);
		let m = board.san_to_move("g4");
		board.play(&m.unwrap());
		let m = board.san_to_move("Qh4");
		board.play(&m.unwrap());
		println!("{}", board);
		println!("{:?}", board.status());
		println!("{:?}", board.legal_moves());
	}
	#[test]
	fn test_mat2() {
		let board = ChessBoard::from_fen("R6k/5ppp/8/8/8/8/P1P3PP/r6K w - - 0 1 ").unwrap();
		println!("{}", board);
		println!("{:?}", board.status());
		println!("{:?}", board.legal_moves());
	}
	#[test]
	fn test_fen() {
		pub const TEST_FENS: &[&str] = &[
			// 1. Initial
			"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",

			// 2. 1.e4
			"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",

			// 3. En passant
			"rnbqkbnr/pp1ppppp/8/3P4/8/8/PPP2PPP/RNBQKBNR w KQkq d6 0 3",

			// 4. Partial castling
			"rnbq1rk1/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQK2R w K - 5 5",

			// 5. Partial castling
			"r3k2r/pppq1ppp/2np4/4p3/4P3/2NP1N2/PPPQ1PPP/2KR1B1R b kq - 7 7",

			// 6. Kiwi
			"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",

			// 7. Promotion imminente
			"8/6P1/8/8/8/8/8/7K w - - 0 1",

			// 8. Mat du berger
			"rnbqkb1r/pppp1Qpp/5n2/4p3/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 0 3",

			// 9. Only kings
			"4k3/8/8/8/8/8/8/4K3 w - - 0 1",

			// 10. Stress test (illegal)
			"rnbqkbnr/pppppppp/pppppppp/pppppppp/PPPPPPPP/PPPPPPPP/pppppppp/RNBQKBNR w KQkq - 0 1",
		];
		for fen in TEST_FENS {
			let board = ChessBoard::from_fen(fen);
			
			let board = board.unwrap_or_else(|e| panic!("{} {:?}", fen, e));

			let fen_out = board.to_fen();
			assert_eq!(fen_out[..fen_out.len() - 4], fen[..fen.len() - 4]);
		}

		assert!(ChessBoard::from_fen("").is_err());
		// Missing square
		assert!(ChessBoard::from_fen("rnbqkbn/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err());
		// Missing line
		assert!(ChessBoard::from_fen("rnbqkbnr/pppppppp/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err());
		// Invalid trait
		assert!(ChessBoard::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR A KQkq - 0 1").is_err());
		// No optional fields
		ChessBoard::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -").unwrap_or_else(|e| panic!("{} {:?}", "rnbqkbnr/pppppppp/8/8/8/PPPPPPPP/RNBQKBNR A KQkq -", e));
	}
}
