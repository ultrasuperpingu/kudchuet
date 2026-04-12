use egui::Id;
use egui_field_editor::{EguiInspect, EguiInspector, add_button};

#[cfg(not(target_arch = "wasm32"))]
use crate::ai::external_engine::ExternalEngineEntry;
use crate::gui::board_app::GenericBoardApp;
use crate::gui::{BoardGame, BoardMove};

#[derive(PartialEq, Copy, Clone)]
pub(super) enum RightTab {
	GameSettings,
	Settings,
	Theme,
	ImportExport,
	#[cfg(not(target_arch = "wasm32"))]
	ExternalEngine
}
const LABEL_RATIO: f32 = 0.4;

impl<G: BoardGame+Sync+Send+'static> GenericBoardApp<G>
	where G::M : BoardMove<G> + Send
{
	pub(super) fn draw_options_panels(&mut self, ui: &mut egui::Ui) {
		if let Some(tab) = self.open_right_tab {
			egui::Panel::right("right_content_panel")
				.resizable(true)
				.default_size(200.0)
				.size_range(100.0..=800.0)
				.show_inside(ui, |ui| {
					ui.vertical(|ui| {
						ui.heading(match tab {
							RightTab::GameSettings => "🎾 Game Settings",
							RightTab::Theme => "🎨 Theme",
							RightTab::ImportExport => "🖹 Import/Export",
							RightTab::Settings => "⚙ Advance Settings",
							#[cfg(not(target_arch = "wasm32"))]
							RightTab::ExternalEngine => "🖥 External AI Engine",
						});
						ui.separator();
						
						egui::ScrollArea::vertical().show(ui, |ui| {
							match tab {
								RightTab::GameSettings => {
									if self.game_state_manager.get_settings_mut().inspect("", "", LABEL_RATIO, false, ui).changed() {
									}
								}
								RightTab::Settings => {
									let active_engines = self.ai_engine_manager.get_all_engine_names();
									let mut opts_changed = None;
									for engine_name in active_engines {
										if let Ok(engine) = self.ai_engine_manager.ensure_engine(&engine_name) {
											if let Some(mut opts)= engine.get_options() {
												let resp = opts.inspect(engine_name.as_str(), "", LABEL_RATIO, false, ui);
												//let resp = ui.add(EguiInspector::new(&mut opts).id_salt(engine_name.as_str()));
												if resp.changed() {
													println!("Options changed for engine {} {:?}", engine_name, resp.id);
													for (k,v) in  &opts.uci {
														println!("Label {} / id {:?}", k, Id::new(k));
														if Id::new(k) == resp.id {
															println!("Option {} changed to {:?}", k, v);
														}
													}
													opts_changed = Some((engine_name, opts));
												}
											}
										}
									}
									if let Some((engine_name, opts)) = opts_changed {
										if let Some(engine) = self.ai_engine_manager.get_engine_mut(&engine_name) {
											engine.reset_with_options(opts);
											self.ai_engine_manager.save_all_engine_options(ui.ctx());
										}
									}
									if self.ai_engine_manager.is_thinking() {
										egui_field_editor::add_button("Stop Thinking", "Stop thinking and return current best move", false, ui, |_ui| {
											self.ai_engine_manager.stop_thinking();
										});
									}
								}
								RightTab::Theme => {
									//ui.heading("Board Style");
									//if self.board_drawer.get_style_mut().inspect("", "", LABEL_RATIO, false, ui).changed() {
									if ui.add(EguiInspector::new(self.board_drawer.get_style_mut()).id_salt("board_style").with_title("Board Style")).changed() {
										self.board_drawer.save_style(ui.ctx());
									}
									add_button("Default Board Style", "Reset board style to default", false, ui, |ui| {
										*self.board_drawer.get_style_mut() = G::default_style();
										self.board_drawer.save_style(ui.ctx());
									});
									//if ui.button("Default Board Style").clicked() {
									//	*self.board_drawer.get_style_mut() = G::default_style();
									//	self.board_drawer.save_style(ui.ctx());
									//}
									if self.board_drawer.get_piece_drawer().has_custom_properties() {
										let piece_drawer = self.board_drawer.get_piece_drawer_mut();
										//if self.board_drawer.get_piece_drawer_mut().inspect("", "", LABEL_RATIO, false, ui).changed() {
										if ui.add(EguiInspector::new(piece_drawer).id_salt("piece_style").with_title("Piece Style")).changed() {
											//self.board_drawer.save_style(ui.ctx());
										}
										add_button("Default Piece Style", "Reset pieces style to default", false, ui, |_ui| {
											self.board_drawer.get_piece_drawer_mut().set_default();
											//self.board_drawer.save_style(ui.ctx());
										});
										//if ui.button("Default Piece Style").clicked() {
										//	self.board_drawer.get_piece_drawer_mut().set_default();
										//	//self.board_drawer.save_style(ui.ctx());
										//}
									}
								}
								RightTab::ImportExport => {
									let game = self.game().clone();
									self.import_export_panel.ui(ui, &game, &mut |game| {
										self.ai_engine_manager.set_paused(true);
										self.game_state_manager.load(game);
										self.board_drawer.full_reset();
									});
								}
								#[cfg(not(target_arch = "wasm32"))]
								RightTab::ExternalEngine => {
									self.draw_external_engine_panel(ui);
								}
							}
						});
					});
				});
		}

		egui::Panel::right("tab_buttons")
			.resizable(false)
			.exact_size(40.0)
			.frame(egui::Frame::NONE.inner_margin(4.0)) 
			.show_inside(ui, |ui| {
				ui.vertical_centered(|ui| {
					ui.add_space(10.0);
					
					let mut tab_button = |ui: &mut egui::Ui, tab: RightTab, icon: &str| {
						let is_selected = self.open_right_tab == Some(tab);
						let btn = egui::RichText::new(icon).size(20.0);
						if ui.selectable_label(is_selected, btn).clicked() {
							if is_selected {
								self.open_right_tab = None;
							} else {
								self.open_right_tab = Some(tab);
							}
						}
					};
					

					//tab_button(ui, RightTab::GameSettings, "🎲");
					tab_button(ui, RightTab::GameSettings, "🎾");
					ui.add_space(8.0);
					tab_button(ui, RightTab::Theme, "🎨");
					ui.add_space(8.0);
					tab_button(ui, RightTab::ImportExport, "🖹");
					ui.add_space(8.0);
					tab_button(ui, RightTab::Settings, "⚙");
					#[cfg(not(target_arch = "wasm32"))]
					{
						ui.add_space(8.0);
						tab_button(ui, RightTab::ExternalEngine, "🖥");
					}
				});
			});
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn draw_external_engine_panel(&mut self, ui: &mut egui::Ui) {
		ui.label(egui::RichText::new("External Engines").strong());
		ui.add_space(4.0);

		let mut changed = false;
		let mut remove_idx: Option<usize> = None;
		for (i, entry) in self.ai_engine_manager.get_external_providers_mut().iter_mut().enumerate() {
			if entry.inspect(&entry.name.clone(), "", LABEL_RATIO, false, ui).changed() {
				changed = true;
			}

			if ui.button("❌ Delete").clicked() {
				remove_idx = Some(i);
			}
			ui.separator();
		}

		if let Some(i) = remove_idx {
			let _ = self.ai_engine_manager.get_external_providers_mut().remove(i);
			changed = true;
		}

		if ui.button("➕ Add Engine").clicked() {
			self.ai_engine_manager.add_external_provider(ExternalEngineEntry {
				name: "New Engine".into(),
				path: "".into(),
				args: "".into()
			});
		}
		if changed {
			self.ai_engine_manager.save_external_engines(ui.ctx());
		}
	}
}
#[derive(Default)]
pub struct ImportExportPanel {
	pub position_input: String,
	pub last_error: Option<String>,
}

impl ImportExportPanel {
	pub fn ui<G: BoardGame>(
		&mut self, 
		ui: &mut egui::Ui, 
		game: &G,
		on_position_loaded: &mut dyn FnMut(G)
	)
	where <G as minimax::Game>::M: BoardMove<G>
	{
		ui.vertical(|ui| {
			ui.label(egui::RichText::new("Position").strong());
			
			ui.add(egui::TextEdit::multiline(&mut self.position_input)
				//.hint_text("Ex: A valid FEN or equivalent")
				.desired_rows(5)
				.font(egui::TextStyle::Monospace));

			ui.add_space(8.0);

			ui.horizontal(|ui| {
				if ui.button("Show Current Position").clicked() {
					if let Some(pos) = game.position_to_string() {
						self.position_input = pos;
						self.last_error = None;
					} else {
						self.last_error=Some("Not supported".into());
					}
				}

				if ui.button("📋 Copy to Clipboard").clicked() {
					ui.ctx().copy_text(self.position_input.clone());
				}
			});

			ui.separator();
			if let Some(err) = &self.last_error {
				ui.label(egui::RichText::new(format!("Invalid input: {}", err)).color(egui::Color32::RED));
			}
			ui.scope(|ui| {
				ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 80, 40);
				if ui.button(egui::RichText::new("📥 Load").strong().color(egui::Color32::WHITE))
					.on_hover_text("This will reset the current game state!")
					.clicked() 
				{
					match game.get_position_from_string(&self.position_input) {
						Ok(p) => {
							on_position_loaded(p);
							self.last_error = None;
						}
						Err(e) => {
							println!("Error loading FEN: {}", e);
							self.last_error = Some(e);
						}
					}
				}
			});
		});
	}
}