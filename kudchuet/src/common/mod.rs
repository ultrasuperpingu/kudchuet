use std::fmt::Debug;
use minimax::Strategy;
#[cfg(not(target_arch = "wasm32"))]
use minimax::strategies::iterative::SearchStopSignal;


use crate::common::ai::{AIEngineProvider, MoveSearcherBuilderDyn};
use crate::common::{ai::{AIEngine, AIOptions, internal_engine::InternalEngine}, gui::{BoardGame, BoardMove}};

pub mod gui;
pub mod bitboards;
pub mod ai;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PlayerType {
	#[default]
	Human,
	Computer 
}
impl PlayerType {
	pub fn is_human(&self) -> bool {
		match self {
			PlayerType::Human => true,
			PlayerType::Computer => false,
		}
	}
	pub fn is_computer(&self) -> bool {
		match self {
			PlayerType::Human => false,
			PlayerType::Computer => true,
		}
	}
}
pub enum PlayerController
{
	Human,
	Engine(usize),
}

impl PlayerController {
	pub fn is_human(&self) -> bool {
		match self {
			PlayerController::Human => true,
			PlayerController::Engine(_) => false,
		}
	}
	pub fn is_computer(&self) -> bool {
		match self {
			PlayerController::Human => false,
			PlayerController::Engine(_) => true,
		}
	}
}
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GameResult {
	#[default]
	Player1,
	Player2,
	Draw,
	OnGoing
}
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Player {
	#[default]
	Player1,
	Player2,
	RandomMove
}
impl Player {
	pub fn opponent(&self) -> Self {
		match self {
			Player::Player1 => Player::Player2,
			Player::Player2 => Player::Player1,
			Player::RandomMove => Player::RandomMove,
		}
	}
	pub fn idx(&self) -> usize {
		match self {
			Player::Player1 => 0,
			Player::Player2 => 1,
			Player::RandomMove => unreachable!(),
		}
	}
}
impl Into<GameResult> for Player {
	fn into(self) -> GameResult {
		match self {
			Player::Player1 => GameResult::Player1,
			Player::Player2 => GameResult::Player2,
			Player::RandomMove => panic!("Result from random move"),
		}
	}
}
impl std::fmt::Display for Player {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Player::Player1 => f.write_str("Player 1"),
			Player::Player2 => f.write_str("Player 2"),
			Player::RandomMove => f.write_str("Random Move"),
		}
	}
}
impl GameResult {
	pub fn is_player1(&self) -> bool {
		matches!(self, GameResult::Player1)
	}
	pub fn is_player2(&self) -> bool {
		matches!(self, GameResult::Player2)
	}
	pub fn is_draw(&self) -> bool {
		matches!(self, GameResult::Draw)
	}
	pub fn is_finished(&self) -> bool {
		!matches!(self, GameResult::OnGoing)
	}
}
pub trait ConcreteStrategy<G>: Strategy<G>
where
	G: minimax::Game,
{
	fn get_options(&self) -> AIOptions;
	fn reset_with_options(&mut self, opts: AIOptions);
	fn root_value(&self) -> minimax::Evaluation;
	fn stop_search(&self) {}
	#[cfg(not(target_arch = "wasm32"))]
	fn stop_signal(&self) -> SearchStopSignal {
		SearchStopSignal::new()
	}
}
#[cfg(target_arch = "wasm32")]
pub type MoveSearcher<T> = minimax::IterativeSearch<T>;
#[cfg(not(target_arch = "wasm32"))]
pub type MoveSearcher<T> = minimax::ParallelSearch<T>;

