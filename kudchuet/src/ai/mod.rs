pub mod cli_engine;
pub mod engine_manager;
pub mod external_engine;
pub mod internal_engine;
pub mod minimax;
pub mod uci;

use egui_field_editor::EguiInspect;
use minimax::IterativeOptions;
#[cfg(not(target_arch = "wasm32"))]
use minimax::ParallelOptions;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, mem::discriminant, pin::Pin, time::Duration};

use crate::ai::internal_engine::InternalEngine;
use crate::ai::minimax::perfect_solver::PerfectSolver;
use crate::ai::uci::{UciOptionConfig, UciValue};
use crate::gui::{BoardGame, BoardMove};
use crate::{MoveSearcher, StrategyWithOptions, new_move_searcher_static};
pub trait AIEngine<G: BoardGame + Sync>: Send
where
	G::M: BoardMove<G> + Send,
{
	fn get_options(&self) -> Option<&HashMap<String, UciValue>>;
	fn get_options_mut(&mut self) -> Option<&mut HashMap<String, UciValue>>;
	fn set_options(&mut self, options: HashMap<String, UciValue>);
	fn set_position(&self, game: &G);

	fn choose_move(&self, game: &G) -> Option<G::M>;
	fn choose_move_async(&mut self, game: G) -> Pin<Box<dyn Future<Output = Option<G::M>> + Send>>;

	fn set_depth_or_timeout(&mut self, depth: u8, timeout: Duration);
	fn set_max_depth(&mut self, depth: u8) {
		self.set_depth_or_timeout(depth, Duration::new(0, 0));
	}
	fn set_timeout(&mut self, timeout: Duration) {
		self.set_depth_or_timeout(99, timeout);
	}
	fn stop_thinking(&self);
}
/*
impl<G> AIEngine<G> for Box<dyn AIEngine<G>>
where
	G: BoardGame + Sync,
	G::M: BoardMove<G> + Send,
{
	fn get_options(&self) -> Option<&HashMap<String, UciValue>> {
		(**self).get_options()
	}
	fn get_options_mut(&mut self) -> Option<&mut HashMap<String, UciValue>> {
		(**self).get_options_mut()
	}

	fn set_options(&mut self, options: HashMap<String, UciValue>) {
		(**self).set_options(options)
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
	fn choose_move_async(&mut self, game: G) -> Pin<Box<dyn Future<Output = Option<G::M>> + Send>> {
		(**self).choose_move_async(game)
	}
	fn stop_thinking(&self) {
		(**self).stop_thinking();
	}
}

pub trait AIEngineProvider<G: BoardGame + Sync>
where
	G::M: BoardMove<G> + Send,
{
	type Engine: AIEngine<G>;
	fn get_name(&self) -> &str;

	fn build_engine(&self) -> Self::Engine;
}
*/
pub trait AIEngineProvider<G>: Send + Sync
where
	G: BoardGame + Sync,
	G::M: BoardMove<G> + Send,
{
	fn get_name(&self) -> &str;

	fn build_engine(&self) -> Box<dyn AIEngine<G>>;
}
pub struct AIBuilder<G, AI>
where
	G: BoardGame + Sync,
	G::M: BoardMove<G> + Send,
	AI: StrategyWithOptions<G> + Default,
{
	name: String,
	phantom: std::marker::PhantomData<(G, AI)>,
}

impl<G, AI> AIBuilder<G, AI>
where
	G: BoardGame + Sync,
	G::M: BoardMove<G> + Send,
	AI: StrategyWithOptions<G> + Default,
{
	pub fn new(name: String) -> Self {
		Self {
			name,
			phantom: std::marker::PhantomData,
		}
	}
}
impl<G, AI> AIEngineProvider<G> for AIBuilder<G, AI>
where
	G: BoardGame + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + Sync + Eq + 'static,
	AI: StrategyWithOptions<G> + Default + Send + Sync + 'static,
{
	//type Engine = Box<dyn AIEngine<G>>;
	fn get_name(&self) -> &str {
		&self.name
	}
	fn build_engine(&self) -> Box<dyn AIEngine<G>> {
		let engine = InternalEngine::new(AI::default());
		Box::new(engine)
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
	//type Engine = Box<dyn AIEngine<G>>;
	fn get_name(&self) -> &str {
		&self.name
	}
	fn build_engine(&self) -> Box<dyn AIEngine<G>> {
		let engine = InternalEngine::new(new_move_searcher_static(
			self.evaluator.clone(),
			self.initial_depth,
		));
		Box::new(engine)
	}
}
/*
#[derive(EguiInspect, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct AIOptions {
	pub table_megabyte_size: usize,
	pub max_depth: u8,
	pub max_time: f32,
	pub threads: Option<usize>,
	#[inspect(transparent = true, hashmap(custom_fn = "crate::ai::uci::inspect_uci_value"))]
	pub uci: HashMap<String, UciValue>,
}
impl From<AIOptions> for minimax::IterativeOptions {
	fn from(value: AIOptions) -> Self {
		Self::new().with_table_byte_size(value.table_megabyte_size * 1024 * 1024)
	}
}
impl From<minimax::IterativeOptions> for AIOptions {
	fn from(value: minimax::IterativeOptions) -> Self {
		Self {
			table_megabyte_size: value.table_byte_size / 1024 / 1024,
			max_depth: 15,
			max_time: 0.0,
			threads: None,
			uci: HashMap::new(),
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
			uci: HashMap::new(),
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
		for (k, v) in self.uci.iter_mut() {
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
		for (k, v) in other.uci.iter() {
			if !self.uci.contains_key(k) {
				self.uci.insert(k.clone(), v.clone());
			}
		}
	}
	pub fn set_uci(&mut self, options: &[UciOptionConfig]) {
		for opt in options {
			match opt {
				UciOptionConfig::Check { name, default } => {
					self.uci
						.insert(name.clone(), UciValue::Bool(default.unwrap_or(false)));
				}
				UciOptionConfig::Spin {
					name,
					default,
					min,
					max,
				} => {
					self.uci.insert(
						name.clone(),
						UciValue::Spin(default.unwrap_or(0), *min, *max),
					);
				}
				UciOptionConfig::Combo { name, default, var } => {
					self.uci.insert(
						name.clone(),
						UciValue::Combo(
							default
								.clone()
								.unwrap_or_else(|| var.first().cloned().unwrap_or_default()),
							var.clone(),
						),
					);
				}
				UciOptionConfig::String { name, default } => {
					self.uci.insert(
						name.clone(),
						UciValue::String(default.clone().unwrap_or_default()),
					);
				}
				UciOptionConfig::Button { name } => {
					self.uci.insert(name.clone(), UciValue::Button);
				}
			}
		}
	}
}*/

pub fn eval_to_percent(cp: i16) -> f32 {
	1.0 / (1.0 + -(cp as f32 / 200.0).exp())
}
