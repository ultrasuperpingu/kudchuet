use crate::{GameOutcome, Player};
use crate::ai::minimax::mcts_node::MctsNode;
use crate::ai::minimax::{Game, Strategy};
use ego_tree::{NodeId, NodeRef, Tree};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

/// A trait for random number generation.
///
/// This allows for different random number generation strategies to be used with the MCTS search,
/// which is particularly useful for testing and ensuring reproducibility.
pub trait RandomGenerator: Default + Clone {
	/// Returns the next random `i32`.
	fn next(&mut self) -> i32;
	/// Returns a random `i32` within the specified range (exclusive of `to`).
	fn next_range(&mut self, from: i32, to: i32) -> i32;
}

/// A `RandomGenerator` that uses the `rand` crate for random number generation.
#[derive(Clone, Default)]
pub struct StandardRandomGenerator;

impl RandomGenerator for StandardRandomGenerator {
	fn next(&mut self) -> i32 {
		fastrand::i32(i32::MIN..i32::MAX)
	}

	fn next_range(&mut self, from: i32, to: i32) -> i32 {
		fastrand::i32(from..to)
	}
}
/// Used for alpha-beta pruning to mark nodes as having a definite outcome.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Bound {
	/// The outcome of the node is not yet determined.
	None = 0,
	/// This node is a guaranteed win for the current player.
	DefoWin = 1,
	/// This node is a guaranteed loss for the current player.
	DefoLose = 2,
}
/// The main struct for running the Monte Carlo Tree Search algorithm.
///
/// It holds the search tree, the random number generator, and the configuration for the search.
pub struct MonteCarloTreeSearch<T: Game, K: RandomGenerator>
where
	T::S: Clone,
{
	tree: Tree<MctsNode<T>>,
	root_id: NodeId,
	random: K,
	use_alpha_beta_pruning: bool,
	next_action: MctsAction,
}

/// A builder for creating instances of `MonteCarloTreeSearch`.
///
/// This provides a convenient way to configure the MCTS search with different parameters.
pub struct MonteCarloTreeSearchBuilder<T: Game, K: RandomGenerator>
where
	T::S: Clone,
{
	board: T::S,
	random_generator: K,
	use_alpha_beta_pruning: bool,
}

impl<T: Game, K: RandomGenerator> MonteCarloTreeSearchBuilder<T, K>
where
	T::S: Clone,
{
	/// Creates a new builder with the given initial board state.
	pub fn new(board: T::S) -> Self {
		Self {
			board,
			random_generator: K::default(),
			use_alpha_beta_pruning: true,
		}
	}

	/// Sets the random number generator for the MCTS search.
	pub fn with_random_generator(mut self, rg: K) -> Self {
		self.random_generator = rg;
		self
	}

	/// Enables or disables alpha-beta pruning.
	pub fn with_alpha_beta_pruning(mut self, use_abp: bool) -> Self {
		self.use_alpha_beta_pruning = use_abp;
		self
	}

	/// Builds the `MonteCarloTreeSearch` instance with the configured parameters.
	pub fn build(self) -> MonteCarloTreeSearch<T, K> {
		MonteCarloTreeSearch::new(
			self.board,
			self.random_generator,
			self.use_alpha_beta_pruning,
		)
	}
}

