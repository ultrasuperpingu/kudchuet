use eframe::egui;
use egui::{Color32, Stroke};
use crate::chinese_checkers::bitboard::ChineseCheckerBoard;
use crate::chinese_checkers::game::ChineseCheckersMaterialEval;
use crate::chinese_checkers::{ChineseCheckers, ChineseCheckersPlayer, Move};
use crate::common::ai::incomplete_info_searcher::ExpectiMinimaxBuilder;
use crate::common::gui::board_app::GenericBoardApp;
use crate::common::gui::board_drawer::SquareDrawer;
use crate::common::gui::{BoardGame, BoardMove, BoardStyle, EGUIPieceType, Shape};


impl BoardMove<ChineseCheckers> for Move {
	fn from(&self) -> Option<u16> {
		Some(self.from as u16)
	}

	fn to(&self) -> u16 {
		self.to as u16
	}
}
impl EGUIPieceType for ChineseCheckersPlayer {
	fn shape(&self) -> Shape {
		match self {
			ChineseCheckersPlayer::Red => Shape::Circle { color: Color32::RED, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
			ChineseCheckersPlayer::Blue => Shape::Circle { color: Color32::BLUE, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
			ChineseCheckersPlayer::Green => Shape::Circle { color: Color32::GREEN, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
			ChineseCheckersPlayer::Yellow => Shape::Circle { color: Color32::YELLOW, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
			ChineseCheckersPlayer::Black => Shape::Circle { color: Color32::BLACK, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
			ChineseCheckersPlayer::White => Shape::Circle { color: Color32::WHITE, size: 0.7, text: "".into(), text_color: Color32::WHITE, stroke_color: None },
		}
	}
}
impl BoardGame for ChineseCheckers {

	type PieceType=ChineseCheckersPlayer;

	fn width(&self) -> u8 {
		13
	}

	fn height(&self) -> u8 {
		17
	}

	fn play(&mut self, mv: Self::M) {
		let _ = self.play(mv);
	}
	#[inline(always)]
	fn result(&self) -> crate::common::GameResult {
		match self.winner() {
			Some(p) => crate::common::GameResult::Player(p.idx()),
			None => crate::common::GameResult::OnGoing,
		}
	}

	fn current_player(&self) -> crate::common::Player {
		match self.turn {
			ChineseCheckersPlayer::Red => crate::common::Player::Player(0),
			ChineseCheckersPlayer::Blue => crate::common::Player::Player(1),
			ChineseCheckersPlayer::Green => crate::common::Player::Player(2),
			ChineseCheckersPlayer::Yellow => crate::common::Player::Player(3),
			ChineseCheckersPlayer::Black => crate::common::Player::Player(4),
			ChineseCheckersPlayer::White => crate::common::Player::Player(5),
		}
	}
	fn nb_players(&self) -> u8 {
		self.nb_players
	}
	fn get_name(&self, p: crate::common::Player) -> String {
		match p {
			crate::common::Player::PLAYER1 => "White".into(),
			crate::common::Player::PLAYER2 => "Black".into(),
			crate::common::Player::Player(idx) => {
				let players = Self::active_players(self.nb_players);
				if idx < players.len() as u8 {
					return match players[idx as usize] {
						ChineseCheckersPlayer::Red => "Red",
						ChineseCheckersPlayer::Blue => "Blue",
						ChineseCheckersPlayer::Green => "Green",
						ChineseCheckersPlayer::Yellow => "Yellow",
						ChineseCheckersPlayer::Black => "Black",
						ChineseCheckersPlayer::White => "White",
					}.into()
				}
				"".into()
			}
			crate::common::Player::RandomMove => unreachable!(),
		}
	}
	fn piece_at(&self, x: u8, y: u8) -> Option<Self::PieceType> {
		if self.black.get(x, y) {
			return  Some(ChineseCheckersPlayer::Black);
		}
		if self.white.get(x, y) {
			return  Some(ChineseCheckersPlayer::White);
		}
		if self.yellow.get(x, y) {
			return  Some(ChineseCheckersPlayer::Yellow);
		}
		if self.green.get(x, y) {
			return  Some(ChineseCheckersPlayer::Green);
		}
		if self.blue.get(x, y) {
			return  Some(ChineseCheckersPlayer::Blue);
		}
		if self.red.get(x, y) {
			return  Some(ChineseCheckersPlayer::Red);
		}
		None
	}

	fn index_from_coords(x: u8, y: u8) -> u16 {
		ChineseCheckerBoard::index_from_coords(x, y) as u16
	}
	fn coords_from_index(index: u16) -> (u8, u8) {
		ChineseCheckerBoard::coords_from_index(index as usize)
	}
	fn default_style() -> BoardStyle {
		BoardStyle {
			uniform_color: egui::Color32::from_rgb(200, 190, 125),
			show_coordinates_mod: crate::common::gui::CoordMod::None,
			played_highlights_shape: Shape::Rect { color: Color32::from_rgba_unmultiplied(120, 120, 120, 128), size: 1.0, text: "".into(), text_color: Color32::BLACK, stroke_color: None },
			half_size_offset_mod: crate::common::gui::HalfSizeOffsetMod::Even,
			clear_color: Some(egui::Color32::from_rgb(200, 190, 125)),
			..Default::default()
		}
	}
	fn get_position_from_string(&self, fen: &String) -> Result<Self, String> {
		Self::from_fen(fen)
	}
	fn position_to_string(&self) -> Option<String> {
		Some(self.to_fen())
	}
}

struct ChineseCheckerSquareDrawer{}
impl SquareDrawer<ChineseCheckers> for ChineseCheckerSquareDrawer {
	fn draw(&self, painter: &egui::Painter, style: &BoardStyle, _game: &ChineseCheckers, square: &egui::Rect, x_coord: u8, y_coord: u8)
	{
		painter.rect_filled(*square, 0.0, style.uniform_color);
		if ChineseCheckerBoard::is_playable(x_coord, y_coord) {
			let actives = ChineseCheckers::active_players(_game.nb_players);
			painter.circle_stroke(square.center(), square.width()*0.7/2.0, Stroke::new(3.0, style.dark_color));
			if actives.contains(&ChineseCheckersPlayer::Red) && ChineseCheckerBoard::initial_blue().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::RED);
			}
			else if actives.contains(&ChineseCheckersPlayer::Blue) && ChineseCheckerBoard::initial_red().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::BLUE);
			}
			else if actives.contains(&ChineseCheckersPlayer::Yellow) && ChineseCheckerBoard::initial_green().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::YELLOW);
			}
			else if actives.contains(&ChineseCheckersPlayer::Green) && ChineseCheckerBoard::initial_yellow().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::GREEN);
			}
			else if actives.contains(&ChineseCheckersPlayer::Black) && ChineseCheckerBoard::initial_white().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::BLACK);
			}
			else if actives.contains(&ChineseCheckersPlayer::White) && ChineseCheckerBoard::initial_black().get(x_coord, y_coord) {
				painter.circle_filled(square.center(), square.width()*0.3/2.0, Color32::WHITE);
			}
		}
	}

	fn draw_overlay(&self, _painter: &egui::Painter, _style: &BoardStyle, _game: &ChineseCheckers, _board_rect: &egui::Rect, _cell_size: f32) {
	}
}
pub fn create_board() -> GenericBoardApp<ChineseCheckers> {
	let ai_provider = ExpectiMinimaxBuilder::new("Material".into(), ChineseCheckersMaterialEval::default(), 4);
	let mut board = GenericBoardApp::new(ChineseCheckers::new(6), vec![Box::new(ai_provider)]);
	board.board_drawer.set_square_drawer(Box::new(ChineseCheckerSquareDrawer{}));
	board
}