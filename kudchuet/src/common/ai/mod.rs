
pub mod external_engine;
pub mod internal_engine;
pub mod cli_engine;
pub mod uci;
pub mod engine_manager;
pub mod incomplete_info_searcher;

use std::{collections::HashMap, fmt::Debug, mem::discriminant, pin::Pin, time::Duration};
use egui::Id;
use egui_field_editor::EguiInspect;
use minimax::IterativeOptions;
#[cfg(not(target_arch = "wasm32"))]
use minimax::ParallelOptions;
use serde::{Deserialize, Serialize};

use crate::common::{ai::uci::UciOptionConfig, gui::{BoardGame, BoardMove}};

pub trait AIEngine<G: BoardGame + Sync>: Send
where
	G::M: BoardMove<G> + Send,
{
	fn get_options(&self) -> Option<AIOptions>;
	fn reset_with_options(&mut self, options: AIOptions);
	fn set_position(&self, game: &G);

	fn choose_move(&self, game: &G) -> Option<G::M>;
	fn choose_move_async(
		&mut self,
		game: G,
	) -> Pin<Box<dyn Future<Output = Option<G::M>> + Send>>;

	fn set_depth_or_timeout(&mut self, depth:u8, timeout: Duration);
	fn set_max_depth(&mut self, depth:u8) {
		self.set_depth_or_timeout(depth, Duration::new(0, 0));
	}
	fn set_timeout(&mut self, timeout: Duration) {
		self.set_depth_or_timeout(99, timeout);
	}
	fn stop_thinking(&self);
}

impl<G> AIEngine<G> for Box<dyn AIEngine<G>>
where
	G: BoardGame+Sync,
	G::M: BoardMove<G>+Send,
{
	fn get_options(&self) -> Option<AIOptions> {
		(**self).get_options()
	}

	fn reset_with_options(&mut self, options: AIOptions) {
		(**self).reset_with_options(options)
	}

	fn set_position(&self, game: &G) {
		(**self).set_position(game)
	}

	fn choose_move(&self, game: &G) -> Option<G::M> {
		(**self).choose_move(game)
	}

	fn set_depth_or_timeout(&mut self, depth: u8, timeout: Duration) {
		(**self).set_depth_or_timeout(depth, timeout)
	}
	fn choose_move_async(
		&mut self,
		game: G,
	) -> Pin<Box<dyn Future<Output = Option<G::M>> + Send>> {
		(**self).choose_move_async(game)
	}
	fn stop_thinking(&self) {
		(**self).stop_thinking();
	}
}

pub trait AIEngineProvider<G: BoardGame+Sync>
where
	G::M: BoardMove<G>+Send,
{
	type Engine: AIEngine<G>;
	fn get_name(&self) -> &String;

	fn build_engine(&self) -> Self::Engine;
}

pub struct FunctionAIBuilder<G, F, E>
where
	G: BoardGame+Sync,
	G::M: BoardMove<G>+Send,
	F: Fn() -> E,
	E: AIEngine<G>,
{
	name: String,
	builder: F,
	phantom: std::marker::PhantomData<(G, E)>,
}

impl<G, F, E> FunctionAIBuilder<G, F, E>
where
	G: BoardGame+Sync,
	G::M: BoardMove<G>+Send,
	F: Fn() -> E,
	E: AIEngine<G>,
{
	pub fn new(name: String, builder: F) -> Self {
		Self {
			name,
			builder,
			phantom: std::marker::PhantomData,
		}
	}
}

impl<G, F, E> AIEngineProvider<G> for FunctionAIBuilder<G, F, E>
where
	G: BoardGame+Sync,
	G::M: BoardMove<G>+Send,
	F: Fn() -> E,
	E: AIEngine<G>,
{
	type Engine = E;
	fn get_name(&self) -> &String {
		&self.name
	}
	fn build_engine(&self) -> Self::Engine {
		(self.builder)()
	}
}

pub struct MoveSearcherBuilder<G, E>
where
	G: BoardGame,
	G::M: BoardMove<G>,
	E: minimax::Evaluator<G = G> + Default + Clone + Send + Sync + Eq + 'static,
{
	name: String,
	evaluator: E,
	initial_depth: u8,
	phantom: std::marker::PhantomData<G>,
}

impl<G, T> MoveSearcherBuilder<G, T>
where
	G: BoardGame,
	G::M: BoardMove<G>,
	T: minimax::Evaluator<G = G> + Default + Clone + Send + Sync + Eq + 'static,
{
	pub fn new(name: String, evaluator: T, initial_depth: u8) -> Self {
		Self {
			name,
			evaluator,
			initial_depth,
			phantom: std::marker::PhantomData,
		}
	}
}

