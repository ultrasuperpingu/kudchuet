use eframe::egui;

use kudchuet::{
	GameOutcome, Player, PlayerType,
	ai::{AIEngine, minimax::Game},
	gui::{BoardGame, BoardMove},
	new_move_searcher,
};

use crate::gui::Place;

use super::mancala::{Mancala, Move};

pub struct MancalaApp {
	game: Mancala,
	computer: Box<dyn AIEngine<Mancala>>,
	selected: Option<usize>,
	pub players: [PlayerType; 2],
}

impl BoardGame for Mancala {
	type PieceType = Move;
	type Settings = kudchuet::gui::DefaultSettings;

	fn width(&self) -> u8 {
		todo!()
	}

	fn height(&self) -> u8 {
		todo!()
	}

	fn current_player(&self) -> Player {
		if self.to_move {
			Player::PLAYER2
		} else {
			Player::PLAYER1
		}
	}

	fn piece_at(&self, _x: u8, _y: u8) -> Option<Self::PieceType> {
		todo!()
	}

	fn index_from_coords(_x: u8, _y: u8) -> u16 {
		todo!()
	}

	fn coords_from_index(_index: u16) -> (u8, u8) {
		todo!()
	}
	fn result(&self) -> GameOutcome {
		<Self as Game>::get_outcome(self)
	}
}
impl BoardMove<Mancala> for Move {}
impl Default for MancalaApp {
	fn default() -> Self {
		Self {
			game: Mancala::default(),
			computer: new_move_searcher(super::mancala::EvaluatorMancala::default(), 5),
			selected: None,
			players: [PlayerType::default(), PlayerType::Computer],
		}
	}
}
impl MancalaApp {
	pub fn is_current_player_computer(&self) -> bool {
		match self.game.to_move {
			false => self.players[0].is_computer(),
			true => self.players[1].is_computer(),
		}
	}
}
impl eframe::App for MancalaApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.heading("Awalé");

			ui.horizontal(|ui| {
				ui.label(format!("Joueur courant : {:?}", self.game.to_move));
				ui.label(format!("Score Bas : {}", self.game.bottom_pit(0)));
				ui.label(format!("Score Haut : {}", self.game.top_pit(0)));

				if Mancala::get_outcome(&self.game).is_ended() {
					ui.colored_label(egui::Color32::RED, "Partie terminée");
				}

				if ui.button("Réinitialiser").clicked() {
					self.game = Mancala::default();
					self.selected = None;
				}
			});

			ui.separator();

			let avail_w = ui.available_width();
			let avail_h = ui.available_height();
			let board_w = avail_w * 0.95;
			let board_h = avail_h * 0.7;

			let pit_w = board_w / 8.0;
			let pit_h = board_h / 3.0;

			let (rect, response) =
				ui.allocate_exact_size(egui::vec2(board_w, board_h), egui::Sense::click());
			let painter = ui.painter_at(rect);

			let store_w = pit_w;
			let store_h = pit_h * 2.0;

			let store_top_rect = egui::Rect::from_min_size(
				egui::pos2(rect.left(), rect.top() + pit_h * 0.5),
				egui::vec2(store_w, store_h),
			);

			let store_bottom_rect = egui::Rect::from_min_size(
				egui::pos2(rect.right() - store_w, rect.top() + pit_h * 0.5),
				egui::vec2(store_w, store_h),
			);

			painter.rect_filled(store_top_rect, 10.0, egui::Color32::from_rgb(180, 140, 90));
			painter.rect_filled(
				store_bottom_rect,
				10.0,
				egui::Color32::from_rgb(180, 140, 90),
			);

			painter.text(
				store_top_rect.center(),
				egui::Align2::CENTER_CENTER,
				format!("{}", self.game.top_pit(0)),
				egui::FontId::proportional(pit_h * 0.6),
				egui::Color32::BLACK,
			);

			painter.text(
				store_bottom_rect.center(),
				egui::Align2::CENTER_CENTER,
				format!("{}", self.game.bottom_pit(0)),
				egui::FontId::proportional(pit_h * 0.6),
				egui::Color32::BLACK,
			);

			for i in 0..12 {
				let is_top = i >= 6;
				let idx = if is_top { 12 - i - 1 } else { i };

				let x = rect.left() + pit_w * (idx as f32 + 1.0);
				let y = if is_top {
					rect.top() + pit_h * 0.2
				} else {
					rect.top() + pit_h * 1.8
				};

				let pit_rect = egui::Rect::from_min_size(
					egui::pos2(x, y),
					egui::vec2(pit_w * 0.9, pit_h * 0.9),
				);

				let color = if Some(i) == self.selected {
					egui::Color32::from_rgb(200, 200, 50)
				} else {
					egui::Color32::from_rgb(210, 170, 120)
				};

				painter.rect_filled(pit_rect, pit_h * 0.4, color);
				let text = if is_top {
					format!("{}", self.game.top_pit(12 - i))
				} else {
					format!("{}", self.game.bottom_pit(6 - i))
				};
				painter.text(
					pit_rect.center(),
					egui::Align2::CENTER_CENTER,
					text,
					egui::FontId::proportional(pit_h * 0.5),
					egui::Color32::BLACK,
				);
			}

			// --- Gestion du clic ---
			if !self.is_current_player_computer() {
				if let Some(pos) = response.interact_pointer_pos() {
					if response.clicked() {
						if self.game.skipped {
							self.game = Mancala::apply(&mut self.game, Place(0)).unwrap();
							return;
						}
						for i in 0..12 {
							let is_top = i >= 6;
							let idx = if is_top { 12 - i - 1 } else { i };

							let x = rect.left() + pit_w * (idx as f32 + 1.0);
							let y = if is_top {
								rect.top() + pit_h * 0.2
							} else {
								rect.top() + pit_h * 1.8
							};

							let pit_rect = egui::Rect::from_min_size(
								egui::pos2(x, y),
								egui::vec2(pit_w * 0.9, pit_h * 0.9),
							);

							if pit_rect.contains(pos) {
								if !Mancala::get_outcome(&self.game).is_ended()
									&& self.game.to_move == is_top
								{
									let mv = if is_top { 12 - i } else { 6 - i } as u8;
									self.game = Mancala::apply(&mut self.game, Place(mv)).unwrap();
									self.selected = Some(i as usize);
								}
							}
						}
					}
				}
			} else if !Mancala::get_outcome(&self.game).is_ended() {
				let mv = self.computer.choose_move(&self.game);
				self.game = Mancala::apply(&mut self.game, mv.unwrap()).unwrap();
			}
		});
	}
}
