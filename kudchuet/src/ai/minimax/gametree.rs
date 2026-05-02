use crate::{
	GameOutcome, Player, StrategyWithOptions,
	ai::{
		AIOptions,
		minimax::{Evaluation, Game, Strategy, util::AppliedMove},
	},
	utils::NHHashMap,
};

use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Node<M> {
	pub(crate) state: u64,
	pub(crate) parent: Option<usize>,
	pub(crate) children: Vec<usize>,

	pub(crate) visits: f32,
	pub(crate) wins: f32,
	pub(crate) draws: f32,

	pub(crate) untried_moves: Vec<M>,
	pub(crate) player_to_move: Player,
	pub(crate) outcome: GameOutcome,
	pub(crate) incoming_move: Option<M>,
}
#[derive(Debug, Clone)]
pub struct StateInfo<S>
//where S: Send
{
	pub(crate) state: S,
	pub(crate) expanded_node: usize,
}
impl<M> Node<M> {
	pub fn winrate(&self) -> f32 {
		if self.visits == 0.0 {
			0.0
		} else {
			self.wins / self.visits
		}
	}
	pub fn drawrate(&self) -> f32 {
		if self.visits == 0.0 {
			0.0
		} else {
			self.draws / self.visits
		}
	}
	pub fn lossrate(&self) -> f32 {
		if self.visits == 0.0 {
			0.0
		} else {
			(self.visits - self.draws - self.wins) / self.visits
		}
	}
	pub fn score(&self) -> Evaluation {
		if self.outcome.is_ended() {
			self.outcome.evaluate(self.player_to_move)
		} else {
			let winrate = self.winrate();
			let drawrate = self.drawrate();
			(8000.0 * (winrate + 0.5 * drawrate - 0.5)) as Evaluation
		}
	}
}
#[derive(Debug, Clone)]
pub struct GameTree<G: Game>
where
	G::S: Clone,
{
	pub(crate) root_id: usize,
	pub(crate) nodes: Vec<Node<G::M>>,
	pub states: NHHashMap<u64, StateInfo<G::S>>,
}
impl<G: Game> Default for GameTree<G>
where
	G::S: Clone,
{
	fn default() -> Self {
		Self {
			root_id: usize::MAX,
			nodes: Default::default(),
			states: NHHashMap::default(),
		}
	}
}
impl<G: Game> GameTree<G>
where
	G::S: Clone,
{
	pub fn from(state: G::S) -> Self {
		let mut s = Self::default();
		let mut moves = vec![];
		G::generate_moves(&state, &mut moves);
		let hash = G::get_hash(&state);
		s.root_id = s.nodes.len();
		s.nodes.push(Node {
			state: hash,
			parent: None,
			children: vec![],
			visits: 0.0,
			wins: 0.0,
			draws: 0.0,
			untried_moves: moves,
			player_to_move: G::get_current_player(&state),
			outcome: GameOutcome::OnGoing,
			incoming_move: None,
		});
		let hash = G::get_hash(&state);
		s.states.insert(
			hash,
			StateInfo {
				state: state,
				expanded_node: 0,
			},
		);
		s
	}
}
impl<G: Game> GameTree<G>
where
	G::S: Clone,
{
	pub fn get_outcome(&mut self, id: usize) -> GameOutcome {
		let state_info = self.states.get(&self.nodes[id].state).unwrap();
		if state_info.expanded_node != id {
			return self.nodes[state_info.expanded_node].outcome;
		}
		if self.nodes[id].outcome != GameOutcome::OnGoing {
			return self.nodes[id].outcome;
		}
		if !self.nodes[id].untried_moves.is_empty() {
			return GameOutcome::OnGoing;
		}

		let player = self.nodes[id].player_to_move;
		let children = self.nodes[id].children.clone();

		let outcomes: Vec<_> = children.iter().map(|c| self.get_outcome(*c)).collect();

		let result = if outcomes.iter().any(|o| o.is_win_for(player)) {
			//TODO: o.is_win_for(G::get_next_player(state))
			player.into()
		} else if outcomes.iter().any(|o| !o.is_ended()) {
			GameOutcome::OnGoing
		} else if outcomes.iter().all(|o| o.is_win_for(player.opponent())) {
			player.opponent().into()
		} else if outcomes.iter().any(|o| o.is_draw()) {
			GameOutcome::Draw
		} else {
			unreachable!()
		};

		self.nodes[id].outcome = result;

		self.nodes[id].outcome
	}

	pub fn expand_all(&mut self, node_id: usize) -> GameOutcome
	where
		G::S: Clone,
	{
		let mut untried = vec![];
		std::mem::swap(&mut self.nodes[node_id].untried_moves, &mut untried);

		for m in &untried {
			let state = self.get_node_state(node_id).unwrap();
			let new_state = AppliedMove::<G>::applied_clone(&state, *m);
			let new_state_hash = G::get_hash(&new_state);

			let mut moves = vec![];
			let outcome = G::generate_moves(&new_state, &mut moves);
			let child_id = self.nodes.len();
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
				incoming_move: Some(*m),
			});
			let new_state_entry = self.states.get(&new_state_hash);
			if let Some(_entry) = new_state_entry {
				//let child_id = entry.expanded_node;
				self.nodes[node_id].children.push(child_id);
			} else {
				self.nodes[node_id].children.push(child_id);
				self.states.insert(
					new_state_hash,
					StateInfo {
						state: new_state,
						expanded_node: child_id,
					},
				);

				let result = self.expand_all(child_id);
				// pruning
				if result.is_win_for(self.nodes[node_id].player_to_move) {
					//TODO: o.is_win_for(G::get_next_player(state))
					return self.nodes[node_id].player_to_move.into();
				}
			}
		}
		self.get_outcome(node_id)
		//GameOutcome::OnGoing
	}
	pub fn simulate(&self, node_id: usize) -> GameOutcome
	where
		G::S: Clone,
	{
		let mut sim_state = self.get_node_state(node_id).unwrap().clone();
		let mut hash = G::get_hash(&sim_state);
		if let Some(n) = self.get_expanded_node(hash) {
			if n.outcome.is_ended() {
				return n.outcome;
			}
		}
		let mut result = G::get_outcome(&sim_state);

		while !result.is_ended() {
			let mut moves = vec![];
			result = G::generate_moves(&sim_state, &mut moves);
			if result == GameOutcome::OnGoing {
				let m = fastrand::choice(moves);
				sim_state = AppliedMove::<G>::applied_clone(&mut sim_state, m.unwrap());
				hash = G::get_hash(&sim_state);
				if let Some(n) = self.get_expanded_node(hash) {
					if n.outcome.is_ended() {
						return n.outcome;
					}
				}
			}
		}
		result
	}
	pub fn get_root(&self) -> &Node<G::M> {
		&self.nodes[self.root_id]
	}
	pub fn get_root_id(&self) -> usize {
		self.root_id
	}
	pub fn set_root_id(&mut self, id: usize) -> bool {
		if id < self.nodes.len() {
			self.root_id = id;
			true
		} else {
			false
		}
	}

	pub fn cleanup(&mut self) -> NHHashMap<usize, usize> {
		let mut ids = NHHashMap::default();
		self.collect_ids(self.root_id, &mut ids);
		let mut i = 0;
		while i < self.nodes.len() {
			if !ids.contains_key(&i) {
				let moved_id = self.nodes.len() - 1;
				self.nodes.swap_remove(i);
				if ids.contains_key(&moved_id) {
					//safety: just tested by contains key
					*ids.get_mut(&moved_id).unwrap() = i;
					i += 1;
				}
			} else {
				i += 1;
			}
		}
		for node in &mut self.nodes {
			if let Some(p) = node.parent {
				node.parent = ids.get(&p).map(|i| *i);
			}
			// safety: children have been collected
			node.children = node.children.iter().map(|c| ids[c]).collect();
		}

		self.root_id = ids[&self.root_id];
		let mut to_remove = vec![];
		for (hash, info) in self.states.iter_mut() {
			let id = ids.get(&info.expanded_node);
			if let Some(id) = id {
				info.expanded_node = *id;
			} else {
				to_remove.push(*hash);
			}
		}
		for hash in to_remove {
			self.states.remove(&hash);
		}
		ids
	}
	fn collect_ids(&self, root: usize, ids: &mut NHHashMap<usize, usize>) {
		ids.insert(root, root);
		let exp = self.get_node_state_info(root);
		if let Some(exp) = exp {
			if !ids.contains_key(&exp.expanded_node) {
				ids.insert(exp.expanded_node, exp.expanded_node);
				for i in &self.get_node(exp.expanded_node).unwrap().children {
					self.collect_ids(*i, ids);
				}
			}
		}
		for i in &self.get_node(root).unwrap().children {
			self.collect_ids(*i, ids);
		}
	}
	pub fn get_root_state(&self) -> &G::S {
		&self.states[&self.nodes[self.root_id].state].state
	}
	pub fn get_state(&self, hash: u64) -> Option<&G::S> {
		self.states.get(&hash).map(|s| &s.state)
	}
	pub fn get_state_mut(&mut self, hash: u64) -> Option<&mut G::S> {
		self.states.get_mut(&hash).map(|s| &mut s.state)
	}
	pub fn get_node_state(&self, id: usize) -> Option<&G::S> {
		self.states.get(&self.nodes[id].state).map(|s| &s.state)
	}
	pub fn get_node_state_mut(&mut self, id: usize) -> Option<&mut G::S> {
		self.states
			.get_mut(&self.nodes[id].state)
			.map(|s| &mut s.state)
	}
	pub(crate) fn get_node_state_info(&self, id: usize) -> Option<&StateInfo<G::S>> {
		self.states.get(&self.nodes[id].state)
	}
	pub fn get_node_state_info_mut(&mut self, id: usize) -> Option<&mut StateInfo<G::S>> {
		self.states.get_mut(&self.nodes[id].state)
	}
	pub fn get_node(&self, id: usize) -> Option<&Node<G::M>> {
		self.nodes.get(id)
	}
	pub fn get_node_mut(&mut self, id: usize) -> Option<&mut Node<G::M>> {
		self.nodes.get_mut(id)
	}
	pub fn get_expanded_node(&self, state_hash: u64) -> Option<&Node<G::M>> {
		self.nodes.get(self.states.get(&state_hash)?.expanded_node)
	}
	pub fn get_expanded_node_mut(&mut self, state_hash: u64) -> Option<&mut Node<G::M>> {
		self.nodes
			.get_mut(self.states.get(&state_hash)?.expanded_node)
	}
	pub fn get_node_expanded_node(&self, id: usize) -> Option<&Node<G::M>> {
		let n = self.get_node(id)?;
		let si = self.states.get(&n.state)?;
		let expanded = self.nodes.get(si.expanded_node);
		if let Some(exp) = expanded {
			return Some(exp);
		}
		Some(n)
	}
	pub fn get_node_expanded_node_mut(&mut self, id: usize) -> Option<&mut Node<G::M>> {
		let n = self.get_node(id)?;
		let si = self.states.get(&n.state)?;
		self.nodes.get_mut(si.expanded_node)
	}
}

