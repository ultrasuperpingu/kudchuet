use eframe::egui;
use egui::Color32;
use crate::chess::mychess::ChessPosEval;
use crate::chess::{Color, Move, Square, mychess::ChessMaterialEval};
use crate::common::ai::{AIEngine, AIEngineProvider, MoveSearcherBuilderDyn};
use crate::common::bitboards::Bitboard8x8;
use crate::common::gui::board_app::GenericBoardApp;
use crate::common::gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType};
use crate::common::gui::shapes::{Shape, TextData};

use super::mychess::ChessBoard;

impl BoardMove<ChessBoard> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.from.0 as u16)
	}

	fn to(&self) -> u16 {
		self.to.0 as u16
	}
	fn to_uci(&self) -> Option<String> {
		Some(self.to_string())
	}
	fn from_uci(m_str: &String) -> Result<Self, String> {
		Self::from_uci(m_str).ok_or("Invalid uci".into())
	}
}
impl EGUIPieceType for ChessPiece {
	fn shape(&self) -> Shape {
		match self {
			ChessPiece::WhitePawn =>   Shape::Composed(vec![Shape::String { text: TextData { text: "♟".into(), color: Color32::WHITE, size: 1.0 } }, Shape::String { text: TextData { text: "♙".into(), color: Color32::BLACK, size: 1.0 } }]),
			ChessPiece::WhiteRook =>   Shape::Composed(vec![Shape::String { text: TextData { text: "♜".into(), color: Color32::WHITE, size: 1.0 } }, Shape::String { text: TextData { text: "♖".into(), color: Color32::BLACK, size: 1.0 } }]),
			ChessPiece::WhiteKnight => Shape::Composed(vec![Shape::String { text: TextData { text: "♞".into(), color: Color32::WHITE, size: 1.0 } }, Shape::String { text: TextData { text: "♘".into(), color: Color32::BLACK, size: 1.0 } }]),
			ChessPiece::WhiteBishop => Shape::Composed(vec![Shape::String { text: TextData { text: "♝".into(), color: Color32::WHITE, size: 1.0 } }, Shape::String { text: TextData { text: "♗".into(), color: Color32::BLACK, size: 1.0 } }]),
			ChessPiece::WhiteQueen =>  Shape::Composed(vec![Shape::String { text: TextData { text: "♛".into(), color: Color32::WHITE, size: 1.0 } }, Shape::String { text: TextData { text: "♕".into(), color: Color32::BLACK, size: 1.0 } }]),
			ChessPiece::WhiteKing =>   Shape::Composed(vec![Shape::String { text: TextData { text: "♚".into(), color: Color32::WHITE, size: 1.0 } }, Shape::String { text: TextData { text: "♔".into(), color: Color32::BLACK, size: 1.0 } }]),
			ChessPiece::BlackPawn =>   Shape::String { text: TextData { text: "♟".into(), color: Color32::BLACK, size: 1.0 } },
			ChessPiece::BlackRook =>   Shape::String { text: TextData { text: "♜".into(), color: Color32::BLACK, size: 1.0 } },
			ChessPiece::BlackKnight => Shape::String { text: TextData { text: "♞".into(), color: Color32::BLACK, size: 1.0 } },
			ChessPiece::BlackBishop => Shape::String { text: TextData { text: "♝".into(), color: Color32::BLACK, size: 1.0 } },
			ChessPiece::BlackQueen =>  Shape::String { text: TextData { text: "♛".into(), color: Color32::BLACK, size: 1.0 } },
			ChessPiece::BlackKing =>   Shape::String { text: TextData { text: "♚".into(), color: Color32::BLACK, size: 1.0 } },
		}
	}
}
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum ChessPiece {
	WhitePawn,
	WhiteRook,
	WhiteKnight,
	WhiteBishop,
	WhiteQueen,
	WhiteKing,
	BlackPawn,
	BlackRook,
	BlackKnight,
	BlackBishop,
	BlackQueen,
	BlackKing
} 
impl BoardGame for ChessBoard {

	type PieceType=ChessPiece;

	fn width(&self) -> u8 {
		8
	}

