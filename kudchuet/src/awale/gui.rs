use eframe::egui;

use crate::awale::game::AwaleMaterialEval;
use crate::common::{Player, PlayerType, ai::AIEngine, gui::{BoardGame, BoardMove, EGUIPieceType}, new_move_searcher};

use super::rules::Awale;



pub struct AwaleApp {
	game: Awale,
	computer : Box<dyn AIEngine<Awale>>,
	selected: Option<usize>,
	pub players: [PlayerType;2],
	
}
impl EGUIPieceType for u8 {

}
impl BoardGame for Awale {
	type PieceType=u8;

	fn width(&self) -> u8 {
		todo!()
	}

	fn height(&self) -> u8 {
		todo!()
	}

	fn current_player(&self) -> Player {
		self.turn
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
}
impl BoardMove<Awale> for usize {
	
}
impl Default for AwaleApp {
	fn default() -> Self {
		Self {
			game: Awale::default(),
			computer: new_move_searcher(AwaleMaterialEval{}, 5),
			selected:None,
			players: [PlayerType::default(), PlayerType::Computer],
			
		}
	}
}
impl AwaleApp {
	pub fn is_current_player_computer(&self) -> bool {
		match self.game.turn {
			Player::Player1 => self.players[0].is_computer(),
			Player::Player2 => self.players[1].is_computer(),
			_ => unreachable!(),
		}
	}
}
impl eframe::App for AwaleApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.heading("Awalé");

			// --- Infos de partie ---
			ui.horizontal(|ui| {
				ui.label(format!("Joueur courant : {:?}", self.game.turn));
				ui.label(format!("Score Bas : {}", self.game.score_bottom));
				ui.label(format!("Score Haut : {}", self.game.score_top));

				if self.game.game_over {
					ui.colored_label(
						egui::Color32::RED,
						"Partie terminée"
					);
				}

				if ui.button("Réinitialiser").clicked() {
					self.game = Awale::default();
					self.selected = None;
				}
			});

			ui.separator();

			// --- Dimensions ---
			let avail_w = ui.available_width();
			let avail_h = ui.available_height();
			let board_w = avail_w * 0.95;
			let board_h = avail_h * 0.7;

			let pit_w = board_w / 8.0; // 6 trous + 2 marges
			let pit_h = board_h / 3.0;

			let (rect, response) =
				ui.allocate_exact_size(egui::vec2(board_w, board_h), egui::Sense::click());
			let painter = ui.painter_at(rect);

			// --- Grands greniers ---
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
			painter.rect_filled(store_bottom_rect, 10.0, egui::Color32::from_rgb(180, 140, 90));

			painter.text(
				store_top_rect.center(),
				egui::Align2::CENTER_CENTER,
				format!("{}", self.game.score_top),
				egui::FontId::proportional(pit_h * 0.6),
				egui::Color32::BLACK,
			);

			painter.text(
				store_bottom_rect.center(),
				egui::Align2::CENTER_CENTER,
				format!("{}", self.game.score_bottom),
				egui::FontId::proportional(pit_h * 0.6),
				egui::Color32::BLACK,
			);

			// --- Dessin des 12 trous ---
			for i in 0..12 {
				let is_top = i >= 6;
				let idx = if is_top { 12 - i-1 } else { i };

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

				painter.text(
					pit_rect.center(),
					egui::Align2::CENTER_CENTER,
					format!("{}", self.game.pit(i)),
					egui::FontId::proportional(pit_h * 0.5),
					egui::Color32::BLACK,
				);
			}

			// --- Gestion du clic ---
			if !self.is_current_player_computer() {
				if let Some(pos) = response.interact_pointer_pos() {
					if response.clicked() {
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

								if !self.game.game_over && self.game.play(i){
									self.selected = Some(i);
								}
							}
						}
					}
				}
			} else if !self.game.is_over() {
				let mv = self.computer.choose_move(&self.game);
				self.game.play_unchecked(mv.unwrap());
			}
		});
	}
}
