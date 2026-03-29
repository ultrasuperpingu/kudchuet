use eframe::egui;
use crate::common::ai::AIEngine;
use crate::common::bitboards::Bitboard6x5;
use crate::common::{PlayerType, new_move_searcher};
use super::game::YoteMaterialEval;
use super::{Yote, Move, Player, GameResult};


pub struct YoteApp {
	game: Yote,
	selection: SelectionState,
	computer : Box<dyn AIEngine<Yote>>,
	pub players: [PlayerType;2],
	depth: u8,
	legal_moves:Vec<Move>
}

impl Default for YoteApp {
	fn default() -> Self {
		let mut inst=Self {
			game: Yote::new(),
			selection: SelectionState::None,
			computer: new_move_searcher(YoteMaterialEval{}, 5),
			players:[PlayerType::Human, PlayerType::Human],
			depth:6,
			legal_moves:vec![]
		};
		inst.computer.set_max_depth(inst.depth);
		inst.legal_moves=inst.game.legal_moves();
		inst
	}
}

impl eframe::App for YoteApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.heading("Yoté");
			ui.label("Configuration");
			ui.horizontal(|ui| {
			ui.label("Blancs :");
			ui.selectable_value(&mut self.players[0], PlayerType::Human, "Humain");
			ui.selectable_value(&mut self.players[0], PlayerType::Computer, "IA"); });
			ui.horizontal(|ui| {
				ui.label("Noirs :");
				ui.selectable_value(&mut self.players[1], PlayerType::Human, "Humain");
				ui.selectable_value(&mut self.players[1], PlayerType::Computer, "IA");
			});
			match self.game.result() {
				GameResult::Player1 => {ui.label("🎉 Les Blancs ont gagné");},
				GameResult::Player2 => {ui.label("🛡️ Les Noirs ont gagné");},
				GameResult::OnGoing => {},
				GameResult::Draw => {ui.label("🛡️ Match nul");}
			}

			ui.separator();
			ui.label("Profondeur :");
			ui.add(egui::Slider::new(&mut self.depth, 1..=15).text("profondeur"));
			self.computer.set_max_depth(self.depth);
			ui.separator();
			if ui.button("🔄 Nouvelle partie").clicked() {
				self.game = Yote::new();
				self.selection = SelectionState::None;
			}
			ui.separator();
			self.draw_reserve(ui, Player::Player1);
			self.draw_board(ui);
			self.draw_reserve(ui, Player::Player2);

		});
	}
}
impl YoteApp {
	pub fn is_current_player_computer(&self) -> bool {
		match self.game.turn {
			Player::Player1 => self.players[0].is_computer(),
			Player::Player2 => self.players[1].is_computer(),
			_ => unreachable!()
		}
	}
	fn draw_reserve(&mut self, ui: &mut egui::Ui, player: Player) {
		let count = match player {
			Player::Player1 => self.game.reserve_white,
			Player::Player2 => self.game.reserve_black,
			_ => unreachable!()
		};

		let color = match player {
			Player::Player1 => egui::Color32::from_rgb(240, 240, 255),
			Player::Player2 => egui::Color32::from_rgb(20, 20, 20),
			_ => unreachable!()
		};

		ui.horizontal(|ui| {
			ui.label(format!("{:?} réserve :", player));

			for _ in 0..count {
				let (rect, response) = ui.allocate_exact_size(
					egui::vec2(20.0, 20.0),
					egui::Sense::click(),
				);

				ui.painter().circle_filled(rect.center(), 8.0, color);
				if let SelectionState::ChoosingSupplement { from, to, ref options } = self.selection {

					if options.contains(&Some(30)) && self.game.turn == player.opponent() {
						// surbrillance
						ui.painter().circle_stroke(rect.center(), 10.0, egui::Stroke::new(2.0, egui::Color32::RED));

						if response.clicked() {
							let mv = Move::Take {
								from,
								to,
								supplement_pawn: Some(30),
							};
							self.game.play(mv);
							self.selection = SelectionState::None;
							self.legal_moves=self.game.legal_moves();
						}
					}
				}
			}
			if let SelectionState::ChoosingSupplement { from, to, ref options } = self.selection {
				if options.contains(&None) && self.game.turn == player.opponent() {
					let (rect, response) = ui.allocate_exact_size(
						egui::vec2(20.0, 20.0),
						egui::Sense::click(),
					);
					let stroke = egui::Stroke::new(2.0, egui::Color32::RED);
					ui.painter().circle_stroke(rect.center(), 10.0, stroke);
					ui.painter().line_segment([rect.left_bottom(),rect.right_top()], stroke);
					if response.clicked() {
						let mv = Move::Take {
							from,
							to,
							supplement_pawn: None,
						};
						self.game.play(mv);
						self.selection = SelectionState::None;
						self.legal_moves=self.game.legal_moves();
					}
				}
			}
		});
	}

