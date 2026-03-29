
use crate::common::bitboards::Bitboard8x8;

pub mod chess;
pub mod chess2;
pub mod mychess;
pub mod magic_tables;
pub mod evaluation;
pub mod gui;
pub mod pext_tables;
pub mod fen;
pub mod san;
//pub mod uci_parser;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Piece {
	Pawn,
	Rook,
	Knight,
	Bishop,
	Queen,
	King
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Color {
	White,
	Black
}
impl Color {
	#[inline(always)]
	pub fn opponent(&self) -> Color {
		match self {
			Color::White => Color::Black,
			Color::Black => Color::White,
		}
	}
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Square(u8);
impl Square {
	pub const NUM : usize = 64;
	#[inline(always)]
	pub const fn from_index(index:u8) -> Self {
		debug_assert!(index < 64);
		Square(index)
	}
	#[inline(always)]
	pub const fn from_coords(file:usize, rank:usize) -> Self {
		let index = Bitboard8x8::index_from_coords(file as u8, rank as u8) as u8;
		debug_assert!(index < 64);
		Square(index)
	}
	#[inline(always)]
	pub const fn file(&self) -> usize {
		(self.0 as usize) % 8
	}

	#[inline(always)]
	pub const fn rank(&self) -> usize {
		(self.0 as usize) / 8
	}
	#[inline]
	pub fn step(&self, dx: i8, dy: i8) -> Option<Square> { 
		let f = self.file() as i8 + dx;
		let r = self.rank() as i8 + dy;
		if (0..8).contains(&f) && (0..8).contains(&r) {
			Some(Square::from_coords(f as usize, r as usize))
		} else {
			None
		}
	}
}

impl std::fmt::Display for Square {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let (file, rank) = Bitboard8x8::coords_from_index(self.0 as usize);
		let file_char = (b'a' + file) as char;
		let rank_char = (b'1' + rank) as char;

		write!(f, "{}{}", file_char, rank_char)
	}
}

impl From<Square> for usize {
	#[inline(always)]
	fn from(val: Square) -> Self {
		val.0 as usize
	}
}
impl Square {
	pub fn from_uci(s: &str) -> Option<Square> {
		let bytes = s.as_bytes();
		if bytes.len() != 2 {
			return None;
		}

		let file = bytes[0];
		let rank = bytes[1];

		if !(b'a'..=b'h').contains(&file) { return None; }
		if !(b'1'..=b'8').contains(&rank) { return None; }

		let file_idx = (file - b'a') as u8;
		let rank_idx = (rank - b'1') as u8;

		Some(Square(rank_idx * 8 + file_idx))
	}
}
impl std::str::FromStr for Square {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Square::from_uci(s).ok_or("invalid UCI square")
	}
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Move {
	pub from: Square,
	pub to: Square,
	pub promotion:Option<Piece>
}

impl std::fmt::Display for Move {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}{}{}", self.from, self.to,
			match self.promotion {
				Some(Piece::Queen) => "q",
				Some(Piece::Rook) => "r",
				Some(Piece::Bishop) => "b",
				Some(Piece::Knight) => "n",
				_ => "",
			}
		)
	}
}
impl Move {
	pub fn from_uci(s: &str) -> Option<Move> {
		let bytes = s.as_bytes();

		// Longueur valide : 4 (normal) ou 5 (promotion)
		if bytes.len() != 4 && bytes.len() != 5 {
			return None;
		}

		// Parse les cases from/to
		let from = Square::from_uci(&s[0..2])?;
		let to   = Square::from_uci(&s[2..4])?;

		// Promotion éventuelle
		let promotion = if bytes.len() == 5 {
			match bytes[4] {
				b'q' => Some(Piece::Queen),
				b'r' => Some(Piece::Rook),
				b'b' => Some(Piece::Bishop),
				b'n' => Some(Piece::Knight),
				_ => return None,
			}
		} else {
			None
		};

		Some(Move { from, to, promotion })
	}
	pub fn is_castling(&self, piece: Piece) -> bool {
		self.castling_type(piece).is_some()
	}
	pub fn castling_type(&self, piece: Piece) -> Option<u8> {
		if piece != Piece::King {
			return None;
		}
		match (self.from.0, self.to.0) {
			(4, 6) => { // white king
				Some(0)
			}
			(4, 2) => { // white queen
				Some(1)
			}
			(60, 62) => { // black king
				Some(2)
			}
			(60, 58) => { // black queen
				Some(3)
			}
			_ => None
		}
	}
}

impl std::str::FromStr for Move {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Move::from_uci(s).ok_or("invalid UCI move")
	}
}

// TODO: test if an enum is better
#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct CastlingRights(u8);
impl CastlingRights {
	pub const WK: u8 = 1;
	pub const WQ: u8 = 2;
	pub const BK: u8 = 4;
	pub const BQ: u8 = 8;
	pub(crate) const CASTLING_MASK_FROM: [u8; 64] = {
		let mut m = [0u8; 64];

		// White
		m[4]  = Self::WK | Self::WQ; // E1
		m[7]  = Self::WK;     // H1
		m[0]  = Self::WQ;     // A1

		// Black
		m[60] = Self::BK | Self::BQ; // E8
		m[63] = Self::BK;     // H8
		m[56] = Self::BQ;     // A8

		m
	};
	pub(crate) const CASTLING_MASK_TO: [u8; 64] = {
		let mut m = [0u8; 64];

		// White
		m[0]  = Self::WQ; // A1
		m[7]  = Self::WK; // H1

		// Black
		m[56] = Self::BQ; // A8
		m[63] = Self::BK; // H8

		m
	};
	#[inline]
	pub fn white_kingside(&self) -> bool {
		self.0 & 1 == 0
	}
	#[inline]
	pub fn white_queenside(&self) -> bool {
		self.0 & 2 == 0
	}
	#[inline]
	pub fn black_kingside(&self) -> bool {
		self.0 & 4 == 0
	}
	#[inline]
	pub fn black_queenside(&self) -> bool {
		self.0 & 8 == 0
	}
	#[inline]
	pub fn none(&self) -> bool {
		self.0 == 15
	}
	#[inline]
	pub fn remove_white_kingside(&mut self) {
		self.0 |= 1;
	}
	#[inline]
	pub fn remove_white_queenside(&mut self) {
		self.0 |= 2;
	}
	#[inline]
	pub fn remove_black_kingside(&mut self) {
		self.0 |= 4;
	}
	#[inline]
	pub fn remove_black_queenside(&mut self) {
		self.0 |= 8;
	}

}
#[cfg(test)]
mod tests {
	use crate::common::gui::BoardGame;

	#[test]
	fn test_promotion() {
	let b = crate::chess::mychess::ChessBoard::from_fen("8/8/1P1k4/1K1b1p2/5P2/8/r1p5/8 b - - 0 1").unwrap();
	let m = b.move_from_string(&"c2c1q".into());
	println!("{m:?}");
	}
}