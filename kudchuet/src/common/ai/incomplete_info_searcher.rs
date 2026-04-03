
use minimax::{Game, IterativeOptions, StochasticGame, Strategy, TurnBasedGame, TurnBasedGameEvaluator, strategies::expecti_iterative::ExpectiIterativeSearch};
#[cfg(not(target_arch = "wasm32"))]
use minimax::strategies::iterative::SearchStopSignal;

use std::fmt::Debug;
use crate::common::{ConcreteStrategy, ai::{self, AIEngine, AIEngineProvider, AIOptions}, gui::{BoardGame, BoardMove}};


impl<E> ConcreteStrategy<E::G> for ExpectiIterativeSearch<E>
where
	E: TurnBasedGameEvaluator + Default,
	E::G: TurnBasedGame+StochasticGame,
	<E::G as Game>::S: Clone + BoardGame,
	<E::G as Game>::M: Clone + Eq,
	<<E::G as Game>::S as Game>::M: BoardMove<<E::G as Game>::S>
{
	fn get_options(&self) -> AIOptions {
		let mut opts = AIOptions::from(*self.options());
		opts.max_depth = self.get_max_depth();
		opts.max_time = self.get_max_time().as_secs_f32();
		opts.table_megabyte_size = self.options().table_byte_size / 1024 / 1024;
		opts.uci.insert("Mtdf".into(), ai::UciValue::Bool(self.options().get_mtdf()));
		opts
	}

	fn reset_with_options(&mut self, opts: AIOptions) {
		println!("reset_with_options {:?}", opts);
		let mut iter = minimax::IterativeOptions::new()
			.with_table_byte_size(opts.table_megabyte_size * 1024 * 1024);

		if Some(&ai::UciValue::Bool(true)) == opts.uci.get("Mdtf") {
			iter = iter.with_mtdf();
		}
		
		*self = Self::new(
			E::default(),
			iter,
		);
		if opts.max_time <= 0.0 {
			self.set_max_depth(opts.max_depth);
		} else {
			self.set_depth_or_timeout(opts.max_depth, std::time::Duration::from_secs_f32(opts.max_time));
		}
		println!("ai {} {:?}", self.get_max_depth(), self.get_max_time());
	}
	fn stop_search(&self) {
		self.next_search_stop_signal().stop_search();
	}
	#[cfg(not(target_arch = "wasm32"))]
	fn stop_signal(&self) -> SearchStopSignal {
		self.next_search_stop_signal()
	}

	fn root_value(&self) -> minimax::Evaluation {
		0
	}
}


pub struct ExpectiMinimaxBuilder<G, E>
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

impl<G, T> ExpectiMinimaxBuilder<G, T>
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
impl<G, E> AIEngineProvider<G> for ExpectiMinimaxBuilder<G, E>
where
	G: BoardGame+TurnBasedGame+StochasticGame + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + Sync + Eq + 'static,
	E: minimax::Evaluator<G = G> +TurnBasedGameEvaluator+ Default + Clone + Send + Sync + Eq + 'static + Debug,
{
	//type Engine = crate::common::ai::internal_engine::InternalEngine<G, ExpectiMinimaxAlphaBeta<E>>;
	type Engine = Box<dyn AIEngine<G>>;
	fn get_name(&self) -> &String {
		&self.name
	}
	fn build_engine(&self) -> Self::Engine {
		
		let mut ai = ExpectiIterativeSearch::new(self.evaluator.clone(), IterativeOptions::new());
		ai.set_max_depth(self.initial_depth);
		Box::new(crate::common::ai::internal_engine::InternalEngine::new(ai))
	}
}