impl<G, T> AIEngineProvider<G> for MoveSearcherBuilder<G, T>
where
	G: BoardGame + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + Sync + Eq + 'static,
	T: minimax::Evaluator<G = G> + Default + Clone + Send + Sync + Eq + 'static + Debug,
{
	type Engine = crate::common::ai::internal_engine::InternalEngine<G, crate::common::MoveSearcher<T>>;
	fn get_name(&self) -> &String {
		&self.name
	}
	fn build_engine(&self) -> Self::Engine {
		crate::common::ai::internal_engine::InternalEngine::new(crate::common::new_move_searcher_static(self.evaluator.clone(), self.initial_depth))
	}
}

pub struct MoveSearcherBuilderDyn<G, E>
where
	G: BoardGame,
	G::M: BoardMove<G>,
	E: minimax::Evaluator<G = G> + Default + Clone + Send + Sync + Eq + 'static,
{
	name: String,
	evaluator: E,
	initial_depth: u8,
	phantom: std::marker::PhantomData<G>,
}

impl<G, T> MoveSearcherBuilderDyn<G, T>
where
	G: BoardGame,
	G::M: BoardMove<G>,
	T: minimax::Evaluator<G = G> + Default + Clone + Send + Sync + Eq + 'static,
{
	pub fn new(name: String, evaluator: T, initial_depth: u8) -> Self {
		Self {
			name,
			evaluator,
			initial_depth,
			phantom: std::marker::PhantomData,
		}
	}
}

impl<G, T> AIEngineProvider<G> for MoveSearcherBuilderDyn<G, T>
where
	G: BoardGame + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + Sync + Eq + 'static,
	T: minimax::Evaluator<G = G> + Default + Clone + Send + Sync + Eq + 'static + Debug,
{
	type Engine = Box<dyn AIEngine<G>>;
	fn get_name(&self) -> &String {
		&self.name
	}
	fn build_engine(&self) -> Self::Engine {
		let engine = crate::common::ai::internal_engine::InternalEngine::new(crate::common::new_move_searcher_static(self.evaluator.clone(), self.initial_depth));
		Box::new(engine)
	}
}

#[derive(EguiInspect, Clone, PartialEq, Default, Serialize, Deserialize, Debug)]
pub enum UciValue {
	#[default]
	Button,
	Bool(bool),
	Spin(i64, Option<i64>, Option<i64>),
	String(String),
	Combo(String, Vec<String>),
}
impl Into<UciValue> for UciOptionConfig {
	fn into(self) -> UciValue {
		match self {
			UciOptionConfig::Check { name:_, default } => UciValue::Bool(default.map_or(false, |v| v)),
			UciOptionConfig::Spin { name:_, default, min, max } => UciValue::Spin(default.map_or(0, |v| v.into()), min, max),
			UciOptionConfig::Combo { name:_, default, var } => UciValue::Combo(default.map_or("".to_string(), |v| v.into()), var),
			UciOptionConfig::Button { name:_ } => UciValue::Button,
			UciOptionConfig::String { name:_, default } => UciValue::String(default.map_or("".to_string(), |v| v.into())),
		}
	}
}
impl UciValue {
	pub fn to_option_string(&self) -> Option<String> {
		match self {
			UciValue::Button => None,
			UciValue::Bool(v) => Some(v.to_string()),
			UciValue::Spin(v, _, _) => Some(v.to_string()),
			UciValue::String(v) => Some(v.to_string()),
			UciValue::Combo(v, _) => Some(v.to_string()),
		}
	}
	pub fn set_bool(&mut self, val: bool) {
		match self {
			UciValue::Bool(v) => *v = val,
			_ => {},
		}
	}
}
fn inspect_uci_value(
	item: &mut UciValue,
	parent_id: egui::Id,
	label: &str,
	tooltip: &str,
	label_ratio: f32,
	read_only: bool,
	ui: &mut egui::Ui) -> egui::Response
{
	match item {
		UciValue::Button => {
			if ui.button(label).clicked() {
				println!("coucou {:?}", Id::new(label));
				let mut resp = ui.response();
				resp.id = Id::new(label);
				resp.mark_changed();
				resp
			}
			else {
				ui.response()
			}
		},
		UciValue::Bool(v) => {
			let mut resp = v.inspect_with_custom_id(parent_id, label, tooltip, label_ratio, read_only, ui);
			resp.id = Id::new(label);
			resp
		},
		UciValue::Spin(v, min, max) => {
			let mut resp = if let Some(min) = min {
				if let Some(max) = max {
					egui_field_editor::add_number_slider(v, label, tooltip, label_ratio, read_only, *min, *max, ui)
				} else {
					egui_field_editor::add_number_slider(v, label, tooltip, label_ratio, read_only, *min, i64::MAX, ui)
				}
			} else {
				if let Some(max) = max {
					egui_field_editor::add_number_slider(v, label, tooltip, label_ratio, read_only, i64::MIN, *max, ui)
				} else {
					v.inspect(label, tooltip, label_ratio, read_only, ui)
				}
			};
			resp.id = Id::new(label);
			resp
		},
		UciValue::String(v) => {
			let mut resp = v.inspect_with_custom_id(parent_id, label, tooltip, label_ratio, read_only, ui);
			resp.id = Id::new(label);
			resp
		},
		UciValue::Combo(v, var) => {
			let mut index = var.iter().position(|e| e == v).map_or(0, |v| v);
			let mut resp = egui_field_editor::add_combobox(&mut index, label, tooltip, label_ratio, read_only, var, ui);
			if resp.changed() {
				*v = var[index].clone();
			}
			resp.id = Id::new(label);
			resp
		},
	}
}
#[derive(EguiInspect, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct AIOptions {
	pub table_megabyte_size: usize,
	pub max_depth: u8,
	pub max_time: f32,
	pub threads: Option<usize>,
	#[inspect(transparent=true, hashmap(custom_fn="inspect_uci_value"))]
	pub uci: HashMap<String, UciValue>,
}
impl From<AIOptions> for minimax::IterativeOptions {
	fn from(value: AIOptions) -> Self {
		let v = Self::new().with_table_byte_size(value.table_megabyte_size * 1024 * 1024);
		v
	}
}
impl From<minimax::IterativeOptions> for AIOptions {
	fn from(value: minimax::IterativeOptions) -> Self {
		Self {
			table_megabyte_size: value.table_byte_size / 1024 / 1024,
			max_depth:15,
			max_time: 0.0,
			threads: None,
			uci: HashMap::new()
		}
	}
}
impl Default for AIOptions {
	fn default() -> Self {
		Self {
			table_megabyte_size: 128,
			max_depth: 15,
			max_time: 0.0,
			threads: None,
			uci: HashMap::new()
		}
	}
}
impl AIOptions {
	
