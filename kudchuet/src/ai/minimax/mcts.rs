use std::sync::{
	Arc,
	atomic::{AtomicBool, Ordering},
};

use crate::{
	GameOutcome, Player, StrategyWithOptions,
	ai::{
		AIOptions,
		minimax::{
			Evaluation, Game, SearchStopSignal, Strategy,
			gametree::{GameTree, Node, StateInfo},
			util::AppliedMove,
		},
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

pub struct MCTS<G: Game>
where
	G::S: Clone,
{
	tree: Option<GameTree<G>>,
	pub opts: MCTSOptions,
	stop_signal: Arc<AtomicBool>,
}
impl<G: Game> Default for MCTS<G>
where
	G::S: Clone,
{
	fn default() -> Self {
		Self {
			tree: Default::default(),
			opts: Default::default(),
			stop_signal: Arc::new(AtomicBool::new(false)),
		}
	}
}

impl<G: Game> GameTree<G>
where
	G::S: Clone,
{
	fn select(&self, exploration_factor: f32, mut node_id: usize) -> (usize, bool) {
		let mut already_proved = false;
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
				already_proved = true;
				break;
			}
		}
		(node_id, already_proved)
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
				depth_to_end: if outcome.is_ended() { 0 } else { u16::MAX },
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
				depth_to_end: expanded_node.depth_to_end,
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

	fn _simulate_expand(&mut self, mut node_id: usize) -> GameOutcome
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
		let (id, already_proved) = self.select(exploration_factor, node_id);
		if already_proved {
			return;
		}
		let new_id = self.expand(id);
		let result = self.simulate(new_id);
		//println!("{:?}", result);
		self.backpropagate(root_player, new_id, result, use_min_max);
	}

	pub fn find_best_move(&self) -> Option<<G as Game>::M> {
		if let Some(best) = self.find_best_proved_move() {
			return Some(best);
		}
		let best_child = self.nodes[self.root_id].children.iter().max_by(|&a, &b| {
			let na = self.get_node_expanded_node(*a).unwrap();
			let nb = self.get_node_expanded_node(*b).unwrap();
			na.visits.partial_cmp(&nb.visits).unwrap()
		});
		if let Some(best_child) = best_child {
			let best_move = self.nodes[*best_child]
				.incoming_move
				.clone()
				.expect("root child must have a move");
			//println!(
			//	"to_move: {}\n{}",
			//	self.nodes[self.root_id].player_to_move, self
			//);
			Some(best_move)
		} else {
			None
		}
	}
}
impl<G: Game> MCTS<G>
where
	G::S: Clone,
{
	pub fn mcts(&mut self, root_state: &G::S, iterations: usize) -> Option<G::M>
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
		self.stop_signal.store(false, Ordering::Relaxed);
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
			depth_to_end: u16::MAX,
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

		for _i in 0..iterations {
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
			if self.stop_signal.load(Ordering::Relaxed) {
				println!("stop_signal received");
				break;
			}
			//println!("end of iteration {}", _i);
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
		self.mcts(&state, self.opts.max_nb_iteration)
	}
	fn root_value(&self) -> Evaluation {
		if let Some(t) = &self.tree {
			let root = t.get_root();

			if root.outcome.is_ended() {
				root.outcome.evaluate(root.player_to_move)
			} else {
				let winrate = root.winrate();
				let drawrate = root.drawrate();
				let lossrate = 1.0 - winrate - drawrate;

				let score = (winrate * 8000.0) - (lossrate * 8000.0);
				score as Evaluation
			}
		} else {
			0
		}
	}
}
impl<G: Game> StrategyWithOptions<G, AIOptions> for MCTS<G>
where
	G::S: Clone,
{
	fn get_options(&self) -> AIOptions {
		AIOptions::default()
	}

	fn reset_with_options(&mut self, _opts: AIOptions) {}

	fn stop_signal(&self) -> SearchStopSignal {
		SearchStopSignal(self.stop_signal.clone())
	}
}
