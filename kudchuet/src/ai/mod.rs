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
	/// Optional synchronization hook for stateful/external engines.
	///
	/// Stateless engines may ignore this entirely.
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

pub fn eval_to_percent(cp: i16) -> f32 {
	1.0 / (1.0 + -(cp as f32 / 200.0).exp())
}