#[cfg(not(target_arch = "wasm32"))]
impl<G, E> ConcreteStrategy<G> for MoveSearcher<E>
where
	G: minimax::Game,
	E: minimax::Evaluator<G = G> + Clone + Sync + Send + 'static + Default + Eq + Debug,
	G::S: Clone + Send + Sync,
	G::M: Eq + Send + Sync + Clone,
{
	fn get_options(&self) -> AIOptions {
		let mut opts = AIOptions::from(*self.options());
		opts.max_depth = self.get_max_depth();
		opts.max_time = self.get_max_time().as_secs_f32();
		opts.threads = self.parallel_options().num_threads;
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
		let mut par = minimax::ParallelOptions::new();
		par.num_threads = opts.threads;

		*self = minimax::ParallelSearch::new(
			E::default(),
			iter,
			par,
		);
		if opts.max_time <= 0.0 {
			self.set_max_depth(opts.max_depth);
		} else {
			self.set_depth_or_timeout(opts.max_depth, std::time::Duration::from_secs_f32(opts.max_time));
		}
		println!("ai {} {:?}", self.get_max_depth(), self.get_max_time());
	}
	fn stop_search(&self) {
		self.stop_signal().stop_search();
	}
	fn stop_signal(&self) -> SearchStopSignal {
		self.stop_signal()
	}

	fn root_value(&self) -> minimax::Evaluation {
		MoveSearcher::<E>::root_value(self)
	}
}
#[cfg(target_arch = "wasm32")]
impl<G, E> ConcreteStrategy<G> for MoveSearcher<E>
where G: minimax::Game,
	E: minimax::Evaluator<G = G> + Default,
	<<E as minimax::Evaluator>::G as minimax::Game>::S: Clone,
	<<E as minimax::Evaluator>::G as minimax::Game>::M: Eq+Clone
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
		let mut iter = minimax::IterativeOptions::new()
			.with_table_byte_size(opts.table_megabyte_size * 1024 * 1024);

		if Some(&ai::UciValue::Bool(true)) == opts.uci.get("Mdtf") {
			iter = iter.with_mtdf();
		}

		*self = minimax::IterativeSearch::new(
			E::default(),
			iter
		);
		println!("ai {} {:?}", self.get_max_depth(), self.get_max_time());
		if opts.max_time <= 0.0 {
			self.set_max_depth(opts.max_depth);
		} else {
			self.set_depth_or_timeout(opts.max_depth, std::time::Duration::from_secs_f32(opts.max_time));
		}
	}
	fn root_value(&self) -> minimax::Evaluation {
		 MoveSearcher::<E>::root_value(self)
	}
}
#[cfg(target_arch = "wasm32")]
pub fn new_move_searcher<G, T>(evaluator: T, initial_depth: u8) -> Box<dyn AIEngine<G>>
	where
		G: BoardGame + Send+Sync+ 'static,
		G::M: BoardMove<G> + Copy + Eq +Send+ 'static,
		T: minimax::Evaluator<G = G> + Default + Eq + Clone + Send + 'static,
{
	let mut searcher = minimax::IterativeSearch::new(
		evaluator,
		minimax::IterativeOptions::new()
			.with_table_byte_size(32 * 1024 * 1024)
	);
	searcher.set_max_depth(initial_depth);
	Box::new(InternalEngine::new(searcher))
}
#[cfg(target_arch = "wasm32")]
pub fn new_move_searcher_with_opts<T>(opts: minimax::IterativeOptions) -> MoveSearcher<T>
	where T: minimax::Evaluator+Default,
	<<T as minimax::Evaluator>::G as minimax::Game>::M: Eq,
	<<T as minimax::Evaluator>::G as minimax::Game>::S: Clone
{
	minimax::IterativeSearch::new(
		T::default(),
		opts
	)
}
type DynProvider<G> = Box<dyn AIEngineProvider<G, Engine = Box<dyn AIEngine<G> + 'static>>>;
pub fn new_move_searcher_vec<G, T>(name: String, evaluator: T, initial_depth: u8) -> Vec<DynProvider<G>>
where
	G: BoardGame + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + Sync + Eq + 'static,
	T: minimax::Evaluator<G = G> + Default + Eq + Clone + Send + Sync + 'static + Debug,
{
	vec![Box::new(MoveSearcherBuilderDyn::<G, T>::new(name, evaluator, initial_depth))]
}


#[cfg(not(target_arch = "wasm32"))]
pub fn new_move_searcher<G, T>(evaluator: T, initial_depth: u8) -> Box<dyn AIEngine<G>>
where
	G: BoardGame + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + Sync + Eq + 'static,
	T: minimax::Evaluator<G = G> + Default + Eq + Clone + Send + Sync + 'static + Debug,
{
	let mut searcher = minimax::ParallelSearch::new(
		evaluator,
		minimax::IterativeOptions::new()
			.with_table_byte_size(128 * 1024 * 1024),
		minimax::ParallelOptions::new(),
	);
	searcher.set_max_depth(initial_depth);
	Box::new(InternalEngine::new(searcher))
}
#[cfg(not(target_arch = "wasm32"))]
pub fn new_move_searcher_static<G, T>(evaluator: T, initial_depth: u8) -> MoveSearcher<T>
where
	G: BoardGame + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + Sync + Eq + 'static,
	T: minimax::Evaluator<G = G> + Default + Clone + Send + Sync + 'static,
{
	let opts = minimax::IterativeOptions::new().with_table_byte_size(128 * 1024 * 1024);
	let mut searcher = minimax::ParallelSearch::new(
		evaluator,
		opts,
		minimax::ParallelOptions::new(),
	);
	searcher.set_max_depth(initial_depth);

	searcher
}

#[cfg(target_arch = "wasm32")]
pub fn new_move_searcher_static<G, T>(evaluator: T, initial_depth: u8) -> MoveSearcher<T>
where
	G: BoardGame + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + Sync + Eq + 'static,
	T: minimax::Evaluator<G = G> + Default + Clone + Send + Sync + 'static,
{
	let mut searcher = minimax::IterativeSearch::new(
		evaluator,
		minimax::IterativeOptions::new()
			.with_table_byte_size(128 * 1024 * 1024),
	);
	searcher.set_max_depth(initial_depth);
	searcher
}