impl<T: Game, K: RandomGenerator> MonteCarloTreeSearch<T, K>
where
	T::S: Clone,
{
	/// Returns a new builder for `MonteCarloTreeSearch`.
	pub fn builder(board: T::S) -> MonteCarloTreeSearchBuilder<T, K> {
		MonteCarloTreeSearchBuilder::new(board)
	}

	/// Creates a new `MonteCarloTreeSearch` instance.
	///
	/// It is recommended to use the builder pattern via `MonteCarloTreeSearch::builder()` instead.
	pub fn new(board: T::S, rg: K, use_alpha_beta_pruning: bool) -> Self {
		let root_mcts_node = MctsNode::new(0, board);
		let tree: Tree<MctsNode<T>> = Tree::new(root_mcts_node);
		let root_id = tree.root().id();

		Self {
			tree,
			root_id: root_id.clone(),
			random: rg,
			use_alpha_beta_pruning,
			next_action: MctsAction::Selection {
				R: root_id.clone(),
				RP: vec![],
			},
		}
	}

	/// Returns an immutable reference to the underlying search tree.
	pub fn get_tree(&self) -> &Tree<MctsNode<T>> {
		&self.tree
	}

	/// Returns the next MCTS action to be performed. Useful for debugging and visualization.
	pub fn get_next_mcts_action(&self) -> &MctsAction {
		&self.next_action
	}

	/// Executes a single step of the MCTS algorithm (Selection, Expansion, Simulation, or Backpropagation).
	pub fn execute_action(&mut self) {
		match self.next_action.clone() {
			MctsAction::Selection { R, RP: _cr } => {
				let maybe_selected_node = self.select_next_node(R);
				self.next_action = match maybe_selected_node {
					None => MctsAction::EverythingIsCalculated,
					Some(selected_node) => MctsAction::Expansion { L: selected_node },
				};
			}
			MctsAction::Expansion { L } => {
				let (children, selected_child) = self.expand_node(L);
				self.next_action = MctsAction::Simulation {
					C: selected_child,
					AC: children,
				};
			}
			MctsAction::Simulation { C, AC: _ac } => {
				let outcome = self.simulate(C);
				self.next_action = MctsAction::Backpropagation { C, result: outcome };
			}
			MctsAction::Backpropagation { C, result } => {
				let affected_nodes = self.backpropagate(C, result);
				self.next_action = MctsAction::Selection {
					R: self.root_id.clone(),
					RP: affected_nodes,
				}
			}
			MctsAction::EverythingIsCalculated => {}
		}
	}

	/// Performs one full iteration of the MCTS algorithm (Selection, Expansion, Simulation, Backpropagation).
	/// Returns the path of nodes that were updated during backpropagation.
	pub fn do_iteration(&mut self) -> Vec<NodeId> {
		self.execute_action();
		let mut is_selection = matches!(self.next_action, MctsAction::Selection { R: _, RP: _ });
		let mut is_fully_calculated =
			matches!(self.next_action, MctsAction::EverythingIsCalculated);
		while !is_selection && !is_fully_calculated {
			self.execute_action();
			is_selection = matches!(self.next_action, MctsAction::Selection { R: _, RP: _ });
			is_fully_calculated = matches!(self.next_action, MctsAction::EverythingIsCalculated);
		}

		match self.next_action.clone() {
			MctsAction::Selection { R: _, RP: rp } => rp,
			_ => vec![],
		}
	}

	/// Runs the MCTS search for a specified number of iterations.
	pub fn iterate_n_times(&mut self, n: u32) {
		let mut iteration = 0;
		while iteration < n {
			self.do_iteration();
			iteration += 1;
		}
	}

	/// Returns a reference to the root node of the search tree.
	pub fn get_root(&self) -> MctsTreeNode<'_, T> {
		let root = self.tree.root();
		root.into()
	}

	/// Selects the most promising node to expand, using the UCB1 formula.
	fn select_next_node(&self, root_id: NodeId) -> Option<NodeId> {
		let mut promising_node_id = root_id.clone();
		let mut has_changed = false;
		loop {
			let mut best_child_id: Option<NodeId> = None;
			let mut max_ucb = f64::MIN;
			let node = self.tree.get(promising_node_id).unwrap();
			for child in node.children() {
				if child.value().is_fully_calculated {
					continue;
				}

				let current_ucb = MonteCarloTreeSearch::<T, K>::ucb_value(
					node.value().visits,
					child.value().wins,
					child.value().visits,
				);
				if current_ucb > max_ucb {
					max_ucb = current_ucb;
					best_child_id = Some(child.id());
				}
			}
			if best_child_id.is_none() {
				break;
			}
			promising_node_id = best_child_id.unwrap();
			has_changed = true;
		}

		if has_changed {
			Some(promising_node_id.clone())
		} else {
			let root = self.tree.root();
			if root.children().count() == 0 {
				Some(root_id.clone())
			} else {
				None
			}
		}
	}

	/// Expands a leaf node by creating its children, representing all possible moves from that state.
	fn expand_node(&mut self, node_id: NodeId) -> (Vec<NodeId>, NodeId) {
		let node = self.tree.get(node_id).unwrap();
		if node.children().count() != 0 {
			panic!("BUG: expanding already expanded node");
		}
		if node.value().outcome != GameOutcome::OnGoing {
			return (vec![], node_id.clone());
		}

		let children_height = node.value().height + 1;
		let all_possible_moves = self.get_available_moves(node_id);
		let mut new_mcts_nodes = Vec::with_capacity(all_possible_moves.len());

		for possible_move in all_possible_moves {
			let mut board_clone = node.value().board.clone();
			let new = T::apply(&mut board_clone, possible_move);
			if let Some(n) = new {
				board_clone = n;
			}
			let new_node_id = self.random.next();
			let mut mcts_node = MctsNode::new(new_node_id, board_clone);
			mcts_node.prev_move = Some(possible_move);
			mcts_node.height = children_height;
			new_mcts_nodes.push(mcts_node);
		}

		let mut new_node_ids = Vec::with_capacity(new_mcts_nodes.len());
		let mut node = self.tree.get_mut(node_id).unwrap();
		for mcts_node in new_mcts_nodes {
			let child = node.append(mcts_node);
			new_node_ids.push(child.id());
		}
		let node_ref = self.tree.get(node_id).unwrap();
		let children: Vec<_> = node_ref.children().collect();
		if children.is_empty() {
			return (new_node_ids, node_id);
		}
		let selected_child_index = self.random.next_range(0, children.len() as i32) as usize;
		let selected_child = children[selected_child_index].id();
		(new_node_ids, selected_child)
	}

	/// Simulates a random playout from a given node until the game ends.
	fn simulate(&mut self, node_id: NodeId) -> GameOutcome {
		let node = self.tree.get(node_id).unwrap();
		let mut board = node.value().board.clone();
		let mut outcome = GameOutcome::from(
			T::get_winner(&board)
		);
		let mut hashes = self.get_branch_hashes(node_id);

		while outcome == GameOutcome::OnGoing {
			let mut all_possible_moves = vec![];
			T::generate_moves(&board, &mut all_possible_moves);

			while !all_possible_moves.is_empty() {
				let random_move_index =
					self.random.next_range(0, all_possible_moves.len() as i32) as usize;
				let random_move = all_possible_moves.get(random_move_index).unwrap();
				let mut new_board = board.clone();
				let new = T::apply(&mut new_board, *random_move);
				if let Some(n) = new {
					new_board = n;
				}
				let new_board_hash = T::get_hash(&new_board);
				if hashes.contains(&new_board_hash) {
					all_possible_moves.swap_remove(random_move_index);
					continue;
				} else {
					hashes.insert(new_board_hash);
					board = new_board;
					break;
				}
			}

			if all_possible_moves.is_empty() {
				//TODO: it should not happen...
				return GameOutcome::Player(Player(u8::MAX))
			}

			outcome = GameOutcome::from(
				T::get_winner(&board),
			);
		}
		outcome
	}

	/// Propagates the result of a simulation back up the tree, updating node statistics.
	fn backpropagate(&mut self, node_id: NodeId, outcome: GameOutcome) -> Vec<NodeId> {
		let mut branch = vec![node_id.clone()];

		loop {
			let temp_node = self.tree.get(*branch.last().unwrap()).unwrap();
			match temp_node.parent() {
				None => break,
				Some(parent) => branch.push(parent.id()),
			}
		}
		let root_player = self.get_root().value().current_player;
		let is_win = outcome.is_win_for(root_player);
		let is_draw = outcome.is_draw();

		for node_id in &branch {
			let bound = self.get_bound(*node_id);
			let is_fully_calculated = self.is_fully_calculated(*node_id, bound);
			let mut temp_node = self.tree.get_mut(*node_id).unwrap();
			let mcts_node = temp_node.value();
			mcts_node.visits += 1;
			if is_win {
				mcts_node.wins += 1;
			}

			if is_draw {
				mcts_node.draws += 1;
			}

			if is_fully_calculated {
				mcts_node.is_fully_calculated = true;
			}

			if bound != Bound::None {
				mcts_node.bound = bound;
			}
		}

		branch
	}

	/// Determines the bound of a node for alpha-beta pruning.
	fn get_bound(&self, node_id: NodeId) -> Bound {
		if !self.use_alpha_beta_pruning {
			return Bound::None;
		}

		let node = self.tree.get(node_id).unwrap();
		let mcts_node = node.value();
		if mcts_node.bound != Bound::None {
			return mcts_node.bound;
		}
		let root_player = self.get_root().value().current_player;

		if mcts_node.outcome.is_win_for(root_player) {
			return Bound::DefoWin;
		}

		if mcts_node.outcome.is_lose_for(root_player) {
			return Bound::DefoLose;
		}

		if node.children().count() == 0 {
			return Bound::None;
		}

		if node.children().all(|x| x.value().bound == Bound::DefoLose) {
			return Bound::DefoLose;
		}

		if node.children().any(|x| x.value().bound == Bound::DefoWin) {
			return Bound::DefoWin;
		}

		Bound::None
	}

	/// Checks if a node can be considered fully calculated, meaning its outcome is certain.
	fn is_fully_calculated(&self, node_id: NodeId, bound: Bound) -> bool {
		if bound != Bound::None {
			return true;
		}

		let node = self.tree.get(node_id).unwrap();
		if node.value().outcome != GameOutcome::OnGoing {
			return true;
		}

		if node.children().count() == 0 {
			return false;
		}

		let all_children_calculated = node.children().all(|x| x.value().is_fully_calculated);

		all_children_calculated
	}

	/// Calculates the UCB1 (Upper Confidence Bound 1) value for a node.
	fn ucb_value(total_visits: i32, node_wins: i32, node_visit: i32) -> f64 {
		const EXPLORATION_PARAMETER: f64 = std::f64::consts::SQRT_2;

		if node_visit == 0 {
			i32::MAX.into()
		} else {
			((node_wins as f64) / (node_visit as f64))
				+ EXPLORATION_PARAMETER
					* f64::sqrt(f64::ln(total_visits as f64) / (node_visit as f64))
		}
	}

	/// Retrieves the hashes of all nodes in the branch from the given node to the root.
	fn get_branch_hashes(&self, node_id: NodeId) -> HashSet<u64> {
		let mut current_node = self.tree.get(node_id).unwrap();
		let mut branch_hashes = HashSet::with_capacity(current_node.value().height + 1);
		loop {
			branch_hashes.insert(current_node.value().board_hash);
			match current_node.parent() {
				None => break,
				Some(parent) => current_node = parent,
			}
		}
		branch_hashes
	}

	/// Determines the available moves from a given node, avoiding cycles in the tree.
	fn get_available_moves(&self, node_id: NodeId) -> Vec<T::M> {
		let node = self.tree.get(node_id).unwrap();
		let hashes = self.get_branch_hashes(node_id);

		let mut available_moves = vec![];
		T::generate_moves(&node.value().board, &mut available_moves);
		let mut filtered_moves = Vec::with_capacity(available_moves.len());
		for available_move in available_moves {
			let mut board_clone = node.value().board.clone();
			let new = T::apply(&mut board_clone, available_move);
			if let Some(n) = new {
				board_clone = n;
			}
			let hash = T::get_hash(&board_clone);
			if !hashes.contains(&hash) {
				filtered_moves.push(available_move);
			}
		}
		filtered_moves
	}
}

