#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::clone_on_copy)]

use ai::minimax::SearchStopSignal;
use ai::minimax::Strategy;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::ai::minimax::BEST_EVAL;
use crate::ai::minimax::IterativeOptions;
#[cfg(target_arch = "wasm32")]
use crate::ai::minimax::IterativeSearch;
#[cfg(not(target_arch = "wasm32"))]
use crate::ai::minimax::ParallelOptions;
use crate::ai::minimax::WORST_EVAL;
use crate::ai::minimax::interface::Evaluator;
use crate::ai::minimax::interface::{Evaluation, Game};
#[cfg(not(target_arch = "wasm32"))]
use crate::ai::minimax::ybw::ParallelSearch;
#[cfg(target_arch = "wasm32")]
use crate::ai::uci::UciValue;
#[cfg(not(target_arch = "wasm32"))]
use crate::ai::uci::UciValue;
use crate::ai::{AIEngine, internal_engine::InternalEngine};
use crate::ai::{AIEngineProvider, MoveSearcherBuilder};
use crate::gui::{BoardGame, BoardMove};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PlayerType {
	#[default]
	Human,
	Computer,
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
pub enum PlayerController {
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GameOutcome {
	Player(Player),
	Draw,
	OnGoing,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Player(pub u8);
impl Default for Player {
	fn default() -> Self {
		Player(0)
	}
}
impl Player {
	pub const PLAYER1: Self = Self(0);
	pub const PLAYER2: Self = Self(1);
	pub fn opponent(&self) -> Self {
		match self {
			Self(0) => Self(1),
			Self(1) => Self(0),
			Self(_) => panic!("Opponent called on a multiplayer game"),
		}
	}
	pub fn idx(&self) -> usize {
		self.0 as usize
	}
	pub fn next<G: Game>(&self, state: &G::S) -> Self {
		G::get_next_player(&state)
	}
}
impl From<Player> for GameOutcome {
	fn from(val: Player) -> Self {
		GameOutcome::Player(val)
	}
}
impl std::fmt::Display for Player {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self(id) => write!(f, "Player {}", *id + 1),
		}
	}
}
impl GameOutcome {
	pub const PLAYER1: Self = Self::Player(Player(0));
	pub const PLAYER2: Self = Self::Player(Player(1));
	pub fn player1_wins(&self) -> bool {
		matches!(self, GameOutcome::Player(Player(0)))
	}
	pub fn player2_wins(&self) -> bool {
		matches!(self, GameOutcome::Player(Player(1)))
	}
	pub fn is_draw(&self) -> bool {
		matches!(self, GameOutcome::Draw)
	}
	pub fn is_ended(&self) -> bool {
		!matches!(self, GameOutcome::OnGoing)
	}
	pub fn is_win_for(&self, p: Player) -> bool {
		self == &GameOutcome::Player(p)
	}
	pub fn is_lose_for(&self, p: Player) -> bool {
		match self {
			GameOutcome::Player(player) => player != &p,
			_ => false,
		}
	}
	pub fn evaluate(&self, to_eval: Player) -> Evaluation {
		match *self {
			GameOutcome::Player(p) => {
				if p == to_eval {
					BEST_EVAL
				} else {
					WORST_EVAL
				}
			}
			_ => 0,
		}
	}
}
impl TryFrom<GameOutcome> for Player {
	type Error = String;

	fn try_from(value: GameOutcome) -> Result<Self, Self::Error> {
		match value {
			GameOutcome::Player(p) => Ok(p),
			GameOutcome::Draw => Err("Draw can not be converted to Player".into()),
			GameOutcome::OnGoing => Err("OnGoing can not be converted to Player".into()),
		}
	}
}
pub trait StrategyWithOptions<G>: Strategy<G>
where
	G: Game,
{
	fn get_options(&self) -> HashMap<String, UciValue>;
	fn set_options(&mut self, opts: &HashMap<String, UciValue>);
	fn stop_signal(&self) -> SearchStopSignal {
		SearchStopSignal::new()
	}
}

#[cfg(target_arch = "wasm32")]
pub type MoveSearcher<T> = ai::minimax::IterativeSearch<T>;
#[cfg(not(target_arch = "wasm32"))]
pub type MoveSearcher<T> = ParallelSearch<T>;

#[cfg(not(target_arch = "wasm32"))]
impl<G, E> StrategyWithOptions<G> for MoveSearcher<E>
where
	G: Game,
	E: Evaluator<G = G> + Clone + Sync + Send + 'static + Default + Eq + Debug,
	G::S: Clone + Send + Sync,
	G::M: Eq + Send + Sync + Clone,
{
	fn get_options(&self) -> HashMap<String, UciValue> {
		//let mut opts = AIOptions::from(*self.options());
		//opts.max_depth = self.get_max_depth();
		//opts.max_time = self.get_max_time().as_secs_f32();
		//opts.threads = self.parallel_options().num_threads;
		//opts.table_megabyte_size = self.options().table_byte_size / 1024 / 1024;
		let mut opts=HashMap::new();
		opts
			.insert("Mtdf".into(), UciValue::Bool(self.options().get_mtdf()));
		opts.insert("Hash".into(), UciValue::Spin(self.options().table_byte_size as i64 / 1024 / 1024, Some(0), None));
		opts.insert("Threads".into(), UciValue::Spin(self.parallel_options().num_threads.unwrap_or(0) as i64, Some(0), None));
		opts
			.insert("Timeout".into(), UciValue::Spin(self.get_max_time().as_millis() as i64, Some(0), None));
		opts
			.insert("Depth".into(), UciValue::Spin(self.get_max_depth() as i64, Some(0), None));
		opts
	}

	fn set_options(&mut self, opts: &HashMap<String, UciValue>) {
		println!("reset_with_options {:?}", opts);
		/*let mut iter =
			IterativeOptions::new().with_table_byte_size(opts.table_megabyte_size * 1024 * 1024);

		if Some(&UciValue::Bool(true)) == opts.uci.get("Mdtf") {
			iter = iter.with_mtdf();
		}
		let mut par = ParallelOptions::new();
		par.num_threads = opts.threads;

		*self = ParallelSearch::new(E::default(), iter, par);
		if opts.max_time <= 0.0 {
			self.set_max_depth(opts.max_depth);
		} else {
			self.set_depth_or_timeout(
				opts.max_depth,
				std::time::Duration::from_secs_f32(opts.max_time),
			);
		}*/
		println!("ai {} {:?}", self.get_max_depth(), self.get_max_time());
	}
	//fn stop_search(&self) {
	//	self.stop_signal().stop_search();
	//}
	fn stop_signal(&self) -> SearchStopSignal {
		self.stop_signal()
	}

}

#[cfg(target_arch = "wasm32")]
impl<G, E> StrategyWithOptions<G> for MoveSearcher<E>
where
	G: Game,
	E: Evaluator<G = G> + Default,
	<<E as Evaluator>::G as Game>::S: Clone,
	<<E as Evaluator>::G as Game>::M: Eq + Clone,
{
	fn get_options(&self) -> HashMap<String, UciValue> {
		/*let mut opts = AIOptions::from(*self.options());
		opts.max_depth = self.get_max_depth();
		opts.max_time = self.get_max_time().as_secs_f32();
		opts.table_megabyte_size = self.options().table_byte_size / 1024 / 1024;
		opts.uci
			.insert("Mtdf".into(), UciValue::Bool(self.options().get_mtdf()));
		opts*/
		let mut opts=HashMap::new();
		opts
			.insert("Mtdf".into(), UciValue::Bool(self.options().get_mtdf()));
		opts.insert("Hash".into(), UciValue::Spin(self.options().table_byte_size as i64 / 1024 / 1024, Some(0), None));
		opts
			.insert("Timeout".into(), UciValue::Spin(self.get_max_time().as_millis() as i64, Some(0), None));
		opts
			.insert("Depth".into(), UciValue::Spin(self.get_max_depth() as i64, Some(0), None));
		opts
	}
	fn set_options(&mut self, opts: &HashMap<String, UciValue>) {
		/*let mut iter =
			IterativeOptions::new().with_table_byte_size(opts.table_megabyte_size * 1024 * 1024);

		if Some(&UciValue::Bool(true)) == opts.uci.get("Mdtf") {
			iter = iter.with_mtdf();
		}

		*self = IterativeSearch::new(E::default(), iter);
		println!("ai {} {:?}", self.get_max_depth(), self.get_max_time());
		if opts.max_time <= 0.0 {
			self.set_max_depth(opts.max_depth);
		} else {
			self.set_depth_or_timeout(
				opts.max_depth,
				std::time::Duration::from_secs_f32(opts.max_time),
			);
		}*/
	}
}
#[cfg(not(target_arch = "wasm32"))]
pub fn new_move_searcher_static<G, T>(evaluator: T, initial_depth: u8) -> MoveSearcher<T>
where
	G: BoardGame + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + Sync + Eq + 'static,
	T: Evaluator<G = G> + Default + Clone + Send + Sync + 'static,
{
	let opts = IterativeOptions::new().with_table_byte_size(128 * 1024 * 1024);
	let mut searcher = ParallelSearch::new(evaluator, opts, ParallelOptions::new());
	searcher.set_max_depth(initial_depth);

	searcher
}

#[cfg(target_arch = "wasm32")]
pub fn new_move_searcher_static<G, T>(evaluator: T, initial_depth: u8) -> MoveSearcher<T>
where
	G: BoardGame + Send + Sync + 'static,
	G::M: BoardMove<G> + Copy + Send + Sync + Eq + 'static,
	T: Evaluator<G = G> + Default + Clone + Send + Sync + 'static,
{
	let mut searcher = IterativeSearch::new(
		evaluator,
		IterativeOptions::new().with_table_byte_size(128 * 1024 * 1024),
	);
	searcher.set_max_depth(initial_depth);
	searcher
}

pub mod ai;
pub mod gui;
pub mod utils;
pub mod sgf;