use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::{abalone::{game::AbaloneMaterialEval, rules::HEXES}, common::{Player, PlayerType, ai::AIEngine, new_move_searcher}};

use super::rules::{Hex, Abalone, Cell, Move};

pub struct AbaloneApp {
	game: Abalone,
	computer : Box<dyn AIEngine<Abalone>>,
	selected: SelectionState,
	pub players: [PlayerType;2],
}
impl Default for AbaloneApp {
	fn default() -> Self {
		Self {
			game: Abalone::default(),
			computer: new_move_searcher(AbaloneMaterialEval{}, 4),
			selected: SelectionState::None,
			players: [PlayerType::default(), PlayerType::Computer],
			
		}
	}
}
impl AbaloneApp {
	pub fn new() -> Self {
		Default::default()
	}
}
impl AbaloneApp {
	pub fn is_current_player_computer(&self) -> bool {
		match self.game.turn {
			Player::Player1 => self.players[0].is_computer(),
			Player::Player2 => self.players[1].is_computer(),
			_ => unreachable!(),
		}
	}
}
impl eframe::App for AbaloneApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show_inside(ui, |ui| {
			ui.heading("Abalone");

			let available = ui.available_rect_before_wrap();
			let painter = ui.painter_at(available);

			let center = available.center();
			let cell_radius = 22.0;   // taille des billes
			let size = 28.0;          // taille de la cellule hex


			let board_color = Color32::from_rgb(0xaf, 0x69 ,0x49);
			let outline_color = Color32::from_rgb(0x70, 0x20, 30);
			let selection_color = Color32::from_rgb(240, 220, 80);

			// collecter les clics
			let mut clicked_hex: Option<Hex> = None;
			
			// 1. Récupérer la bille sélectionnée
			let selected_hex = match self.selected {
				SelectionState::PieceSelected(h) => Some(h),
				_ => None,
			};

			for hex in HEXES {
				let cell = self.game.cell(hex);
				if cell.is_none() {
					continue;
				}
				let cell = cell.unwrap();
				let pos = hex_to_screen(hex, center, size);

				// fond de case
				painter.circle_filled(pos, cell_radius, board_color);
				painter.circle_stroke(pos, cell_radius, egui::Stroke::new(1.5, outline_color));

				// bille
				if let Some(c) = match cell {
					Cell::Empty => None,
					Cell::Black => Some(Color32::BLACK),
					Cell::White => Some(Color32::WHITE),
				} {
					painter.circle_filled(pos, cell_radius * 0.8, c);
				}
				if let SelectionState::PieceSelected(sel) = self.selected {
					if sel == hex {
						painter.circle_stroke(
							pos,
							cell_radius * 0.9,
							Stroke::new(3.0, selection_color),
						);
					}
				}
				if let SelectionState::ChoosingSupplement { from, line_dir, len } = self.selected {
					// 1. Surligner la ligne complète
					let mut pos_hex = from;
					for _ in 0..len {
						let pos = hex_to_screen(pos_hex, center, size);
						painter.circle_stroke(
							pos,
							cell_radius * 0.8,
							Stroke::new(3.0, Color32::LIGHT_BLUE),
						);
						pos_hex = pos_hex.add(Hex::DIRS[line_dir as usize]);
					}

					// 2. Surligner les cases latérales possibles (à partir des legal moves)
					let mut legal = Vec::new();
					self.game.legal_moves_inplace(&mut legal);

					for mv in legal {
						if let Move::Side { start, line_dir: ld, side_dir, len: l } = mv {
							// On ne garde que les side moves correspondant à la ligne sélectionnée
							if start == from && ld == line_dir && l == len {
								let target = from.add(Hex::DIRS[side_dir as usize]);
								let pos = hex_to_screen(target, center, size);
								painter.circle_stroke(
									pos,
									cell_radius * 0.6,
									Stroke::new(3.0, Color32::YELLOW),
								);
							}
						}
					}
				}

				// interaction
				let rect = Rect::from_center_size(pos, Vec2::splat(cell_radius * 2.0));
				let response = ui.interact(rect, ui.id().with((hex.q, hex.r)), egui::Sense::click());
				if response.clicked() {
					clicked_hex = Some(hex);
				}
			}
			if let Some(sel) = selected_hex {
				let mut legal = Vec::new();
				self.game.legal_moves_inplace(&mut legal);

				for mv in legal {
					match mv {
						// 1) SIMPLE MOVE : 1 bille → 1 case
						Move::Simple { from, dir } if from == sel => {
							let target = from.add(Hex::DIRS[dir as usize]);
							let pos = hex_to_screen(target, center, size);
							painter.circle_stroke(pos, cell_radius * 0.6, Stroke::new(2.0, Color32::YELLOW));
						}

						// 2) PUSH INLINE : 2–3 billes → 1 case dans l’axe
						Move::Push { start, dir, len } if start == sel => {
							let d = Hex::DIRS[dir as usize];
							let target = start.add(Hex { q: d.q * len as i8, r: d.r * len as i8 });
							let pos = hex_to_screen(target, center, size);
							painter.circle_stroke(pos, cell_radius * 0.6, Stroke::new(2.0, Color32::YELLOW));
						}

						// 3) SIDE MOVE : 2–3 billes → déplacement latéral
						Move::Side { start, line_dir, len, .. } => {
							// Calcul des deux extrémités
							let mut end = start;
							for _ in 1..len {
								end = end.add(Hex::DIRS[line_dir as usize]);
							}

							// Si la bille sélectionnée est l'une des extrémités
							if sel == start || sel == end {
								// Surligner l'autre extrémité
								let other = if sel == start { end } else { start };
								let pos = hex_to_screen(other, center, size);
								painter.circle_stroke(pos, cell_radius * 0.6, Stroke::new(2.0, Color32::YELLOW));
							}
						}



						_ => {}
					}
				}

			}
			if !self.is_current_player_computer() {
				// appliquer le clic après la boucle
				if let Some(h) = clicked_hex {
					self.on_click(h);
				}
 			} else if !self.game.is_over() {
				let mv = self.computer.choose_move(&self.game);
				self.game.play(mv.unwrap());
			}

			ui.label(format!("Tour : {:?}", self.game.turn));
			ui.label(format!("Black out: {}", self.game.black_out));
			ui.label(format!("White out: {}", self.game.white_out));
		});
	}
}