impl<T: Game> MonteCarloTreeSearch<T, StandardRandomGenerator>
where
	T::S: Clone,
{
	pub fn from_board(board: T::S) -> Self {
		MonteCarloTreeSearchBuilder::new(board).build()
	}
}

/// Represents the four main stages of the MCTS algorithm.
///
/// This enum is used to manage the state of the search process.
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Clone)]
pub enum MctsAction {
	/// **Selection**: Start from the root `R` and select successive child nodes until a leaf node `L` is reached.
	Selection {
		/// The root of the current selection phase.
		R: NodeId,
		/// The path of nodes visited during the last backpropagation phase.
		RP: Vec<NodeId>,
	},
	/// **Expansion**: Create one or more child nodes from the selected leaf node `L`.
	Expansion {
		/// The leaf node to be expanded.
		L: NodeId,
	},
	/// **Simulation**: Run a random playout from a newly created child node `C`.
	Simulation {
		/// The child node from which the simulation will start.
		C: NodeId,
		/// All children created during the expansion phase.
		AC: Vec<NodeId>,
	},
	/// **Backpropagation**: Update the statistics of the nodes on the path from `C` to the root `R`.
	Backpropagation {
		/// The child node from which the simulation was run.
		C: NodeId,
		/// The result of the simulation.
		result: GameOutcome,
	},
	/// Represents a state where the entire tree has been explored and the outcome is certain.
	EverythingIsCalculated,
}

