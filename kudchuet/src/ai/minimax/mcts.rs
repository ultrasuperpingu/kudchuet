use core::f64;

use crate::{
	GameOutcome, Player, ai::minimax::{
		Game, Strategy,
		gametree::{GameTree, Node, StateInfo},
		util::AppliedMove,
	}
};
pub struct MCTSOptions {
	pub max_nb_iteration: usize,
	pub exploration_factor: f64,
}
impl Default for MCTSOptions {
	fn default() -> Self {
		Self {
			max_nb_iteration: 20000,
			exploration_factor: f64::consts::SQRT_2,
		}
	}
}
impl MCTSOptions {
	pub fn with_max_nb_iteration(&mut self, value: usize) -> &mut Self {
		self.max_nb_iteration = value;
		self
	}
	pub fn with_exploration_factor(&mut self, value: f64) -> &mut Self {
		self.exploration_factor = value;
		self
	}
}

pub struct MCTS<G: Game> {
	tree: Option<GameTree<G>>,
	//root_id: usize,
	opts: MCTSOptions,
}
impl<G: Game> Default for MCTS<G> {
	fn default() -> Self {
		Self {
			tree: Default::default(),
			//root_id: 0,
			opts: Default::default(),
		}
	}
}

impl<G: Game> GameTree<G> {

	pub(crate) fn expand(&mut self, mut node_id: usize) -> usize
	where
		G::S: Clone,
	{
		if !self.nodes[node_id].untried_moves.is_empty() {
			let m = self.nodes[node_id].untried_moves.pop().unwrap();
			let mut state = self.get_state(self.nodes[node_id].state).unwrap().clone();
			let new_state = AppliedMove::<G>::new(&mut state, m).clone();
			let new_state_hash = G::get_hash(&new_state);
			let child_id = self.nodes.len();
			self.states.insert(new_state_hash, StateInfo { state: new_state.clone(), expanded_node: child_id });
			let mut moves = vec![];
			let outcome = G::generate_moves(&new_state, &mut moves);
			self.nodes.push(Node {
				state: new_state_hash,
				parent: Some(node_id),
				children: vec![],
				visits: 0.0,
				wins: 0.0,
				draws: 0.0,
				untried_moves: moves,
				player_to_move: G::get_current_player(&new_state),
				outcome,
				incoming_move: Some(m),
			});

			self.nodes[node_id].children.push(child_id);
			node_id = child_id;
		}
		node_id
	}


	pub(crate) fn backpropagate(
		&mut self,
		root_player: Player,
		node_id: usize,
		result: GameOutcome,
	) {
		let mut current = Some(node_id);
		while let Some(id) = current {
			self.nodes[id].visits += 1.0;

			if result.is_win_for(root_player) {
				self.nodes[id].wins += 1.0;
			} else if result.is_draw() {
				self.nodes[id].wins += 0.5;
			}
			//self.get_outcome(id, root_player);
			self.get_outcome(id);

			current = self.nodes[id].parent;
		}
	}

