use eframe::egui;

use crate::common::ai::AIEngine;
use crate::common::{PlayerType, new_move_searcher};
use super::game::ReversiEval;
use super::{Cell, Reversi};

pub struct ReversiApp {
	board: Reversi,
	computer : Box<dyn AIEngine<Reversi>>,
	pub players: [PlayerType;2],
}
impl Default for ReversiApp {
	fn default() -> Self {
		let board = Reversi::default();

		Self {
			board,
			computer: new_move_searcher(ReversiEval{}, 5),
			players:[PlayerType::default(), PlayerType::Computer],
			
		}
	}
}
impl ReversiApp {
	fn handle_click(&mut self, row: usize, col: usize) {
		let mut out=vec![];
		if self.board.is_legal_move(col as u8, row as u8) {
			self.on_move(row, col);
		} else {
			self.board.legal_moves(&mut out);
			if out.len() == 1 && out[0].0 > 7 {
				self.board.pass();
			} else {
				println!("possible move {:?}: clicked: {}, {}", out, row, col);
			}
		}
	}

	/// À remplacer par ton moteur Reversi
	fn on_move(&mut self, row: usize, col: usize) {
		// Exemple minimal : place un pion si la case est vide
		if self.board.cell_from_coords(col as u8, row as u8) == Cell::Empty {
			self.board.play_unchecked(col as u8, row as u8);
		}
	}
	pub fn is_current_player_computer(&self) -> bool {
		match self.board.turn() {
			Cell::White => self.players[1].is_computer(),
			Cell::Black => self.players[0].is_computer(),
			Cell::Empty => unreachable!(),
		}
	}
}
impl eframe::App for ReversiApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.heading("Reversi / Othello");

			ui.horizontal(|ui| {
				if self.board.is_over() {
					ui.label(format!(
						"Partie terminé. Vainqueur : {}",
						match self.board.winner() {
							Some(Cell::Black) => "Noir",
							Some(Cell::White) => "Blanc",
							_ => "-",
						}
					));
				} else {
					ui.label(format!(
						"Joueur courant : {}",
						match self.board.turn() {
							Cell::Black => "Noir",
							Cell::White => "Blanc",
							_ => "-",
						}
					));
				}
				if ui.button("Réinitialiser").clicked() {
					*self = ReversiApp::default();
				}
			});

			ui.separator();

			// --- Dimensions ---
			let avail_w = ui.available_width();
			let avail_h = ui.available_height();
			let board_size = avail_w.min(avail_h) * 0.95;
			let cell_size = board_size / 8.0;

			let (rect, response) =
				ui.allocate_exact_size(egui::vec2(board_size, board_size), egui::Sense::click());
			let painter = ui.painter_at(rect);

			if !self.is_current_player_computer() {
				// --- Gestion du clic ---
				if let Some(pos) = response.interact_pointer_pos() {
					if rect.contains(pos) && response.clicked() {
						let x = pos.x - rect.left();
						let y = pos.y - rect.top();

						let col = (x / cell_size).floor() as usize;
						let row = (y / cell_size).floor() as usize;

						if row < 8 && col < 8 {
							self.handle_click(row, col);
						}
					}
				} 
			} else if !self.board.is_over() {
				let mv = self.computer.choose_move(&self.board);
				if let Some((x, y)) = mv {
					self.board.play_unchecked(x, y);
				} else if self.board.is_over() {
					self.board.pass();
				}
			}
			

			// --- Dessin du plateau ---
			for row in 0..8 {
				for col in 0..8 {
					let x = rect.left() + col as f32 * cell_size;
					let y = rect.top() + row as f32 * cell_size;

					let square = egui::Rect::from_min_size(
						egui::pos2(x, y),
						egui::vec2(cell_size, cell_size),
					);

					// Couleur du plateau (vert Othello)
					painter.rect_filled(
						square,
						0.0,
						egui::Color32::from_rgb(0, 120, 0),
					);

					// Bordure
					painter.rect_stroke(
						square,
						0.0,
						egui::Stroke::new(1.0, egui::Color32::BLACK),
						egui::StrokeKind::Middle,
					);

					// Pion
					match self.board.cell_from_coords(col as u8, row as u8) {
						Cell::Black => {
							painter.circle_filled(
								square.center(),
								cell_size * 0.4,
								egui::Color32::BLACK,
							);
						}
						Cell::White => {
							painter.circle_filled(
								square.center(),
								cell_size * 0.4,
								egui::Color32::WHITE,
							);
							painter.circle_stroke(
								square.center(),
								cell_size * 0.4,
								egui::Stroke::new(2.0, egui::Color32::BLACK),
							);
						}
						Cell::Empty => {}
					}
				}
			}
		});
	}
}
