use std::{
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::ai::minimax::sync_util::timeout_signal;
use crate::{
	GameOutcome, Player, StrategyWithOptions,
	ai::minimax::{
		Evaluation, Game, SearchStopSignal, Strategy,
		gametree::{GameTree, Node, StateInfo},
		util::AppliedMove,
	},
};

pub struct MCTSOptions {
	pub max_nb_iteration: u32,
	pub exploration_factor: f32,
	pub use_min_max: bool,
	pub max_time: Duration,
}
impl Default for MCTSOptions {
	fn default() -> Self {
		Self {
			max_nb_iteration: 20000,
			exploration_factor: std::f32::consts::SQRT_2,
			use_min_max: true,
			max_time: Duration::from_secs(5),
		}
	}
}
impl MCTSOptions {
	pub fn with_max_nb_iteration(&mut self, value: u32) -> &mut Self {
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
impl<M> Node<M> {
	pub fn uct_score(&self, exploration_factor: f32, parent_visits: u32, is_min: bool) -> f32 {
		if self.visits == 0 {
			return i32::MAX as f32;
		}
		let exploitation = if is_min {
			-(self.wins as f32) / self.visits as f32
		} else {
			self.wins as f32 / self.visits as f32
		};
		let exploration = ((parent_visits as f32).ln() / self.visits as f32).sqrt();
		exploitation + exploration_factor * exploration
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
			let parent_visits = self.get_node(node_id).unwrap().visits;
			let node = self.get_node_expanded_node(node_id).unwrap();
			let selected = if G::is_random_move(self.get_state(node.state_hash).unwrap()) {
				let mut sum_proba = 0.0;
				let rand = fastrand::f32();
				let mut ch_node = None;
				for child_id in node.children.iter() {
					let child_node = &self.nodes[*child_id];
					let mv = child_node.incoming_move.unwrap();
					sum_proba += G::get_probability(self.get_state(node.state_hash).unwrap(), mv);
					if sum_proba > rand {
						ch_node = Some(child_id);
						break;
					}
				}
				ch_node
			} else {
				node.children
					.iter()
					//.filter(|id| {
					//	!self
					//		.get_node_expanded_node(**id)
					//		.unwrap()
					//		.outcome
					//		.is_ended()
					//})
					.max_by(|&&a, &&b| {
						let ca = self.get_node(a).unwrap();
						let cb = self.get_node(b).unwrap();
						let player = self.get_node(a).unwrap().player_to_move;
						let is_root_player = player == root_player;

						// minimize if next move is root player
						let ua = ca.uct_score(exploration_factor, parent_visits, is_root_player);
						let ub = cb.uct_score(exploration_factor, parent_visits, is_root_player);
						ua.partial_cmp(&ub).unwrap()
					})
			};

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
		let state = self.get_node_state_mut(parent_id).unwrap();
		let new_state = AppliedMove::<G>::applied_clone(state, m);
		let new_state_hash = G::get_hash(&new_state);
		let tt_state = self.states.get(&new_state_hash);
		let child_id = self.nodes.len();
		if let Some(si) = tt_state {
			let expanded_node = &self.nodes[si.expanded_node];
			self.nodes.push(Node {
				state_hash: new_state_hash,
				parent: Some(parent_id),
				children: vec![],
				visits: 0,
				wins: 0,
				draws: 0,
				untried_moves: vec![],
				player_to_move: expanded_node.player_to_move,
				outcome: expanded_node.outcome,
				depth_to_end: expanded_node.depth_to_end,
				incoming_move: Some(m),
			});
		} else {
			self.states.insert(
				new_state_hash,
				StateInfo {
					state: new_state.clone(),
					expanded_node: child_id,
				},
			);
			let mut moves = vec![];
			let outcome = G::generate_and_filter_moves(&new_state, &mut moves);
			self.nodes.push(Node {
				state_hash: new_state_hash,
				parent: Some(parent_id),
				children: vec![],
				visits: 0,
				wins: 0,
				draws: 0,
				untried_moves: moves,
				player_to_move: G::get_current_player(&new_state),
				outcome,
				depth_to_end: if outcome.is_ended() { 0 } else { u16::MAX },
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

			let node = &mut self.nodes[id];
			node.visits += 1;

			if result.is_win_for(root_player) {
				node.wins += 1;
			} else if result.is_draw() {
				node.draws += 1;
			} else if result.is_lose_for(root_player) {
				node.wins -= 1;
			}

			if use_min_max {
				self.get_outcome(cid);
			}
			if id == self.root_id {
				break;
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
			result =
				G::generate_and_filter_moves(self.get_node_state(node_id).unwrap(), &mut moves);
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
			println!("ahhhhh!!!");
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
		let p = self.get_root().player_to_move;
		let best_child = self
			.get_root()
			.children
			.iter()
			.filter(|c| {
				!self
					.get_node_expanded_node(**c)
					.unwrap()
					.outcome
					.is_lose_for(p)
			})
			.max_by(|&a, &b| {
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
			//	self.get_root().player_to_move, self
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
	pub fn mcts(&mut self, root_state: &G::S, iterations: u32) -> Option<G::M>
	where
		G::S: Clone,
	{
		let root_state_hash = G::get_hash(root_state);
		let mut tree = self.tree.take().unwrap_or_default();
		if let Some(si) = tree.states.get(&root_state_hash) {
			tree.set_root_id(si.expanded_node);
			println!(
				"Reusing tree: {}/{} v.",
				tree.get_root().winrate(),
				tree.get_root().visits
			);
			//TODO: clean only when tree is too large??...
			//tree.cleanup();
		} else {
			println!(
				"Tree does not contains the state. size={}",
				tree.nodes.len()
			);
			if !tree.nodes.is_empty() {
				println!("Resetting tree");
				tree = GameTree::default();
			}
			let mut moves = vec![];
			G::generate_and_filter_moves(root_state, &mut moves);
			tree.root_id = tree.nodes.len();
			tree.nodes.push(Node {
				state_hash: root_state_hash,
				parent: None,
				children: vec![],
				visits: 0,
				wins: 0,
				draws: 0,
				untried_moves: moves,
				player_to_move: G::get_current_player(root_state),
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
		}

		self.stop_signal.store(false, Ordering::Relaxed);
		let _cancel_when_dropped = if !self.opts.max_time.is_zero() {
			#[cfg(not(target_arch = "wasm32"))]
			{
				timeout_signal(self.opts.max_time, &self.stop_signal)
			}
			#[cfg(target_arch = "wasm32")]
			{
				Arc::new(())
			}
		} else {
			Arc::new(())
		};

		let root_player = tree.get_root().player_to_move;
		let mut i = 0;
		while i < iterations {
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
			//println!("end of iteration {}", i);
			i += 1;
		}

		let res = tree.find_best_move();
		println!("res: {:?} in {} iterations\n", res, i);
		tree.print(1);
		self.tree = Some(tree);
		res
	}
	pub fn get_tree(&self) -> Option<&GameTree<G>> {
		self.tree.as_ref()
	}
}

impl<G: Game> Strategy<G> for MCTS<G>
where
	G::S: Clone,
{
	fn choose_move(&mut self, state: &<G as Game>::S) -> Option<<G as Game>::M> {
		self.mcts(state, self.opts.max_nb_iteration)
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
	fn set_timeout(&mut self, _timeout: std::time::Duration) {
		self.opts.max_time = _timeout;
	}
	fn set_max_depth(&mut self, depth: u8) {
		// Set some arbitrary function of rollouts.
		self.opts.max_time = Duration::default();
		self.opts.max_nb_iteration = 5u32.saturating_pow(depth as u32);
	}

	fn set_depth_or_timeout(&mut self, depth: u8, max_time: Duration) {
		self.set_max_depth(depth);
		self.opts.max_time = max_time;
	}
}
impl<G: Game> StrategyWithOptions<G> for MCTS<G>
where
	G::S: Clone,
{
	fn get_options(&self) -> std::collections::HashMap<String, crate::ai::uci::UciValue> {
		std::collections::HashMap::new()
	}

	fn set_options(&mut self, _opts: &std::collections::HashMap<String, crate::ai::uci::UciValue>) {
	}
	fn stop_signal(&self) -> SearchStopSignal {
		SearchStopSignal(self.stop_signal.clone())
	}
}
