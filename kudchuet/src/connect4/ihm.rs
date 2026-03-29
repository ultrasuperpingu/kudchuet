use eframe::egui;
use crate::common::{PlayerType, ai::AIEngine, new_move_searcher};

use super::{Cell, Column, ConnectFour};

const COLS: u8 = 7;
const ROWS: u8 = 6;

pub struct Connect4App {
	board: ConnectFour,
	winner: Option<Cell>,
	computer : Box<dyn AIEngine<ConnectFour>>,
	pub players: [PlayerType;2]
}

impl Default for Connect4App {
	fn default() -> Self {
		Self {
			board: ConnectFour::new(),
			winner: None,
			computer: new_move_searcher(super::game::ConnectFourEval{}, 6),
			players:[PlayerType::default(), PlayerType::Computer]
		}
	}
}
impl Connect4App {
	pub fn reset(&mut self) {
		self.board = ConnectFour::new();
		self.winner = None;
	}

	pub fn drop_piece(&mut self, col: u8) {
		if self.board.is_over() {
			return;
		}

		self.board.play(Column::from_index(col));
		if self.board.is_victory() {
			self.winner = Some(if self.board.player_turn() == Cell::PlayerOne {Cell::PlayerTwo} else {Cell::PlayerOne});
		}
	}
	pub fn is_current_player_computer(&self) -> bool {
		match self.board.player_turn() {
			Cell::Empty => todo!(),
			Cell::PlayerOne => self.players[0].is_computer(),
			Cell::PlayerTwo => self.players[1].is_computer(),
		}
	}
//	pub fn board(&self) -> &ConnectFour {
//		&self.board
//	}
}

impl eframe::App for Connect4App {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.heading("Connect 4");

			ui.horizontal(|ui| {
				if ui.button("New Game").clicked() {
					self.reset();
				}

				ui.label(format!(
					"Turn : {}",
					match self.board.player_turn() {
						Cell::PlayerTwo => "Red",
						Cell::PlayerOne => "Yellow",
						_ => ""
					}
				));

				if let Some(winner) = self.winner {
					ui.label(format!(
						"Winner : {}",
						match winner {
							Cell::PlayerTwo => "Red",
							Cell::PlayerOne => "Yellow",
							_ => ""
						}
					));
				} else if self.board.is_over() {
					ui.label("Game Over (draw)");
				}
			});

			ui.separator();

			// zone pour le plateau
			let available_width = ui.available_width();
			let available_height = ui.available_height();
			let cell_size = (available_width / COLS as f32).max(80.0).min(available_height / ROWS as f32);
			let board_width = cell_size * COLS as f32;
			let board_height = cell_size * ROWS as f32;

			let (rect, response) =
				ui.allocate_exact_size(egui::vec2(board_width, board_height), egui::Sense::click());

			let painter = ui.painter_at(rect);

			// fond bleu du plateau
			painter.rect_filled(rect, 5.0, egui::Color32::from_rgb(0, 0, 150));

			// clic : déterminer la colonne
			if !self.is_current_player_computer() {
				if response.clicked() && !self.board.is_over() {
					if let Some(pos) = response.interact_pointer_pos() {
						let rel_x = pos.x - rect.left();
						let col = (rel_x / cell_size).floor() as u8;
						if col < COLS {
							self.drop_piece(col);
						}
					}
				}
			} else if !self.board.is_over() {
				let mv= self.computer.choose_move(&self.board);
				self.drop_piece(mv.unwrap().index());
		
			}
			// dessine les pions
			for col in 0..COLS {
				for row in 0..ROWS {
					let x = rect.left() + (col as f32 + 0.5) * cell_size;
					// on inverse l’affichage en Y pour que row 0 soit en bas
					let y = rect.bottom() - (row as f32 + 0.5) * cell_size;

					let center = egui::pos2(x, y);
					let radius = cell_size * 0.4;

					let cell_value = self.board.cell(row, col);
					let color = match cell_value {
						Cell::PlayerOne => egui::Color32::YELLOW,
						Cell::PlayerTwo => egui::Color32::RED,
						_ => egui::Color32::from_rgb(20, 20, 80),
					};

					painter.circle_filled(center, radius, color);
					painter.circle_stroke(
						center,
						radius,
						egui::Stroke::new(2.0, egui::Color32::BLACK),
					);
				}
			}
		});
	}
}