	fn draw_board(&mut self, ui: &mut egui::Ui) {
		let white = self.game.white.storage();
		let black = self.game.black.storage();
		
		// Taille dynamique
		let available = ui.available_size();
		let board_size = available.x.min(available.y);
		let cell_size = board_size / 6.0; // <-- 6 colonnes

		ui.vertical_centered(|ui| {
			ui.add_space((available.y - board_size) / 2.0);

			egui::Grid::new("board_grid")
				.spacing([0.0, 0.0])
				.show(ui, |ui| {
					for y in (0..5).rev() {        // <-- 5 lignes
						for x in 0..6 {           // <-- 6 colonnes
							let idx = Bitboard6x5::index_from_coords(x, y) as u8;
							let mask = 1 << idx;

							// Couleur de fond
							let mut bg = egui::Color32::from_gray(220);

							if SelectionState::PieceSelected(idx) == self.selection {
								bg = egui::Color32::from_rgb(180, 200, 255);
							}

							if let SelectionState::PieceSelected(sel) = self.selection {
								if self.legal_moves.iter().any(|m| match m {
									Move::Move { from, to } => *from == sel && *to == idx,
									Move::Take { from, to, .. } => *from == sel && *to == idx,
									Move::Add { .. } => false, // Add n’a pas de from/to
								}) {
									bg = egui::Color32::from_rgb(180, 255, 180);
								}
							}

							// Couleur du pion
							let piece_color = if white & mask != 0 {
								Some(egui::Color32::from_rgb(240, 240, 255))
							} else if black & mask != 0 {
								Some(egui::Color32::from_rgb(20, 20, 20))
							} else {
								None
							};

							// Dessin de la case
							let (rect, response) = ui.allocate_exact_size(
								egui::vec2(cell_size, cell_size),
								egui::Sense::click(),
							);

							ui.painter().rect_filled(rect, 0.0, bg);
							// Dessin de la case avec bordure
							ui.painter().rect(
								rect,
								0.0,
								bg,
								egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
								egui::StrokeKind::Inside
							);


							if let Some(color) = piece_color {
								let radius = cell_size * 0.35;
								ui.painter().circle_filled(rect.center(), radius, color);
							}
							if let SelectionState::ChoosingSupplement { ref options, .. } = self.selection {
								if options.contains(&Some(idx)) {
									ui.painter().circle_stroke(
										rect.center(),
										cell_size * 0.45,
										egui::Stroke::new(3.0, egui::Color32::RED),
									);
								}
							}

							// Interaction
							if !self.is_current_player_computer() {
								if response.clicked() {
									self.handle_click(idx);
								}
							} else if self.game.result() == GameResult::OnGoing {
								let m = self.computer.choose_move(&self.game);
								//println!("{:?}", self.computer.root_value());
								self.game.play(m.unwrap());
								self.legal_moves = self.game.legal_moves();
							}
						}
						ui.end_row();
					}
				});
		});
	}

	fn handle_click(&mut self, idx: u8) {
		//let legal = self.game.legal_moves();

		// 0) Si on est en mode choix du supplément
		if let SelectionState::ChoosingSupplement { from, to, ref options } = self.selection {
			if options.contains(&Some(idx)) {
				// Le joueur choisit quel pion retirer
				let mv = Move::Take {
					from,
					to,
					supplement_pawn: Some(idx),
				};
				self.game.play(mv);
				self.legal_moves=self.game.legal_moves();
			}
			// Dans tous les cas, on sort de ce mode
			self.selection = SelectionState::None;
			return;
		}

		// 1) Si une pièce est déjà sélectionnée
		if let SelectionState::PieceSelected(sel) = self.selection {

			// 1a) Cliquer à nouveau sur la même case → désélection
			if sel == idx {
				self.selection = SelectionState::None;
				return;
			}

			// 1b) Chercher tous les Take possibles vers idx
			let takes: Vec<_> = self.legal_moves.iter().filter_map(|m| match m {
				Move::Take { from, to, supplement_pawn }
					if *from == sel && *to == idx => Some(*supplement_pawn),
				_ => None,
			}).collect();

			// Plusieurs choix → demander au joueur
			if takes.len() > 1 {
				self.selection = SelectionState::ChoosingSupplement {
					from: sel,
					to: idx,
					options: takes,
				};
				return;
			}

			// Un seul choix → jouer directement
			if takes.len() == 1 {
				let mv = Move::Take {
					from: sel,
					to: idx,
					supplement_pawn: takes[0],
				};
				self.game.play(mv);
				self.selection = SelectionState::None;
				self.legal_moves = self.game.legal_moves();
				return;
			}

			// 1c) Essayer un Move simple
			if let Some(mv) = self.legal_moves.iter().find(|m| match m {
				Move::Move { from, to } => *from == sel && *to == idx,
				_ => false,
			}) {
				self.game.play(*mv);
				self.selection = SelectionState::None;
				self.legal_moves = self.game.legal_moves();
				return;
			}

			// 1d) Aucun coup possible → désélection
			self.selection = SelectionState::None;
			return;
		}

		// 2) Aucune case sélectionnée → essayer un Add
		if let Some(mv) = self.legal_moves.iter().find(|m| match m {
			Move::Add { index } => *index == idx,
			_ => false,
		}) {
			self.game.play(*mv);
			self.legal_moves = self.game.legal_moves();
			return;
		}

		// 3) Sinon, sélectionner une pièce du joueur courant
		match self.game.turn {
			Player::Player1 => {
				if (self.game.white.storage() & (1 << idx)) != 0 {
					self.selection = SelectionState::PieceSelected(idx);
				}
			}
			Player::Player2 => {
				if (self.game.black.storage() & (1 << idx)) != 0 {
					self.selection = SelectionState::PieceSelected(idx);
				}
			}
			_ => unreachable!()
		}
	}




}
#[derive(PartialEq)]
enum SelectionState {
	None,
	PieceSelected(u8),        // on a sélectionné une pièce à déplacer
	ChoosingSupplement {
		from: u8,
		to: u8,
		options: Vec<Option<u8>>,     // indices des pions adverses retirables
	}
}