	fn height(&self) -> u8 {
		8
	}

	fn legal_moves(&self) -> Vec<Self::M> {
		self.legal_moves()
	}
	fn play(&mut self, mv: Self::M) {
		self.play(&mv);
	}
	#[inline(always)]
	fn result(&self) -> crate::common::GameResult {
		self.status()
	}

	fn current_player(&self) -> crate::common::Player {
		match self.turn() {
			Color::White => crate::common::Player::PLAYER1,
			Color::Black => crate::common::Player::PLAYER2,
		}
	}
	fn get_name(&self, p: crate::common::Player) -> String {
		match p {
			crate::common::Player::PLAYER1 => "White".into(),
			crate::common::Player::PLAYER2 => "Black".into(),
			_ => unreachable!(),
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		let sq = Square::from_coords(x as usize,y as usize);
		let color = self.color_at(sq)?;
		if color == Color::White {
			match self.piece_at(sq)? {
				super::Piece::Pawn => Some(ChessPiece::WhitePawn),
				super::Piece::Rook => Some(ChessPiece::WhiteRook),
				super::Piece::Knight => Some(ChessPiece::WhiteKnight),
				super::Piece::Bishop => Some(ChessPiece::WhiteBishop),
				super::Piece::Queen => Some(ChessPiece::WhiteQueen),
				super::Piece::King => Some(ChessPiece::WhiteKing),
			}
		} else {
			match self.piece_at(sq)? {
				super::Piece::Pawn => Some(ChessPiece::BlackPawn),
				super::Piece::Rook => Some(ChessPiece::BlackRook),
				super::Piece::Knight => Some(ChessPiece::BlackKnight),
				super::Piece::Bishop => Some(ChessPiece::BlackBishop),
				super::Piece::Queen => Some(ChessPiece::BlackQueen),
				super::Piece::King => Some(ChessPiece::BlackKing),
			}
		}
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		Bitboard8x8::index_from_coords(x, y) as u16
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		Bitboard8x8::coords_from_index(index as usize)
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
	fn move_to_string(&self, mv: &Self::M) -> Option<String> {
		self.move_to_san(mv).ok()
	}
	fn game_to_string(&self, mvs: &[Self::M]) -> Option<String> {
		let mut game= "".to_owned();
		for (i, m) in mvs.iter().enumerate() {
			let m_str=self.move_to_san(m).ok()?;
			game += ((i+1).to_string()+"." + m_str.as_str()+" ").as_str();
		}
		Some(game)
	}
	fn get_position_from_string(&self, pos_str: &String) -> Result<Self, String> {
		Self::from_fen(pos_str)
	}
	fn move_from_string(&self, m_str: &String) -> Result<Self::M, String> {
		let res= self.san_to_move(m_str);
		if res.is_err() {
			let mv = Move::from_uci(m_str);
			if let Some(m) = mv && self.legal_moves().contains(&m) {
				return Ok(m);
			}
		}
		res
	}
	fn game_from_string(&self, _game_str: &String) -> Result<Vec<Self::M>, String> {
		Err("Not Supported".into())
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			dark_color: egui::Color32::from_rgb(105, 105, 185),
			light_color: Color32::from_rgb(240, 240, 250),
			show_coordinates_mod: crate::common::gui::CoordMod::FileRankOnSquare,
			played_highlights_shape: Shape::Rect {
				fill_color: Some(Color32::from_rgba_unmultiplied(120, 120, 120, 128)),
				size: 1.0,
				text: None,
				stroke: None
			},
			..Default::default()
		}
	}
}

pub fn create_board() -> GenericBoardApp<ChessBoard> {
	let engines: Vec<Box<dyn AIEngineProvider<ChessBoard, Engine=Box<dyn AIEngine<ChessBoard>>>>> = vec![
		Box::new(MoveSearcherBuilderDyn::new("Material".into(), ChessMaterialEval{}, 5)),
		Box::new(MoveSearcherBuilderDyn::new("Simple".into(), ChessPosEval{}, 5)),
	];
	GenericBoardApp::new(ChessBoard::default(), engines)
}