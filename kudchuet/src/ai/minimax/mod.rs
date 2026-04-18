//! Strategy implementations.

use std::sync::{
	Arc,
	atomic::{AtomicBool, Ordering},
};

pub use crate::ai::minimax::interface::*;
pub use crate::ai::minimax::iterative::IterativeSearch;
pub use crate::ai::minimax::mcts::{MCTSOptions, MonteCarloTreeSearch};
pub use crate::ai::minimax::minimax::ExpectiMinimax;
pub use crate::ai::minimax::random::Random;
pub use crate::ai::minimax::ybw::ParallelSearch;

pub mod iterative;
#[cfg(not(target_arch = "wasm32"))]
pub mod mcts;
pub mod minimax;
pub mod random;
#[cfg(not(target_arch = "wasm32"))]
pub mod ybw;

mod common;
pub mod interface;
#[cfg(not(target_arch = "wasm32"))]
mod sync_util;
pub mod table;
pub mod util;

/// A shared signal used to request the termination of an ongoing search.
//#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct SearchStopSignal(pub(super) Arc<AtomicBool>);
//#[cfg(not(target_arch = "wasm32"))]
impl SearchStopSignal {
	#[doc(hidden)]
	pub fn new() -> Self {
		Self::default()
	}
	#[doc(hidden)]
	pub fn from_atomic_bool(abool: Arc<AtomicBool>) -> Self {
		Self(abool)
	}
	/// Requests the search to stop.
	pub fn stop_search(&self) {
		self.0.store(true, Ordering::Relaxed);
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Strategies for when to overwrite entries in the transition table.
pub enum Replacement {
	Always,
	DepthPreferred,
	TwoTier,
	// TODO: Bucket(size)
}

/// Options to use for the iterative search engines.
#[derive(Clone, Copy)]
pub struct IterativeOptions {
	pub table_byte_size: usize,
	pub(super) strategy: Replacement,
	pub(super) null_window_search: bool,
	pub(super) null_move_depth: Option<u8>,
	pub(super) singular_extension: bool,
	pub(super) aspiration_window: Option<Evaluation>,
	pub(super) mtdf: bool,
	pub(super) step_increment: u8,
	pub(super) max_quiescence_depth: u8,
	pub(super) min_reorder_moves_depth: u8,
	pub(super) countermove_table: bool,
	pub(super) countermove_history_table: bool,
	pub verbose: bool,
}

impl IterativeOptions {
	pub fn new() -> Self {
		IterativeOptions {
			table_byte_size: 1 << 20,
			strategy: Replacement::TwoTier,
			null_window_search: true,
			null_move_depth: None,
			singular_extension: false,
			aspiration_window: None,
			mtdf: false,
			step_increment: 1,
			max_quiescence_depth: 0,
			min_reorder_moves_depth: u8::MAX,
			countermove_table: false,
			countermove_history_table: false,
			verbose: false,
		}
	}
	pub fn get_strategy(&self) -> Replacement {
		self.strategy
	}
	pub fn get_null_window_search(&self) -> bool {
		self.null_window_search
	}
	pub fn get_null_move_depth(&self) -> Option<u8> {
		self.null_move_depth
	}
	pub fn get_singular_extension(&self) -> bool {
		self.singular_extension
	}
	pub fn get_aspiration_window(&self) -> Option<Evaluation> {
		self.aspiration_window
	}
	pub fn get_mtdf(&self) -> bool {
		self.mtdf
	}
	pub fn get_step_increment(&self) -> Option<u8> {
		self.null_move_depth
	}
	pub fn get_max_quiescence_depth(&self) -> u8 {
		self.max_quiescence_depth
	}
	pub fn get_min_reorder_moves_depth(&self) -> u8 {
		self.min_reorder_moves_depth
	}
	pub fn get_countermove_table(&self) -> bool {
		self.countermove_table
	}
	pub fn get_countermove_history_table(&self) -> bool {
		self.countermove_history_table
	}
	pub fn get_verbose(&self) -> bool {
		self.verbose
	}
}

impl Default for IterativeOptions {
	fn default() -> Self {
		Self::new()
	}
}

impl IterativeOptions {
	/// Approximately how large the transposition table should be in memory.
	pub fn with_table_byte_size(mut self, size: usize) -> Self {
		self.table_byte_size = size;
		self
	}

	/// What rules to use when choosing whether to overwrite the current value
	/// in the transposition table.
	pub fn with_replacement_strategy(mut self, strategy: Replacement) -> Self {
		self.strategy = strategy;
		self
	}

	/// Whether to add null-window searches to try to prune branches that are
	/// probably worse than those already found. Also known as principal
	/// variation search.
	pub fn with_null_window_search(mut self, null: bool) -> Self {
		self.null_window_search = null;
		self
	}

	/// Whether to attempt to cut off early by seeing if each node is amazing
	/// even after passing the turn to the opponent. Null move search explores
	/// the tree at a depth reduced by this amount.
	pub fn with_null_move_depth(mut self, depth_reduction: u8) -> Self {
		self.null_move_depth = Some(depth_reduction);
		self
	}

	/// Whether to extend a branch of the search (by 1) if there is only one
	/// move (or only one reasonable move).
	pub fn with_singular_extension(mut self) -> Self {
		self.singular_extension = true;
		self
	}

	/// Whether to search first in a narrow window around the previous root
	/// value on each iteration.
	pub fn with_aspiration_window(mut self, window: Evaluation) -> Self {
		self.aspiration_window = Some(window);
		self
	}

	/// Whether to search for the correct value in each iteration using only
	/// null-window "Tests", with the
	/// [MTD(f)](https://en.wikipedia.org/wiki/MTD%28f%29) algorithm.
	/// Can be more efficient if the evaluation function is coarse grained.
	pub fn with_mtdf(mut self) -> Self {
		self.mtdf = true;
		self
	}

	/// Increment the depth by two between iterations.
	pub fn with_double_step_increment(mut self) -> Self {
		self.step_increment = 2;
		self
	}

	/// Enable [quiescence
	/// search](https://en.wikipedia.org/wiki/Quiescence_search) at the leaves
	/// of the search tree.  The Evaluator must implement `generate_noisy_moves`
	/// for the search to know when the state has become "quiet".
	pub fn with_quiescence_search_depth(mut self, depth: u8) -> Self {
		self.max_quiescence_depth = depth;
		self
	}

	/// Enable the Evaluator's move reordering after generating moves for all
	/// nodes at this depth or higher. Reordering can be an expensive
	/// operation, but it could cut off a lot of nodes if done well high in
	/// the search tree.
	pub fn with_min_reorder_moves_depth(mut self, depth: u8) -> Self {
		self.min_reorder_moves_depth = depth;
		self
	}

	/// Enable the countermove table, which reorders to the front moves that
	/// have worked to counter the previous move in other branches.
	pub fn with_countermoves(mut self) -> Self {
		self.countermove_table = true;
		self
	}

	/// Enable the countermove history table. It keeps a counter for moves
	/// that have caused beta cutoffs in other branches, and reorders moves
	/// based on this counter.
	pub fn with_countermove_history(mut self) -> Self {
		self.countermove_history_table = true;
		self
	}

	/// Enable verbose print statements of the ongoing performance of the search.
	pub fn verbose(mut self) -> Self {
		self.verbose = true;
		self
	}
}

#[derive(Default)]
pub(crate) struct Stats {
	pub(crate) nodes_explored: u64,
	pub(crate) total_generate_move_calls: u64,
	pub(crate) total_generated_moves: u64,
}

impl Stats {
	pub(crate) fn reset(&mut self) {
		self.nodes_explored = 0;
		self.total_generate_move_calls = 0;
		self.total_generated_moves = 0;
	}
	pub(crate) fn explore_node(&mut self) {
		self.nodes_explored += 1;
	}

	pub(crate) fn generate_moves(&mut self, num_moves: usize) {
		self.total_generate_move_calls += 1;
		self.total_generated_moves += num_moves as u64;
	}

	#[cfg(not(target_arch = "wasm32"))]
	pub(crate) fn add(&mut self, other: &Self) {
		self.nodes_explored += other.nodes_explored;
		self.total_generate_move_calls += other.total_generate_move_calls;
		self.total_generated_moves += other.total_generated_moves;
	}
}

/// Options to use for the parallel search engine.
#[derive(Clone, Copy)]
pub struct ParallelOptions {
	pub num_threads: Option<usize>,
	pub(crate) serial_cutoff_depth: u8,
	pub background_pondering: bool,
}

impl ParallelOptions {
	pub fn new() -> Self {
		ParallelOptions {
			num_threads: None,
			serial_cutoff_depth: 1,
			background_pondering: false,
		}
	}
}

impl Default for ParallelOptions {
	fn default() -> Self {
		Self::new()
	}
}

impl ParallelOptions {
	/// Set the total number of threads to use. Otherwise defaults to num_cpus.
	pub fn with_num_threads(mut self, num_threads: usize) -> Self {
		self.num_threads = Some(num_threads);
		self
	}

	/// At what depth should we stop trying to parallelize and just run serially.
	pub fn with_serial_cutoff_depth(mut self, depth: u8) -> Self {
		self.serial_cutoff_depth = depth;
		self
	}

	/// Continuing processing during opponent's move.
	pub fn with_background_pondering(mut self) -> Self {
		self.background_pondering = true;
		self
	}

	pub fn num_threads(self) -> usize {
		self.num_threads.unwrap_or_else(num_cpus::get)
	}
}