	pub fn find_best_move(&self) -> <G as Game>::M {
		let mut filtered: Vec<_> = self.nodes[self.root_id]
			.children
			.iter()
			.filter(|id| {
				self.nodes[**id]
					.outcome
					.is_win_for(self.nodes[self.root_id].player_to_move)
			})
			.collect();
		if filtered.is_empty() {
			filtered = self.nodes[self.root_id]
				.children
				.iter()
				.filter(|id| self.nodes[**id].outcome.is_draw())
				.collect();
		}
		if filtered.is_empty() {
			filtered = self.nodes[self.root_id].children.iter().collect();
		}

		let best_child = filtered
			.iter()
			.max_by(|&&a, &&b| {
				self.nodes[*a]
					.visits
					.partial_cmp(&self.nodes[*b].visits)
					.unwrap()
			})
			.unwrap();

		let best_move = self.nodes[**best_child]
			.incoming_move
			.clone()
			.expect("root child must have a move");
		println!(
			"to_move: {}\n{}",
			self.nodes[self.root_id].player_to_move, self
		);
		best_move
	}
	
}
impl<G: Game> MCTS<G> {
	pub fn mcts(&mut self, root_state: &G::S, iterations: usize) -> G::M
	where
		G::S: Clone,
	{
		/*let mut tree = self.tree.take();
		if let Some(tree) = tree {
			if let Some(id) = Self::find_node_by_state(&tree, root_state) {
				self.root_id = id;
			} else {
				tree = MCTSTree::<G>::default();
				self.root_id = 0;
			}
		}*/
		let mut tree = GameTree::<G>::default();
		//let mut tree = tree.unwrap();

		let mut moves = vec![];
		G::generate_moves(&root_state, &mut moves);
		let root_state_hash = G::get_hash(root_state);
		tree.root_id = tree.nodes.len();
		tree.nodes.push(Node {
			state: root_state_hash,
			parent: None,
			children: vec![],
			visits: 0.0,
			wins: 0.0,
			draws: 0.0,
			untried_moves: moves,
			player_to_move: G::get_current_player(&root_state),
			outcome: GameOutcome::OnGoing,
			incoming_move: None,
		});
		let root_player = tree.get_root().player_to_move;

		for _ in 0..iterations {
			let (mut node_id, already_computed) = self.select(&tree);
			if already_computed {
				break;
			}

			node_id = tree.expand(node_id);

			let sim_state_hash = tree.nodes[node_id].state;
			let sim_state = tree.states.get(&sim_state_hash).unwrap().state.clone();
			let result = simulate::<G>(sim_state);

			tree.backpropagate(root_player, node_id, result);
		}

		let res = tree.find_best_move();
		self.tree = Some(tree);
		res
	}

	fn select(&mut self, tree: &GameTree<G>) -> (usize, bool) {
		let mut node_id = tree.root_id;
		let mut already_computed = false;
		while tree.nodes[node_id].untried_moves.is_empty()
			&& !tree.nodes[node_id].children.is_empty()
		{
			let parent_visits = tree.nodes[node_id].visits;

			let selected = tree.nodes[node_id]
				.children
				.iter()
				.filter(|id| !tree.nodes[**id].outcome.is_ended())
				.max_by(|&&a, &&b| {
					let ua = self.ucb1(parent_visits, tree.nodes[a].wins, tree.nodes[a].visits);
					let ub = self.ucb1(parent_visits, tree.nodes[b].wins, tree.nodes[b].visits);
					ua.partial_cmp(&ub).unwrap()
				});
			if let Some(id) = selected {
				node_id = *id;
			} else {
				// nothing to select
				already_computed = true;
				break;
			}
		}
		(node_id, already_computed)
	}

	fn ucb1(&self, parent_visits: f64, wins: f64, visits: f64) -> f64 {
		if visits == 0.0 {
			return f64::INFINITY;
		}
		debug_assert!(parent_visits >= 0.0);
		let exploitation = wins / visits;
		let exploration = (parent_visits.ln() / visits).sqrt();
		exploitation + self.opts.exploration_factor * exploration
	}
}

fn simulate<G: Game>(mut sim_state: <G as Game>::S) -> GameOutcome
where
	G::S: Clone,
{
	let mut result = G::get_outcome(&sim_state);

	while !result.is_ended() {
		let mut moves = vec![];
		result = G::generate_moves(&sim_state, &mut moves);
		if result == GameOutcome::OnGoing {
			let m = fastrand::choice(moves);
			sim_state = AppliedMove::<G>::new(&mut sim_state.clone(), m.unwrap()).clone();
		}
	}
	result
}

impl<G: Game> Strategy<G> for MCTS<G>
where
	G::S: Clone,
{
	fn choose_move(&mut self, state: &<G as Game>::S) -> Option<<G as Game>::M> {
		Some(self.mcts(&state, self.opts.max_nb_iteration))
	}
}
