use std::{
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::ai::move_search::sync_util::timeout_signal;
use crate::{
	GameOutcome, Player, StrategyWithOptions,
	ai::move_search::{
		Evaluation, Game, RolloutPolicy, SearchStopSignal, Strategy, gametree::{GameTree, Node, ProofOutcome}, simulate
	},
};
#[derive(Clone, Default)]
pub struct VisitStats {
	pub(crate) visits: u32,
	pub(crate) wins: i32,
	pub(crate) draws: u32,
}
impl<G: Game> Node<G, VisitStats> {
	pub fn winrate(&self) -> f32 {
		if self.data.visits == 0 {
			0.0
		} else {
			self.data.wins as f32 / self.data.visits as f32
		}
	}
	pub fn drawrate(&self) -> f32 {
		if self.data.visits == 0 {
			0.0
		} else {
			self.data.draws as f32 / self.data.visits as f32
		}
	}
	pub fn lossrate(&self) -> f32 {
		if self.data.visits == 0 {
			0.0
		} else {
			(self.data.visits as f32 - self.data.draws as f32 - self.data.wins as f32)
				/ self.data.visits as f32
		}
	}
}
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

pub struct MCTS<G: Game, Policy: RolloutPolicy<G>>
where
	G::S: Clone,
{
	tree: Option<GameTree<G, VisitStats>>,
	pub opts: MCTSOptions,
	stop_signal: Arc<AtomicBool>,
	rollout_policy: Policy,
}
impl<G: Game, Policy: RolloutPolicy<G>> Default for MCTS<G, Policy>
where
	G::S: Clone,
{
	fn default() -> Self {
		Self {
			tree: Default::default(),
			opts: Default::default(),
			stop_signal: Arc::new(AtomicBool::new(false)),
			rollout_policy: Policy::default(),
		}
	}
}
impl<G: Game> Node<G, VisitStats> {
	pub fn uct_score(&self, exploration_factor: f32, parent_visits: u32, is_min: bool) -> f32 {
		if self.data.visits == 0 {
			return i32::MAX as f32;
		}
		let exploitation = if is_min {
			-(self.data.wins as f32) / self.data.visits as f32
		} else {
			self.data.wins as f32 / self.data.visits as f32
		};
		let exploration = ((parent_visits as f32).ln() / self.data.visits as f32).sqrt();
		exploitation + exploration_factor * exploration
	}
	pub fn score(&self) -> Evaluation {
		if self.outcome.is_ended() {
			GameOutcome::from(self.outcome).evaluate_for(self.player_to_move)
		} else {
			let winrate = self.winrate();
			let drawrate = self.drawrate();
			(8000.0 * (winrate + 0.5 * drawrate - 0.5)) as Evaluation
		}
	}
}
impl<G: Game> GameTree<G, VisitStats>
where
	G::S: Clone,
{
	fn select(&self, exploration_factor: f32, mut node_id: usize) -> (usize, bool) {
		let mut already_proved = false;
		let root_player = self.get_root().player_to_move;
		while self.nodes[node_id].untried_moves.is_empty()
			&& !self.nodes[node_id].children.is_empty()
		{
			let parent_visits = self.get_node(node_id).unwrap().data.visits;
			let node = self.get_node(node_id).unwrap();
			let selected = if G::is_random_move(&node.state) {
				let mut sum_proba = 0.0;
				let rand = fastrand::f32();
				let mut ch_node = None;
				for e in node.children.iter() {
					let mv = e.mv;
					sum_proba += G::get_probability(&node.state, mv);
					if sum_proba > rand {
						ch_node = Some(e);
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
					.max_by(|&a, &b| {
						let ca = self.get_node(a.child).unwrap();
						let cb = self.get_node(b.child).unwrap();
						let player = self.get_node(a.child).unwrap().player_to_move;
						let is_root_player = player == root_player;

						// minimize if next move is root player
						let ua = ca.uct_score(exploration_factor, parent_visits, is_root_player);
						let ub = cb.uct_score(exploration_factor, parent_visits, is_root_player);
						ua.partial_cmp(&ub).unwrap()
					})
			};

			if let Some(id) = selected {
				node_id = id.child;
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
		if let Some(m) = self.nodes[node_id].untried_moves.pop() {
			//let child_id = self.create_child_node(node_id, m);
			let (child_id, _res) = self.expand_single_child(node_id, m);
			node_id = child_id;
		}
		node_id
	}
	pub(crate) fn backpropagate(
		&mut self,
		root_player: Player,
		node_id: usize,
		result: GameOutcome,
		//use_min_max: bool,
	) {
		let mut current = Some(node_id);
		while let Some(id) = current {
			let node = &mut self.nodes[id];
			node.data.visits += 1;

			if result.is_win_for(root_player) {
				node.data.wins += 1;
			} else if result.is_draw() {
				node.data.draws += 1;
			} else if result.is_lose_for(root_player) {
				node.data.wins -= 1;
			}

			/*if use_min_max {
				self.get_outcome(id);
			}*/
			if id == self.root_id {
				break;
			}
			current = self.nodes[id].parent;
		}
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
			.filter(|c| !self.get_node(c.child).unwrap().outcome.is_lose_for(p))
			.max_by(|&a, &b| {
				let na = self.get_node(a.child).unwrap();
				let nb = self.get_node(b.child).unwrap();
				na.data.visits.partial_cmp(&nb.data.visits).unwrap()
			});
		if let Some(best_child) = best_child {
			let best_move = best_child.mv;
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
impl<G: Game, Policy: RolloutPolicy<G>> MCTS<G, Policy>
where
	G::S: Clone,
{
	pub fn mcts(&mut self, root_state: &G::S, iterations: u32) -> Option<G::M>
	where
		G::S: Clone,
	{
		let root_state_hash = G::get_hash(root_state);
		let mut tree = self.tree.take().unwrap_or_default();
		if let Some(si) = tree.state_to_node.get(&root_state_hash) {
			tree.set_root_id(*si);
			println!(
				"Reusing tree: {}/{} v.",
				tree.get_root().winrate(),
				tree.get_root().data.visits
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
				//TODO: check clone
				state: root_state.clone(),
				parent: None,
				children: vec![],
				data: VisitStats::default(),
				untried_moves: moves,
				player_to_move: G::get_current_player(root_state),
				outcome: ProofOutcome::Unproved,
				//depth_to_end: u16::MAX,
			});
			tree.state_to_node.insert(root_state_hash, 0);
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
		let root_id = tree.root_id;
		while i < iterations {
			self.iterate(
				&mut tree,
				root_player,
				root_id,
				self.opts.exploration_factor,
				//self.opts.use_min_max,
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
	pub fn iterate(
		&mut self,
		tree: &mut GameTree<G, VisitStats>,
		root_player: Player,
		node_id: usize,
		exploration_factor: f32,
		//use_min_max: bool,
	) where
		G::S: Clone,
	{
		//let tree = self.tree.as_mut().unwrap();
		let (id, already_proved) = tree.select(exploration_factor, node_id);
		if already_proved {
			println!("ahhhhh!!!");
			return;
		}
		let new_id = tree.expand(id);
		let result = simulate(tree.get_node_state(new_id).unwrap(), self.rollout_policy);
		//println!("{:?}", result);
		//self.backpropagate(root_player, new_id, result, use_min_max);
		tree.backpropagate(root_player, new_id, result);
	}
	pub fn get_tree(&self) -> Option<&GameTree<G, VisitStats>> {
		self.tree.as_ref()
	}
}

impl<G: Game, Policy: RolloutPolicy<G>> Strategy<G> for MCTS<G, Policy>
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
				GameOutcome::from(root.outcome).evaluate_for(root.player_to_move)
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
impl<G: Game, Policy: RolloutPolicy<G>> StrategyWithOptions<G> for MCTS<G, Policy>
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
