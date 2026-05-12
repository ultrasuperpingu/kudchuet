use crate::{
	GameOutcome, Player,
	ai::minimax::{Evaluation, Game, util::AppliedMove},
	utils::NHHashMap,
};

use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Node<M> {
	pub(crate) state_hash: u64,
	pub(crate) parent: Option<usize>,
	pub(crate) children: Vec<usize>,

	pub(crate) visits: u32,
	pub(crate) wins: i32,
	pub(crate) draws: u32,

	pub(crate) untried_moves: Vec<M>,
	pub(crate) player_to_move: Player,
	pub(crate) outcome: GameOutcome,
	pub(crate) depth_to_end: u16,
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
		if self.visits == 0 {
			0.0
		} else {
			self.wins as f32 / self.visits as f32
		}
	}
	pub fn drawrate(&self) -> f32 {
		if self.visits == 0 {
			0.0
		} else {
			self.draws as f32 / self.visits as f32
		}
	}
	pub fn lossrate(&self) -> f32 {
		if self.visits == 0 {
			0.0
		} else {
			(self.visits as f32 - self.draws as f32 - self.wins as f32) / self.visits as f32
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
	pub fn player_to_move(&self) -> Player {
		self.player_to_move
	}
	pub fn depth_to_end(&self) -> u16 {
		self.depth_to_end
	}
	pub fn outcome(&self) -> GameOutcome {
		self.outcome
	}
	pub fn incoming_move(&self) -> Option<&M> {
		self.incoming_move.as_ref()
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
			state_hash: hash,
			parent: None,
			children: vec![],
			visits: 0,
			wins: 0,
			draws: 0,
			untried_moves: moves,
			player_to_move: G::get_current_player(&state),
			outcome: GameOutcome::OnGoing,
			depth_to_end: u16::MAX,
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
		let state_info = self.states.get(&self.nodes[id].state_hash).unwrap();
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

		let outcomes: Vec<_> = children
			.iter()
			.map(|c| {
				(
					self.get_outcome(*c),
					self.get_node_expanded_node(*c).unwrap().depth_to_end,
				)
			})
			.collect();

		let (result, depth_to_end) = if outcomes.iter().any(|(o, _)| o.is_win_for(player)) {
			//TODO: o.is_win_for(G::get_next_player(state))
			let depth = outcomes
				.iter()
				.filter(|(o, _)| o.is_win_for(player))
				.map(|(_, d)| *d)
				.min()
				.unwrap();
			(player.into(), depth + 1)
		} else if outcomes.iter().any(|(o, _d)| !o.is_ended()) {
			(GameOutcome::OnGoing, u16::MAX)
		} else if outcomes
			.iter()
			.all(|(o, _d)| o.is_win_for(player.opponent()))
		{
			let worst = outcomes
				.iter()
				.filter(|(o, _)| o.is_win_for(player.opponent()))
				.map(|(_, d)| *d)
				.max()
				.unwrap();
			(player.opponent().into(), worst + 1)
		} else if outcomes.iter().any(|(o, _d)| o.is_draw()) {
			let depth = outcomes
				.iter()
				.filter(|(o, _)| o.is_draw())
				.map(|(_, d)| *d)
				.min()
				.unwrap();
			(GameOutcome::Draw, depth + 1)
		} else {
			unreachable!()
		};

		self.nodes[id].outcome = result;
		self.nodes[id].depth_to_end = depth_to_end;

		self.nodes[id].outcome
	}

	pub fn expand_all(&mut self, node_id: usize, use_pruning: bool) -> GameOutcome
	where
		G::S: Clone,
	{
		let mut untried = vec![];
		std::mem::swap(&mut self.nodes[node_id].untried_moves, &mut untried);

		for m in &untried {
			let state = self.get_node_state_mut(node_id).unwrap();
			let new_state = AppliedMove::<G>::applied_clone(state, *m);
			let new_state_hash = G::get_hash(&new_state);

			let mut moves = vec![];
			let outcome = G::generate_moves(&new_state, &mut moves);
			let child_id = self.nodes.len();
			self.nodes.push(Node {
				state_hash: new_state_hash,
				parent: Some(node_id),
				children: vec![],
				visits: 0,
				wins: 0,
				draws: 0,
				untried_moves: moves,
				player_to_move: G::get_current_player(&new_state),
				outcome,
				depth_to_end: if outcome.is_ended() { 0 } else { u16::MAX },
				incoming_move: Some(*m),
			});
			let new_state_entry = self.states.get(&new_state_hash);
			if let Some(_entry) = new_state_entry {
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

				let result = self.expand_all(child_id, use_pruning);
				// pruning
				if use_pruning && result.is_win_for(self.nodes[node_id].player_to_move) {
					//TODO: o.is_win_for(G::get_next_player(state))
					self.nodes[node_id].outcome = result;
					self.nodes[node_id].depth_to_end = self.nodes[child_id].depth_to_end + 1;
					return result;
				}
			}
		}
		self.get_outcome(node_id)
		//GameOutcome::OnGoing
	}
	pub fn expand_all_iterative(&mut self, root_id: usize, use_pruning: bool) -> GameOutcome
	where
		G::S: Clone,
	{
		let mut stack = vec![];

		stack.push(root_id);

		while let Some(node_id) = stack.pop() {
			if use_pruning
				&& self.get_node_expanded_node(node_id).unwrap().outcome != GameOutcome::OnGoing
			{
				self.get_outcome(node_id);
				continue;
			}
			let m = {
				let moves = &mut self.nodes[node_id].untried_moves;
				if moves.is_empty() {
					None
				} else {
					Some(moves.pop().unwrap())
				}
			};
			if let Some(m) = m {
				stack.push(node_id);

				let state = self.get_node_state_mut(node_id).unwrap();
				let new_state = AppliedMove::<G>::applied_clone(state, m);
				let new_state_hash = G::get_hash(&new_state);

				let mut child_moves = vec![];
				let outcome = G::generate_moves(&new_state, &mut child_moves);

				let child_id = self.nodes.len();

				self.nodes.push(Node {
					state_hash: new_state_hash,
					parent: Some(node_id),
					children: vec![],
					visits: 0,
					wins: 0,
					draws: 0,
					untried_moves: child_moves,
					player_to_move: G::get_current_player(&new_state),
					outcome,
					depth_to_end: if outcome.is_ended() { 0 } else { u16::MAX },
					incoming_move: Some(m),
				});

				self.nodes[node_id].children.push(child_id);

				if !self.states.contains_key(&new_state_hash) {
					self.states.insert(
						new_state_hash,
						StateInfo {
							state: new_state,
							expanded_node: child_id,
						},
					);
					stack.push(child_id);
				}
			} else {
				// All moves done
				let _result = self.get_outcome(node_id);
			}
		}

		self.get_outcome(root_id)
	}
	pub fn find_best_proved_move(&self) -> Option<<G as Game>::M> {
		if self.nodes[self.root_id]
			.children
			.iter()
			.any(|id| !self.get_node_expanded_node(*id).unwrap().outcome.is_ended())
		{
			return None;
		}
		let mut maximize_depth = false;
		let mut filtered: Vec<_> = self.nodes[self.root_id]
			.children
			.iter()
			.filter(|id| {
				self.get_node_expanded_node(**id)
					.unwrap()
					.outcome
					.is_win_for(self.nodes[self.root_id].player_to_move)
			})
			.collect();
		if filtered.is_empty() {
			filtered = self.nodes[self.root_id]
				.children
				.iter()
				.filter(|id| self.get_node_expanded_node(**id).unwrap().outcome.is_draw())
				.collect();
		}
		if filtered.is_empty() {
			maximize_depth = true;
			filtered = self.nodes[self.root_id]
				.children
				.iter()
				.filter(|id| {
					self.get_node_expanded_node(**id)
						.unwrap()
						.outcome
						.is_lose_for(self.nodes[self.root_id].player_to_move)
				})
				.collect();
		}
		if !filtered.is_empty() {
			let best_child = if !maximize_depth {
				filtered
					.iter()
					.min_by_key(|&&id| self.get_node_expanded_node(*id).unwrap().depth_to_end)
			} else {
				filtered
					.iter()
					.max_by_key(|&&id| self.get_node_expanded_node(*id).unwrap().depth_to_end)
			};
			if let Some(best_child) = best_child {
				let best_move = self.nodes[**best_child]
					.incoming_move
					.clone()
					.expect("root child must have a move");
				return Some(best_move);
			}
		}

		None
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
		let mut moves = vec![];

		while !result.is_ended() {
			result = G::generate_and_filter_moves(&sim_state, &mut moves);
			if result == GameOutcome::OnGoing {
				let m = if G::is_random_move(&sim_state) {
					let mut sum_proba = 0.0;
					let rand = fastrand::f32();
					let mut ch_mv = &moves[0];
					for mv in &moves {
						sum_proba += G::get_probability(&sim_state, *mv);
						if sum_proba > rand {
							ch_mv = mv;
							break;
						}
					}
					ch_mv
				} else {
					if moves.is_empty() {
						println!("{hash}: {sim_state:?}");
					}
					fastrand::choice(&moves).unwrap()
				};
				if let Some(state) = G::apply(&mut sim_state, *m) {
					sim_state = state;
				}
				hash = G::get_hash(&sim_state);
				if let Some(n) = self.get_expanded_node(hash) {
					if n.outcome.is_ended() {
						return n.outcome;
					}
				}
			}
			moves.clear();
		}

		result
	}

	pub fn simulate2(&mut self, node_id: usize) -> GameOutcome
	where
		G::S: Clone,
	{
		Self::simulate_from_state(self.get_node_state_mut(node_id).unwrap())
	}
	pub fn simulate_from_state(state: &mut G::S) -> GameOutcome
	where
		G::S: Clone,
	{
		let mut moves = Vec::with_capacity(64);
		Self::simulate_from_state_with_pool(state, &mut moves)
	}
	pub fn simulate_from_state_with_pool(
		state: &mut G::S,
		moves_pool: &mut Vec<G::M>,
	) -> GameOutcome
	where
		G::S: Clone,
	{
		//let outcome = G::get_outcome(&*state);
		//if outcome.is_ended() {
		//	return outcome;
		//}

		let result = G::generate_and_filter_moves(&*state, moves_pool);
		if result.is_ended() {
			return result;
		}
		if moves_pool.is_empty() {
			panic!("Unfinished game returns no moves");
		}

		let m = if G::is_random_move(state) {
			let mut sum_proba = 0.0;
			let rand = fastrand::f32();
			let mut ch_mv = moves_pool.get(0).unwrap();
			for mv in moves_pool.iter() {
				sum_proba += G::get_probability(&state, *mv);
				if sum_proba > rand {
					ch_mv = mv;
					break;
				}
			}
			ch_mv
		} else {
			fastrand::choice(moves_pool.iter()).unwrap()
		};

		let mut next = AppliedMove::<G>::new(&mut *state, *m);

		moves_pool.clear();
		Self::simulate_from_state_with_pool(&mut next, moves_pool)
	}

	pub fn get_root(&self) -> &Node<G::M> {
		&self.nodes[self.root_id]
	}
	pub fn get_root_id(&self) -> usize {
		self.root_id
	}
	pub fn nb_nodes(&self) -> usize {
		self.nodes.len()
	}
	pub fn set_root_id(&mut self, id: usize) -> bool {
		if id < self.nodes.len() {
			self.root_id = id;
			//self.nodes[id].parent = None;
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
		&self.states[&self.nodes[self.root_id].state_hash].state
	}
	pub fn get_state(&self, hash: u64) -> Option<&G::S> {
		self.states.get(&hash).map(|s| &s.state)
	}
	pub fn get_state_mut(&mut self, hash: u64) -> Option<&mut G::S> {
		self.states.get_mut(&hash).map(|s| &mut s.state)
	}
	pub fn get_state_expanded_node_id(&self, hash: u64) -> Option<usize> {
		Some(*self.states.get(&hash).map(|s| &s.expanded_node)?)
	}
	pub fn get_node_state(&self, id: usize) -> Option<&G::S> {
		self.states.get(&self.nodes[id].state_hash).map(|s| &s.state)
	}
	pub fn get_node_state_mut(&mut self, id: usize) -> Option<&mut G::S> {
		self.states
			.get_mut(&self.nodes[id].state_hash)
			.map(|s| &mut s.state)
	}
	pub fn get_node_state_info(&self, id: usize) -> Option<&StateInfo<G::S>> {
		self.states.get(&self.nodes[id].state_hash)
	}
	pub fn get_node_state_info_mut(&mut self, id: usize) -> Option<&mut StateInfo<G::S>> {
		self.states.get_mut(&self.nodes[id].state_hash)
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
		let si = self.states.get(&n.state_hash)?;
		let expanded = self.nodes.get(si.expanded_node);
		if let Some(exp) = expanded {
			return Some(exp);
		}
		Some(n)
	}
	pub fn get_node_expanded_node_mut(&mut self, id: usize) -> Option<&mut Node<G::M>> {
		let n = self.get_node(id)?;
		let si = self.states.get(&n.state_hash)?;
		self.nodes.get_mut(si.expanded_node)
	}
	fn dfs_print<W: std::io::Write>(
		tree: &Vec<Node<G::M>>,
		id: usize,
		depth: usize,
		max_depth: usize,
		states: &NHHashMap<u64, StateInfo<G::S>>,
		is_link: bool,
		f: &mut W,
	) -> fmt::Result {
		if max_depth < depth {
			return Ok(());
		}
		let node = &tree[id];
		let indent = "  ".repeat(depth);
		let info = states.get(&node.state_hash).unwrap();
		let outcome = if info.expanded_node != id {
			tree[info.expanded_node].outcome
		} else {
			node.outcome
		};
		if !is_link {
			let _ = writeln!(
				f,
				"{}Node {} | to_move: {:?} | move: {:?} | visits: {} | winrate: {:.2} | outcome: {:?} ({})",
				indent,
				id,
				node.player_to_move,
				node.incoming_move,
				node.visits as usize,
				node.winrate(),
				outcome,
				node.depth_to_end,
			);
		}
		if info.expanded_node != id {
			Self::dfs_print(tree, info.expanded_node, depth, max_depth, states, true, f)?;
			return Ok(());
		}
		let mut children = node.children.clone();

		children.sort_by(|&a, &b| tree[b].visits.partial_cmp(&tree[a].visits).unwrap());

		for child in children {
			Self::dfs_print(tree, child, depth + 1, max_depth, states, false, f)?;
		}

		Ok(())
	}
	pub fn print(&self, max_depth: usize) {
		let _ = Self::dfs_print(
			&self.nodes,
			self.root_id,
			0,
			max_depth,
			&self.states,
			false,
			&mut std::io::stdout(),
		);
	}
}

impl<G: Game> Display for GameTree<G>
where
	G::S: Clone,
	G::S: std::fmt::Debug,
	G::M: std::fmt::Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		struct FmtWriter<'a, 'b>(&'a mut Formatter<'b>);
		impl<'a, 'b> std::io::Write for FmtWriter<'a, 'b> {
			fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
				match std::str::from_utf8(buf) {
					Ok(s) => {
						self.0.write_str(s).map_err(|_| std::io::ErrorKind::Other)?;
						Ok(buf.len())
					}
					Err(_) => Err(std::io::ErrorKind::InvalidData.into()),
				}
			}
			fn flush(&mut self) -> std::io::Result<()> {
				Ok(())
			}
		}

		let mut fw = FmtWriter(f);
		Self::dfs_print(
			&self.nodes,
			self.root_id,
			0,
			usize::MAX,
			&self.states,
			false,
			&mut fw,
		)
	}
}