fn hex_to_screen(h: Hex, center: Pos2, size: f32) -> Pos2 {
	let x = (h.q as f32 * 1.732 * size) + (h.r as f32 * 0.866 * size);
	let y = h.r as f32 * 1.5 * size;

	Pos2::new(center.x + x, center.y + y)
}


impl AbaloneApp {
	fn on_click(&mut self, h: Hex) {
		match self.selected {
			SelectionState::None => {
				// sélectionner une bille du joueur courant
				if let Some(cell) = self.game.cell(h) {
					if cell == Cell::from_player(self.game.turn) {
						self.selected = SelectionState::PieceSelected(h);
					}
				}
			}
			SelectionState::PieceSelected(from) => {
				if let Some(clicked_dir) = dir_index(from, h) {

					// 1) Clic sur une autre bille du joueur → définir la ligne
					if self.game.cell(h) == Some(Cell::from_player(self.game.turn)) {
						let dq = h.q - from.q;
						let dr = h.r - from.r;
						let steps = dq.abs().max(dr.abs());
						if steps > 0 {
							let dir = Hex { q: dq / steps, r: dr / steps };
							if let Some(line_dir) = Hex::DIRS.iter().position(|&d| d == dir) {
								let len = (steps + 1) as u8;
								self.selected = SelectionState::ChoosingSupplement {
									from,
									line_dir: line_dir as u8,
									len,
								};
								return;
							}
						}
					}

					// 2) Sinon : simple move ou push ou side move direct
					// On cherche dans legal_moves
					let mut legal = Vec::new();
					self.game.legal_moves_inplace(&mut legal);

					for mv in legal {
						match mv {
							Move::Simple { from: f, dir } if f == from && dir == clicked_dir => {
								println!("{}", mv);
								if self.game.play(mv) {
									self.selected = SelectionState::None;
									return;
								}
							}

							Move::Push { start, dir, .. } if start == from && dir == clicked_dir => {
								println!("{}", mv);
								if self.game.play(mv) {
									self.selected = SelectionState::None;
									return;
								}
							}

							_ => {}
						}
					}
				}

				self.selected = SelectionState::None;
			}
			SelectionState::ChoosingSupplement { from, line_dir, len } => {
				if let Some(side_dir) = dir_index(from, h) {
					let mv = Move::Side {
						start: from,
						line_dir,
						side_dir,
						len,
					};
					println!("{}", mv);
					if self.game.play(mv) {
						self.selected = SelectionState::None;
						return;
					}
				}

				self.selected = SelectionState::None;
			}


		}
	}
}

fn dir_index(from: Hex, to: Hex) -> Option<u8> {
	let dq = to.q - from.q;
	let dr = to.r - from.r;

	// Si le mouvement n'est pas sur une ligne hexagonale, impossible
	if dq != 0 && dr != 0 && dq + dr != 0 {
		return None;
	}

	// Normaliser le delta pour obtenir une direction unitaire
	let steps = dq.abs().max(dr.abs());
	if steps == 0 {
		return None;
	}

	let ndq = dq / steps;
	let ndr = dr / steps;

	let delta = Hex { q: ndq, r: ndr };

	for (i, d) in Hex::DIRS.iter().enumerate() {
		if *d == delta {
			return Some(i as u8);
		}
	}
	None
}
/*fn hex_distance(a: Hex, b: Hex) -> u8 {
	let dq = (a.q - b.q).abs() as u8;
	let dr = (a.r - b.r).abs() as u8;
	let ds = (a.q + a.r - b.q - b.r).abs() as u8;
	(dq + dr + ds) / 2
}*/

#[derive(PartialEq, Default)]
enum SelectionState {
	#[default]
	None,
	PieceSelected(Hex),        // on a sélectionné une pièce à déplacer
	ChoosingSupplement {
		from: Hex,
		line_dir: u8,
		len: u8,
	}
}