	#[cfg(not(target_arch = "wasm32"))]
	pub fn new(ioptions: IterativeOptions, poptions: ParallelOptions) -> Self {
		let mut options: AIOptions = ioptions.into();
		options.threads = poptions.num_threads;
		options
	}
	#[cfg(target_arch = "wasm32")]
	pub fn new(ioptions: IterativeOptions) -> Self {
		ioptions.into()
	}
	pub fn table_bytes(&self) -> usize {
		self.table_megabyte_size * 1024 * 1024
	}

	pub fn max_time_duration(&self) -> Option<std::time::Duration> {
		if self.max_time > 0.0 {
			Some(std::time::Duration::from_secs_f32(self.max_time))
		} else {
			None
		}
	}
	pub fn merge(&mut self, other: &Self) {
		for (k,v) in self.uci.iter_mut() {
			let val = other.uci.get(k);
			if let Some(val) = val {
				if discriminant(val) != discriminant(v) {
					*v = val.clone();
				} else {
					match (v, val) {
						(UciValue::Bool(_v1), UciValue::Bool(_v2)) => {
							//nothing to do
						}
						(UciValue::Spin(_v1, min1, max1), UciValue::Spin(_v2, min2, max2)) => {
							//nothing to do
							*min1 = *min2;
							*max1 = *max2;
						}
						(UciValue::String(_s1), UciValue::String(_s2)) => {
							//nothing to do
						}
						(UciValue::Combo(s1, list1), UciValue::Combo(s2, list2)) => {
							//nothing to do
							*list1 = list2.clone();
							if !list2.contains(s1) {
								*s1 = s2.clone();
							}
						}
						(UciValue::Button, UciValue::Button) => {
							// nothing to do
						}
						_ => {
							// impossible
						}
					}
				}
			}
		}
		for (k,v) in other.uci.iter() {
			if !self.uci.contains_key(k) {
				self.uci.insert(k.clone(), v.clone());
			}
		}
	}
	pub fn set_uci(&mut self, options: &[UciOptionConfig]) {
		for opt in options {
			match opt {
				UciOptionConfig::Check { name, default } => {
					self.uci.insert(name.clone(), UciValue::Bool(default.unwrap_or(false)));
				}
				UciOptionConfig::Spin { name, default, min, max } => {
					self.uci.insert(name.clone(), UciValue::Spin(default.unwrap_or(0), *min, *max));
				}
				UciOptionConfig::Combo { name, default, var } => {
					self.uci.insert(name.clone(), UciValue::Combo(default.clone().unwrap_or_else(|| var.get(0).cloned().unwrap_or_default()), var.clone()));
				}
				UciOptionConfig::String { name, default } => {
					self.uci.insert(name.clone(), UciValue::String(default.clone().unwrap_or_default()));
				}
				UciOptionConfig::Button { name } => {
					self.uci.insert(name.clone(), UciValue::Button);
				}
			}
		}
	}
}
