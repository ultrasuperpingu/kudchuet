use crate::{
	GameOutcome, Player,
	ai::minimax::{
		Game, Strategy,
		gametree::{GameTree, Node, StateInfo},
		util::AppliedMove,
	},
};
pub struct MCTSOptions {
	pub max_nb_iteration: usize,
	pub exploration_factor: f32,
	pub use_min_max: bool,
}
impl Default for MCTSOptions {
	fn default() -> Self {
		Self {
			max_nb_iteration: 20000,
			exploration_factor: std::f32::consts::SQRT_2,
			use_min_max: true,
		}
	}
}
impl MCTSOptions {
	pub fn with_max_nb_iteration(&mut self, value: usize) -> &mut Self {
		self.max_nb_iteration = value;
		self
	}
	pub fn with_exploration_factor(&mut self, value: f32) -> &mut Self {
		self.exploration_factor = value;
		self
	}
}

pub struct MCTS<G: Game> {
	tree: Option<GameTree<G>>,
	pub opts: MCTSOptions,
}
impl<G: Game> Default for MCTS<G> {
	fn default() -> Self {
		Self {
			tree: Default::default(),
			opts: Default::default(),
		}
	}
}

impl<G: Game> GameTree<G> {
	fn select(&self, exploration_factor: f32, mut node_id: usize) -> (usize, bool) {
		let mut already_computed = false;
		let root_player = self.get_root().player_to_move;
		while self.nodes[node_id].untried_moves.is_empty()
			&& !self.nodes[node_id].children.is_empty()
		{
			let parent_visits = self.get_node_expanded_node(node_id).unwrap().visits;

			let selected = self.nodes[node_id]
				.children
				.iter()
				.filter(|id| {
					!self
						.get_node_expanded_node(**id)
						.unwrap()
						.outcome
						.is_ended()
				})
				.max_by(|&&a, &&b| {
					let ca = self.get_node_expanded_node(a).unwrap();
					let cb = self.get_node_expanded_node(b).unwrap();
					let player = self.get_node(a).unwrap().player_to_move;
					let is_root_player = player == root_player;

					// minize if next move is root player
					let ua = self.uct_score(
						exploration_factor,
						parent_visits,
						ca.wins,
						ca.visits,
						is_root_player,
					);
					let ub = self.uct_score(
						exploration_factor,
						parent_visits,
						cb.wins,
						cb.visits,
						is_root_player,
					);
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

	fn uct_score(
		&self,
		exploration_factor: f32,
		parent_visits: f32,
		wins: f32,
		visits: f32,
		is_min: bool,
	) -> f32 {
		if visits == 0.0 {
			return f32::INFINITY;
		}
		debug_assert!(parent_visits >= 0.0);
		let exploitation = if is_min {
			-wins / visits
		} else {
			wins / visits
		};
		let exploration = (parent_visits.ln() / visits).sqrt();
		exploitation + exploration_factor * exploration
	}
	pub(crate) fn expand(&mut self, mut node_id: usize) -> usize
	where
		G::S: Clone,
	{
		if !self.nodes[node_id].untried_moves.is_empty() {
			let m = self.nodes[node_id].untried_moves.pop().unwrap();
			let child_id = self.create_child_node(node_id, m);
			node_id = child_id;
		}
		node_id
	}

	fn create_child_node(&mut self, parent_id: usize, m: <G as Game>::M) -> usize
	where
		G::S: Clone,
	{
		let state = self.get_node_state(parent_id).unwrap();
		let new_state = AppliedMove::<G>::applied_clone(state, m);
		let new_state_hash = G::get_hash(&new_state);
		let tt_state = self.states.get(&new_state_hash);
		let child_id = self.nodes.len();
		if tt_state.is_none() {
			self.states.insert(
				new_state_hash,
				StateInfo {
					state: new_state.clone(),
					expanded_node: child_id,
				},
			);
			let mut moves = vec![];
			let outcome = G::generate_moves(&new_state, &mut moves);
			self.nodes.push(Node {
				state: new_state_hash,
				parent: Some(parent_id),
				children: vec![],
				visits: 0.0,
				wins: 0.0,
				draws: 0.0,
				untried_moves: moves,
				player_to_move: G::get_current_player(&new_state),
				outcome,
				incoming_move: Some(m),
			});
		} else {
			let si = tt_state.unwrap();
			let expanded_node = &self.nodes[si.expanded_node];
			self.nodes.push(Node {
				state: new_state_hash,
				parent: Some(parent_id),
				children: vec![],
				visits: 0.0,
				wins: 0.0,
				draws: 0.0,
				untried_moves: vec![],
				player_to_move: expanded_node.player_to_move,
				outcome: expanded_node.outcome,
				incoming_move: Some(m),
			});
		}

		self.nodes[parent_id].children.push(child_id);
		child_id
	}

	pub(crate) fn backpropagate(
		&mut self,
		root_player: Player,
		node_id: usize,
		result: GameOutcome,
		use_min_max: bool,
	) {
		let mut current = Some(node_id);
		while let Some(id) = current {
			let cid = self
				.get_node_state_info(id)
				.map(|n| n.expanded_node)
				.unwrap_or(id);

			let node = &mut self.nodes[cid];
			node.visits += 1.0;

			if result.is_win_for(root_player) {
				node.wins += 1.0;
			} else if result.is_draw() {
				node.draws += 1.0;
			} else if result.is_lose_for(root_player) {
				node.wins -= 1.0;
			}

			if use_min_max {
				self.get_outcome(cid);
			}

			current = self.nodes[id].parent;
		}
	}

	fn simulate_expand(&mut self, mut node_id: usize) -> GameOutcome
	where
		G::S: Clone,
	{
		//println!("{:?}", self.get_node_state(node_id).unwrap());
		let mut result = G::get_outcome(self.get_node_state(node_id).unwrap());
		while !result.is_ended() {
			let mut moves = vec![];
			result = G::generate_moves(&self.get_node_state(node_id).unwrap(), &mut moves);
			if result == GameOutcome::OnGoing {
				let m = fastrand::choice(moves).unwrap();
				node_id = self.create_child_node(node_id, m);
				//println!("{:?}", self.get_node_state(node_id).unwrap());
			}
		}
		result
	}
	pub fn iterate(
		&mut self,
		root_player: Player,
		node_id: usize,
		exploration_factor: f32,
		use_min_max: bool,
	) where
		G::S: Clone,
	{
		let (id, _already_computed) = self.select(exploration_factor, node_id);
		let new_id = self.expand(id);
		let result = self.simulate(new_id);
		//println!("{:?}", result);
		self.backpropagate(root_player, new_id, result, use_min_max);
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
		tree.states.insert(
			root_state_hash,
			StateInfo {
				state: root_state.clone(),
				expanded_node: 0,
			},
		);
		let root_player = tree.get_root().player_to_move;

		for _ in 0..iterations {
			/*let (mut node_id, already_computed) = tree.select(self.opts.exploration_factor, tree.root_id);
			if already_computed {
				break;
			}

			node_id = tree.expand(node_id);

			//let sim_state_hash = tree.nodes[node_id].state;
			//let sim_state = tree.states.get(&sim_state_hash).unwrap().state.clone();
			//let result = simulate::<G>(sim_state);
			let result = tree.simulate(node_id);

			tree.backpropagate(root_player, node_id, result, self.opts.use_min_max);*/
			tree.iterate(
				root_player,
				tree.root_id,
				self.opts.exploration_factor,
				self.opts.use_min_max,
			);
		}

		let res = tree.find_best_move();
		self.tree = Some(tree);
		res
	}
}

impl<G: Game> Strategy<G> for MCTS<G>
where
	G::S: Clone,
{
	fn choose_move(&mut self, state: &<G as Game>::S) -> Option<<G as Game>::M> {
		Some(self.mcts(&state, self.opts.max_nb_iteration))
	}
}