impl<G: Game> Display for GameTree<G>
where
	G::S: Clone,
	G::S: std::fmt::Debug,
	G::M: std::fmt::Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		fn dfs<S: std::fmt::Debug, M: std::fmt::Debug>(
			tree: &Vec<Node<M>>,
			id: usize,
			depth: usize,
			states: &NHHashMap<u64, StateInfo<S>>,
			is_link: bool,
			f: &mut Formatter<'_>,
		) -> fmt::Result {
			let node = &tree[id];
			let indent = "  ".repeat(depth);
			let info = states.get(&node.state).unwrap();
			let outcome = if info.expanded_node != id {
				tree[info.expanded_node].outcome
			} else {
				node.outcome
			};
			if !is_link {
				writeln!(
					f,
					"{}Node {} | to_move: {:?} | move: {:?} | visits: {} | winrate: {:.2} | outcome: {:?}",
					indent,
					id,
					node.player_to_move,
					node.incoming_move,
					node.visits as usize,
					node.winrate(),
					outcome
				)?;
			}
			if info.expanded_node != id {
				dfs(tree, info.expanded_node, depth, states, true, f)?;
				return Ok(());
			}
			let mut children = node.children.clone();

			children.sort_by(|&a, &b| tree[b].visits.partial_cmp(&tree[a].visits).unwrap());

			for child in children {
				dfs(tree, child, depth + 1, states, false, f)?;
			}

			Ok(())
		}

		dfs(&self.nodes, 0, 0, &self.states, false, f)
	}
}
#[derive(Default, Clone)]
pub struct PerfectSolver<G: Game>(Option<GameTree<G>>)
where
	G::S: Clone;
