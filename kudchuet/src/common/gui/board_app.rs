
use egui::{Align, Layout, Ui};

use crate::common::gui::{MultipleMoveSelectionResult, BoardDrawer, RightTab};
use crate::common::ai::{AIEngine, AIEngineProvider};
use crate::common::ai::engine_manager::{EngineManager, ThinkingResult};

use super::{BoardGame, BoardMove};
use super::input_handler::{InputHandler, MoveResult};
use super::game_state_manager::GameStateManager;
use super::options_panel::ImportExportPanel;
use super::board_drawer::DefaultBoardDrawer;
use super::{GameResult, Player};


pub struct GenericBoardApp<G: BoardGame+Sync+Send>
	where G::M : BoardMove<G> + Send
{
	pub(super) ai_engine_manager: EngineManager<G>,
	

	pub board_drawer: Box<dyn BoardDrawer<G>>,

	pub input_handler: InputHandler<G>,

	pub(super) game_state_manager: GameStateManager<G>,

	pub depth: u8,
	pub max_depth: u8,
	pub max_time: f32,


	pub(super) open_right_tab: Option<RightTab>,
	pub(super) import_export_panel: ImportExportPanel,
	inited: bool,
}
impl<G: BoardGame+Sync+Send+'static> eframe::App for GenericBoardApp<G>
	where G::M : BoardMove<G>+Send
{
	fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		if !self.inited {
			self.ai_engine_manager.load_engines_parameters(ctx);
			self.board_drawer.load_style(ctx);
			self.inited = true;
		}
	}
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		self.draw_options_panels(ui);
		egui::CentralPanel::default().show_inside(ui, |ui| {
			let result = self.draw_header_panel(ui);
			ui.separator();
			let computer_playing = self.is_current_player_computer();
			if let Some(click) = self.board_drawer.draw_board(ui, &self.game_to_draw(), !computer_playing) {
				match self.input_handler.process(click, &self.game_state_manager, &mut self.board_drawer) {
					MoveResult::Created{mv, ..} => {
						self.game_state_manager.apply_move(mv);
					}
					_ => {
						if self.is_current_player_random() {
							self.game_state_manager.play_random();
						}
					}
				}
			}
			if result == GameResult::OnGoing {
				if self.is_current_player_computer() && !self.ai_engine_manager.is_paused() {
					match self.ai_engine_manager.pool_ai_result() {
						ThinkingResult::NotThinking => {
							if let Err(err) = self.ai_engine_manager.choose_move_async(&self.game().clone()) {
								eprintln!("Error while trying to play: {}", err);
							}
							ui.request_repaint();
						},
						ThinkingResult::FinishedThinking(m) => {
							if let Some(mv) = m {
								self.apply_move(mv);
								ui.request_repaint();
							}
						},
						ThinkingResult::Thinking => {
							//println!("Still thinking");
						}
					}
				}
			}
			if let Some(candidates) = self.input_handler.pending_moves().clone() {
				match self.ask_select_between_multiple_moves(ui.ctx(), &candidates) {
					MultipleMoveSelectionResult::Pending => {}
					MultipleMoveSelectionResult::Cancelled=> {
						self.input_handler.reset(&mut self.board_drawer, &self.game_state_manager);
					}
					MultipleMoveSelectionResult::Selected(mv) => {
						self.apply_move(mv);
						self.input_handler.reset(&mut self.board_drawer, &self.game_state_manager);
					}
				}
			}
		});
	}
}
impl<G: BoardGame + Clone +Sync+Send+ 'static> GenericBoardApp<G>
where G::M: BoardMove<G>+Send
{
	//pub fn new(game: G, ai: Box<dyn AIEngine<G>>) -> Self {
	pub fn new(game: G, internals_ai: Vec<Box<dyn AIEngineProvider<G, Engine = Box<dyn AIEngine<G>>>>>) -> Self {
		Self {
			ai_engine_manager:  EngineManager::new_with_internals(internals_ai),
			//players: [PlayerType::Human, PlayerType::Computer],

			depth: 5,
			max_depth: 15,
			max_time:0.0,
			
			input_handler: InputHandler::new(),
			board_drawer: Box::new(DefaultBoardDrawer::new()),

			game_state_manager: GameStateManager::new(game),

			open_right_tab:None,
			import_export_panel: ImportExportPanel::default(),
			inited: false,
		}
	}
	pub fn game(&self) -> &G {
		self.game_state_manager.game()
	}
	pub fn game_to_draw(&self) -> &G {
		if let Some(g) = self.input_handler.intermediate_state() {
			g
		} else {
			self.game_state_manager.game()
		}
	}
	pub fn legal_moves(&self) -> &[G::M] {
		self.game_state_manager.legal_moves()
	}
	pub(super) fn undo(&mut self) {
		if self.game_state_manager.undo() {
			self.board_drawer.full_reset();
			self.input_handler.reset(&mut self.board_drawer, &self.game_state_manager);
		}
	}
	pub(super) fn redo(&mut self) {
		if self.game_state_manager.redo() {
			self.board_drawer.full_reset();
			self.input_handler.reset(&mut self.board_drawer, &self.game_state_manager);
		}
	}
	pub(super) fn is_current_player_computer(&self) -> bool {
		self.ai_engine_manager.get_player_engine(self.game().current_player()).is_some()
		/*match self.game().current_player() {
			Player::Player1 => self.ai_engine_manager.active_player1_engine.is_some(),
			Player::Player2 => self.ai_engine_manager.active_player2_engine.is_some(),
			//TODO
			Player::Player(_) => false,
			Player::RandomMove => false,
		}*/
	}
	pub(super) fn is_current_player_random(&self) -> bool {
		match self.game().current_player() {
			Player::Player1 => false,
			Player::Player2 => false,
			Player::Player(_) => false,
			Player::RandomMove => true,
		}
	}
	
	pub(super) fn apply_move(&mut self, mv: G::M) {
		println!("playing {mv:?}");
		self.game_state_manager.apply_move(mv);
		self.board_drawer.set_played_highlights( mv.played_highlights(self.game()));
		self.board_drawer.clear_selection();
		self.input_handler.reset(&mut self.board_drawer, &self.game_state_manager)
	}

	fn ask_select_between_multiple_moves(&self, ctx: &egui::Context, candidates: &Vec<G::M>) -> MultipleMoveSelectionResult<G::M> {
		let screen_rect = ctx.content_rect();
	
		// Modal dialog with fade mask
		egui::Area::new(egui::Id::new("modal_layer"))
			.fixed_pos(screen_rect.min)
			.order(egui::Order::Foreground)
			.show(ctx, |ui| {
				//catch clicks on boards
				ui.allocate_response(screen_rect.size(), egui::Sense::click_and_drag());
				// fade out board
				ui.painter().rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(150));
			});
	
		// Selection area
		egui::Area::new(egui::Id::new("select_move_area"))
			.order(egui::Order::Tooltip)
			.anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
			.show(ctx, |ui| {
				egui::Frame::window(ui.style()).show(ui, |ui| {
					ui.set_width(250.0);
					ui.vertical_centered(|ui| {
						ui.heading("Select Move");
						ui.add_space(8.0);
						ui.label("Multiple moves possible:");
						ui.add_space(12.0);
					
						for mv in candidates {
							let label = self.game().move_to_string(&mv)
								.unwrap_or_else(|| format!("{:?}", mv));

							if ui.button(label).clicked() {
								return MultipleMoveSelectionResult::Selected(mv.clone());
							}
						}

						ui.add_space(12.0);
						ui.separator();
						if ui.button("Cancel").clicked() {
							return MultipleMoveSelectionResult::Cancelled;
						}
						MultipleMoveSelectionResult::Pending
					}).inner
				}).inner
			}).inner
	}

	fn draw_header_panel(&mut self, ui: &mut Ui) -> GameResult {
		let result = self.game().result();
		let engines = self.ai_engine_manager.get_all_engine_names();
		ui.horizontal(|ui| {
			ui.vertical(|ui| {
				self.draw_players_list(engines, ui);
			});
			ui.separator();
			if self.ai_engine_manager.is_thinking() {
				ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
					ui.horizontal(|ui| {
						if ui.button("Stop thinking").clicked() {
							self.ai_engine_manager.stop_thinking();
						}
					});
				});
			}
				/*ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
					ui.horizontal(|ui| {
						ui.label("Depth:");
						if ui.add(egui::Slider::new(&mut self.depth, 1..=self.max_depth)).changed() {
							if self.max_time == 0.0 {
								self.ai.set_max_depth(self.depth);
							} else {
								self.ai.set_depth_or_timeout(self.depth, Duration::from_secs_f32(self.max_time));
							}
						}
					});
				});
				ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
					ui.horizontal(|ui| {
						ui.label("Max Time:");
						if ui.add(egui::Slider::new(&mut self.max_time, 0.0..=120.0)).changed() {
							if self.max_time == 0.0 {
								self.ai.set_max_depth(self.depth);
							} else {
								self.ai.set_depth_or_timeout(self.depth, Duration::from_secs_f32(self.max_time));
							}
						}
					});
				});
				ui.separator();
				if ui.button("Clear TT").clicked() {
					let opts = self.ai.get_options().clone();
					self.ai.reset_with_options(opts);
				}*/
			});
	
		ui.separator();
		ui.horizontal(|ui| {
			if ui.button("New Game").clicked() {
				self.game_state_manager.reset(G::default());
				self.board_drawer.full_reset();
			}

			ui.add_enabled(self.game_state_manager.can_undo(), |ui: &mut Ui| {
				if ui.button("↩").clicked() {
					self.undo();
					self.ai_engine_manager.set_paused(true);
				}
				ui.response()
			});

			ui.add_enabled(self.game_state_manager.can_redo(), |ui: &mut Ui| {
				if ui.button("↪").clicked() {
					self.redo();
				}
				ui.response()
			});
			ui.add_enabled(self.ai_engine_manager.is_paused(), |ui: &mut Ui| {
				if ui.button("▶").clicked() {
					self.ai_engine_manager.set_paused(false);
				}
				ui.response()
			});
			ui.separator();
			if ui.button("↕").on_hover_text("Invert view").clicked() {
				self.board_drawer.get_style_mut().mirrored = !self.board_drawer.get_style().mirrored;
			}
			ui.separator();

			// end turn btn
			let exact_matches : Vec<G::M> = self.input_handler.matching_moves().iter().filter(|m| m.click_sequence(self.game()) == self.input_handler.current_clicks().clone()).copied().collect();
			if exact_matches.len() > 0 && ui.button("End Turn").clicked() {
				if exact_matches.len() > 1 {
					self.input_handler.set_pending_moves(exact_matches);
				} else if exact_matches.len() == 1 {
					self.apply_move(exact_matches[0].clone());
				}
			}

			match result {
				GameResult::Player1 => {
					ui.heading(format!("Winner: {}", self.game().get_name(Player::Player1)));
				}
				GameResult::Player2 => {
					ui.heading(format!("Winner: {}", self.game().get_name(Player::Player2)));
				}
				GameResult::Draw => {
					ui.heading("Draw Game");
				}
				_ => {}
			}
		});
		result
	}

	fn draw_players_list(&mut self, engines: Vec<String>, ui: &mut Ui) {
		if self.game().nb_players() == 2 {
			self.draw_player(Player::Player1, &engines, ui);
			self.draw_player(Player::Player2, &engines, ui);
		} else {
			for i in 0..self.game().nb_players() {
				self.draw_player(Player::Player(i), &engines, ui);
			}
		}
	}
	fn draw_player(&mut self, p: Player, engines: &Vec<String>, ui: &mut Ui) {
		let p_name=self.game().get_name(p);
		ui.horizontal(|ui| {
			ui.label(p_name.clone() + ":");

			let current_engine = self.ai_engine_manager.get_player_engine(p).cloned();
			let mut is_computer = current_engine.is_some();

			if ui.selectable_label(!is_computer, "Human").clicked() {
				self.ai_engine_manager.set_player_engine(p, None);
				is_computer = false;
			}

			if ui.selectable_label(is_computer, "Computer").clicked() {
				if !is_computer {
					self.ai_engine_manager.set_player_engine(p, engines.get(0).cloned());
				}
			}

			if is_computer {
				ui.add_space(8.0);

				let current_engine = self.ai_engine_manager.get_player_engine(p).cloned();
				let mut selected_engine = current_engine.clone()
					.unwrap_or_else(|| engines.get(0).cloned().unwrap_or_default());

				egui::ComboBox::from_id_salt(format!("{} Engine", p_name))
					.selected_text(selected_engine.clone())
					.show_ui(ui, |ui| {
						for eng in engines.iter() {
							ui.selectable_value(&mut selected_engine, eng.clone(), eng);
						}
					});

				if Some(&selected_engine) != current_engine.as_ref() {
					self.ai_engine_manager.set_player_engine(p, Some(selected_engine));
				}
			}
		});
	}
}