impl MctsAction {
	/// Returns the name of the current MCTS action as a string.
	pub fn get_name(&self) -> String {
		match self {
			MctsAction::Selection { R: _, RP: _ } => "Selection".to_string(),
			MctsAction::Expansion { L: _ } => "Expansion".to_string(),
			MctsAction::Simulation { C: _, AC: _ } => "Simulation".to_string(),
			MctsAction::Backpropagation { C: _, result: _ } => "Backpropagation".to_string(),
			MctsAction::EverythingIsCalculated => "EverythingIsCalculated".to_string(),
		}
	}
}

pub struct MctsTreeNode<'a, T: Game>(pub NodeRef<'a, MctsNode<T>>);

impl<'a, T: Game> Deref for MctsTreeNode<'a, T> {
	type Target = NodeRef<'a, MctsNode<T>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'a, T: Game> DerefMut for MctsTreeNode<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<'a, T: Game> Into<NodeRef<'a, MctsNode<T>>> for MctsTreeNode<'a, T> {
	fn into(self) -> NodeRef<'a, MctsNode<T>> {
		self.0
	}
}

impl<'a, T: Game> From<NodeRef<'a, MctsNode<T>>> for MctsTreeNode<'a, T> {
	fn from(node: NodeRef<'a, MctsNode<T>>) -> Self {
		Self(node)
	}
}