impl<G: Game> Strategy<G> for PerfectSolver<G>
where
	G::S: Clone,
{
	fn choose_move(&mut self, state: &G::S) -> Option<G::M> {
		if self.0.is_none() {
			let mut tree = GameTree::<G>::from(state.clone());
			tree.expand_all(tree.root_id);
			self.0 = Some(tree);
		} else {
			let tree = self.0.as_mut().unwrap();
			let hash = G::get_hash(state);
			if let Some(si) = tree.states.get(&hash) {
				tree.set_root_id(si.expanded_node);
				//tree.cleanup();
				tree.expand_all(tree.root_id);
			} else {
				let mut tree = GameTree::<G>::from(state.clone());
				tree.expand_all(tree.root_id);
				self.0 = Some(tree);
			}
		}
		Some(self.0.as_ref()?.find_best_move())
	}
	fn root_value(&self) -> Evaluation {
		if let Some(t) = &self.0 {
			t.get_root().score()
		} else {
			0
		}
	}
}
impl<G> StrategyWithOptions<G, AIOptions> for PerfectSolver<G>
where
	G: Game,
	G::S: Clone,
{
	fn get_options(&self) -> AIOptions {
		AIOptions::default()
	}

	fn reset_with_options(&mut self, _opts: AIOptions) {}
}