impl<'a, T: Game> MctsTreeNode<'a, T> {
	/// Returns the child of the given node that is considered the most promising, based on win rate.
	pub fn get_best_child(&self) -> Option<MctsTreeNode<'a, T>> {
		let mut best_child = None;
		let mut best_child_value = f32::MIN;

		// get the best child amount with DefoWin bound
		for child in self
			.children()
			.filter(|x| x.value().bound == Bound::DefoWin)
		{
			let child_value = child.value().wins_rate();
			if child_value > best_child_value {
				best_child = Some(child);
				best_child_value = child_value;
			}
		}

		if best_child.is_some() {
			return best_child.map(|x| x.into());
		}

		// get the best child overall
		for child in self.children() {
			let child_value = child.value().wins_rate();
			if child_value > best_child_value {
				best_child = Some(child);
				best_child_value = child_value;
			}
		}

		best_child.map(|x| x.into())
	}
}
impl<G: Game, K: RandomGenerator> Strategy<G> for MonteCarloTreeSearchBuilder<G, K>
where
	G::S: Clone,
{
	fn choose_move(&mut self, state: &<G as Game>::S) -> Option<<G as Game>::M> {
		let mut searcher = MonteCarloTreeSearch::<G, K>::new(
			state.clone(),
			self.random_generator.clone(),
			self.use_alpha_beta_pruning,
		);
		searcher.iterate_n_times(20000);
		let m = searcher.get_root().get_best_child();
		if let Some(m) = m {
			return m.value().prev_move;
		}
		None
	}
}
