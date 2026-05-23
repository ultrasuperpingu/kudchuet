use crate::{
	GameOutcome, Player,
	ai::move_search::{Game, util::AppliedMove},
	utils::NHHashMap,
};

use std::{
	collections::hash_map::Entry,
	fmt::{self, Debug, Display, Formatter},
};

#[derive(Debug, Clone)]
pub struct Node<G: Game, Data> {
	pub(crate) state: G::S,
	pub(crate) parent: Option<usize>,
	pub(crate) children: Vec<Edge<G::M>>,

	pub(crate) data: Data,

	pub(crate) untried_moves: Vec<G::M>,
	pub(crate) player_to_move: Player,
	pub(crate) outcome: GameOutcome,
	pub(crate) depth_to_end: u16,
}
impl<G: Game, Data> Node<G, Data> {
	pub fn player_to_move(&self) -> Player {
		self.player_to_move
	}
	pub fn depth_to_end(&self) -> u16 {
		self.depth_to_end
	}
	pub fn outcome(&self) -> GameOutcome {
		self.outcome
	}
}
#[derive(Debug, Clone)]
pub struct Edge<M> {
	pub mv: M,
	pub child: usize,
}
#[derive(Debug, Clone)]
pub struct GameTree<G: Game, Data>
where
	G::S: Clone,
{
	pub(crate) root_id: usize,
	pub(crate) nodes: Vec<Node<G, Data>>,
	pub(crate) state_to_node: NHHashMap<u64, usize>,
}
impl<G: Game, Data> Default for GameTree<G, Data>
where
	G::S: Clone,
{
	fn default() -> Self {
		Self {
			root_id: usize::MAX,
			nodes: Default::default(),
			state_to_node: NHHashMap::default(),
		}
	}
}
impl<G: Game, Data: Default> GameTree<G, Data>
where
	G::S: Clone,
{
	pub fn from(state: G::S) -> Self {
		let mut s = Self::default();
		let mut moves = vec![];
		G::generate_moves(&state, &mut moves);
		let hash = G::get_hash(&state);
		let player = G::get_current_player(&state);
		s.root_id = s.nodes.len();
		s.nodes.push(Node {
			state,
			parent: None,
			children: vec![],
			data: Data::default(),
			untried_moves: moves,
			player_to_move: player,
			outcome: GameOutcome::OnGoing,
			depth_to_end: u16::MAX,
		});
		s.state_to_node.insert(hash, 0);
		s
	}
}
impl<G: Game, Data: Default> GameTree<G, Data>
where
	G::S: Clone,
{
	pub fn get_outcome(&mut self, id: usize) -> GameOutcome {
		let mut visited = Vec::new();
		self.get_outcome_recursive(id, &mut visited)
	}
	fn get_outcome_recursive(&mut self, id: usize, visited: &mut Vec<usize>) -> GameOutcome {
		if visited.contains(&id) {
			return GameOutcome::InCycle;
		}
		visited.push(id);
		//let state_info = self.states.get(&self.nodes[id].state_hash).unwrap();
		//if state_info.node != id {
		//	return self.nodes[state_info.node].outcome;
		//}
		if self.nodes[id].outcome != GameOutcome::OnGoing {
			let mine = visited.pop();
			debug_assert_eq!(mine, Some(id));
			return self.nodes[id].outcome;
		}
		if !self.nodes[id].untried_moves.is_empty() {
			let mine = visited.pop();
			debug_assert_eq!(mine, Some(id));
			return GameOutcome::OnGoing;
		}

		let player = self.nodes[id].player_to_move;

		let child_ids: Vec<_> = self.nodes[id].children.iter().map(|e| e.child).collect();
		let outcomes: Vec<_> = child_ids
			.iter()
			.map(|c| {
				(
					self.get_outcome_recursive(*c, visited),
					self.get_node(*c).unwrap().depth_to_end,
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
		} else if outcomes.iter().all(|(o, _d)| *o == GameOutcome::InCycle) {
			println!("Ahhhhhhhh");
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
		let mine = visited.pop();
		debug_assert_eq!(mine, Some(id));
		self.nodes[id].outcome
	}

	pub fn expand_all(&mut self, node_id: usize) -> GameOutcome
	where
		G::S: Clone,
	{
		self.expand_to_depth(node_id, u16::MAX)
	}
	pub fn expand_to_depth(&mut self, node_id: usize, max_depth: u16) -> GameOutcome
	where
		G::S: Clone,
	{
		self.expand_to_depth_recursive(node_id, max_depth, 0)
	}
	pub fn iterative_deepening_solve(&mut self, node_id: usize, max_depth: u16) -> GameOutcome
	where
		G::S: Clone,
	{
		let mut res = GameOutcome::OnGoing;
		for i in 1..=max_depth {
			res = self.expand_to_depth_recursive(node_id, i, 0);
			if res.is_ended() {
				break;
			}
		}
		res
	}
	fn expand_to_depth_recursive(
		&mut self,
		node_id: usize,
		max_depth: u16,
		depth: u16,
	) -> GameOutcome
	where
		G::S: Clone,
	{
		// TODO: check this is really correct...
		if self.nodes[node_id].outcome.is_ended()
			&& (depth + self.nodes[node_id].depth_to_end) <= max_depth
		{
			return self.nodes[node_id].outcome;
		}
		if max_depth <= depth {
			return GameOutcome::OnGoing;
		}
		while let Some(m) = self.nodes[node_id].untried_moves.pop() {
			let (child_id, res) = self.expand_single_child(node_id, m);
			if res.is_win_for(self.nodes[node_id].player_to_move) {
				self.nodes[node_id].outcome = res;
				self.nodes[node_id].depth_to_end = u16::min(self.nodes[node_id].depth_to_end, self.nodes[child_id].depth_to_end + 1);
			}
		}
		let children: Vec<_> = self.nodes[node_id]
			.children
			.iter()
			.map(|e| e.child)
			.collect();
		let mut is_lose = true;
		let mut is_draw = false;
		let mut has_unknown = false;
		let mut loose_depth = 0;
		let mut draw_depth = u16::MAX;
		for child_id in children {
			let res = self.expand_to_depth_recursive(child_id, max_depth, depth + 1);
			if res.is_win_for(self.nodes[node_id].player_to_move) {
				self.nodes[node_id].outcome = res;
				self.nodes[node_id].depth_to_end = u16::min(self.nodes[node_id].depth_to_end, self.nodes[child_id].depth_to_end + 1);
				is_lose = false;
			} else if res.is_draw() {
				is_draw = true;
				draw_depth = u16::min(draw_depth, self.nodes[child_id].depth_to_end + 1);
				is_lose = false;
			}  else if !res.is_ended() {
				has_unknown = true;
				is_lose = false;
			} else if is_lose { //lose
				debug_assert!(res.is_lose_for(self.nodes[node_id].player_to_move));
				loose_depth = u16::max(loose_depth, self.nodes[child_id].depth_to_end + 1);
			} else {
				//println!("res: {:?}", res);
				debug_assert!(res.is_lose_for(self.nodes[node_id].player_to_move));
			}
		}
		if is_lose {
			//TODO: real player win (for multiplayer)
			self.nodes[node_id].outcome = self.nodes[node_id].player_to_move.opponent().into();
			self.nodes[node_id].depth_to_end = loose_depth;
				
		} else if self.nodes[node_id].outcome == GameOutcome::OnGoing && !has_unknown && is_draw {
			self.nodes[node_id].outcome = GameOutcome::Draw;
			self.nodes[node_id].depth_to_end = draw_depth;
		}
		self.nodes[node_id].outcome
		//self.get_outcome(node_id)
		//GameOutcome::OnGoing
	}

	pub(crate) fn expand_single_child(&mut self, node_id: usize, m: G::M) -> (usize, GameOutcome) {
		let state = self.get_node_state_mut(node_id).unwrap();
		let new_state = AppliedMove::<G>::applied_clone(state, m);
		let new_state_hash = G::get_hash(&new_state);

		match self.state_to_node.entry(new_state_hash) {
			Entry::Occupied(entry) => {
				self.nodes[node_id].children.push(Edge {
					mv: m,
					child: *entry.get(),
				});
				(*entry.get(), self.nodes[*entry.get()].outcome)
			}
			Entry::Vacant(new_state_entry) => {
				let mut moves = vec![];
				let outcome = G::generate_moves(&new_state, &mut moves);
				let child_id = self.nodes.len();
				let player = G::get_current_player(&new_state);
				self.nodes.push(Node {
					state: new_state,
					parent: Some(node_id),
					children: vec![],
					data: Data::default(),
					untried_moves: moves,
					player_to_move: player,
					outcome,
					depth_to_end: if outcome.is_ended() { 0 } else { u16::MAX },
				});
				self.nodes[node_id].children.push(Edge {
					mv: m,
					child: child_id,
				});
				new_state_entry.insert(child_id);
				(child_id, outcome)
			}
		}
	}
}
impl<G: Game, Data> GameTree<G, Data>
where
	G::S: Clone,
{
	pub fn find_best_proved_move(&self) -> Option<<G as Game>::M> {
		if self.nodes[self.root_id]
			.children
			.iter()
			.any(|c| !self.get_node(c.child).unwrap().outcome.is_ended())
		{
			return None;
		}
		let mut maximize_depth = false;
		let mut filtered: Vec<_> = self.nodes[self.root_id]
			.children
			.iter()
			.filter(|c| {
				self.get_node(c.child)
					.unwrap()
					.outcome
					.is_win_for(self.nodes[self.root_id].player_to_move)
			})
			.collect();
		if filtered.is_empty() {
			filtered = self.nodes[self.root_id]
				.children
				.iter()
				.filter(|c| self.get_node(c.child).unwrap().outcome.is_draw())
				.collect();
		}
		if filtered.is_empty() {
			maximize_depth = true;
			filtered = self.nodes[self.root_id]
				.children
				.iter()
				.filter(|c| {
					self.get_node(c.child)
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
					.min_by_key(|c| self.get_node(c.child).unwrap().depth_to_end)
			} else {
				filtered
					.iter()
					.max_by_key(|c| self.get_node(c.child).unwrap().depth_to_end)
			};
			if let Some(best_child) = best_child {
				let best_move = best_child.mv;
				return Some(best_move);
			}
		}

		None
	}
}
// Simulation
impl<G: Game, Data> GameTree<G, Data>
where
	G::S: Clone,
{
	pub fn simulate(&self, node_id: usize) -> GameOutcome
	where
		G::S: Clone,
	{
		let mut sim_state = self.get_node_state(node_id).unwrap().clone();
		let mut hash = G::get_hash(&sim_state);
		if let Some(n) = self.get_state_node(hash) {
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
				if let Some(n) = self.get_state_node(hash) {
					if n.outcome.is_ended() {
						return n.outcome;
					}
				}
			}
			moves.clear();
		}

		result
	}
}
// Cleanup
impl<G: Game, Data> GameTree<G, Data>
where
	G::S: Clone,
{
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
				node.parent = ids.get(&p).copied();
			}
			node.children = node
				.children
				.iter()
				.map(|e| Edge {
					child: ids[&e.child],
					mv: e.mv,
				})
				.collect();
		}

		self.root_id = ids[&self.root_id];
		let mut to_remove = vec![];
		for (hash, info) in self.state_to_node.iter_mut() {
			let id = ids.get(info);
			if let Some(id) = id {
				*info = *id;
			} else {
				to_remove.push(*hash);
			}
		}
		for hash in to_remove {
			self.state_to_node.remove(&hash);
		}
		ids
	}
	fn collect_ids(&self, root: usize, ids: &mut NHHashMap<usize, usize>) {
		ids.insert(root, root);
		for i in &self.get_node(root).unwrap().children {
			self.collect_ids(i.child, ids);
		}
	}
}
// Accessors
impl<G: Game, Data> GameTree<G, Data>
where
	G::S: Clone,
{
	//TODO: Option
	pub fn get_root(&self) -> &Node<G, Data> {
		&self.nodes[self.root_id]
	}
	pub fn get_depth_to_end(&self) -> u16 {
		self.nodes[self.root_id].depth_to_end
	}
	pub fn get_root_id(&self) -> usize {
		self.root_id
	}
	pub fn len(&self) -> usize {
		self.nodes.len()
	}
	pub fn nb_states(&self) -> usize {
		self.state_to_node.len()
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
	//TODO: Option
	pub fn get_root_state(&self) -> &G::S {
		&self.nodes[self.root_id].state
	}
	pub fn get_state(&self, hash: u64) -> Option<&G::S> {
		self.state_to_node.get(&hash).map(|s| &self.nodes[*s].state)
	}
	pub fn get_state_mut(&mut self, hash: u64) -> Option<&mut G::S> {
		self.state_to_node.get(&hash).map(|s| &mut self.nodes[*s].state)
	}
	pub fn get_state_node_id(&self, hash: u64) -> Option<usize> {
		self.state_to_node.get(&hash).copied()
	}
	pub fn get_state_node(&self, hash: u64) -> Option<&Node<G, Data>> {
		self.state_to_node.get(&hash).map(|s| &self.nodes[*s])
	}
	pub fn get_state_node_mut(&mut self, hash: u64) -> Option<&mut Node<G, Data>> {
		self.state_to_node.get(&hash).map(|s| &mut self.nodes[*s])
	}
	pub fn get_node_state(&self, id: usize) -> Option<&G::S> {
		self.nodes.get(id).map(|n| &n.state)
	}
	pub fn get_node_state_mut(&mut self, id: usize) -> Option<&mut G::S> {
		self.nodes.get_mut(id).map(|n| &mut n.state)
	}
	pub fn get_node(&self, id: usize) -> Option<&Node<G, Data>> {
		self.nodes.get(id)
	}
	pub fn get_node_mut(&mut self, id: usize) -> Option<&mut Node<G, Data>> {
		self.nodes.get_mut(id)
	}
}
// Print routine
impl<G: Game, Data> GameTree<G, Data>
where
	G::S: Clone,
{
	fn dfs_print<W: std::io::Write>(
		tree: &Vec<Node<G, Data>>,
		id: usize,
		depth: usize,
		max_depth: usize,
		incoming_move: Option<G::M>,
		states: &NHHashMap<u64, usize>,
		f: &mut W,
	) -> fmt::Result {
		if max_depth < depth {
			return Ok(());
		}
		let node = &tree[id];
		let indent = "  ".repeat(depth);
		let (outcome, depth_to_end) = { (node.outcome, node.depth_to_end) };

		let _ = writeln!(
			f,
			//"{}Node {} | to_move: {:?} | move: {:?} | visits: {} | winrate: {:.2} | outcome: {:?} ({})",
			"{}Node {} | to_move: {:?} | move: {:?} | outcome: {:?} ({})",
			indent,
			id,
			node.player_to_move,
			incoming_move,
			//node.visits as usize,
			//node.winrate(),
			outcome,
			depth_to_end,
		);

		for e in &node.children {
			Self::dfs_print(tree, e.child, depth + 1, max_depth, Some(e.mv), states, f)?;
		}

		Ok(())
	}
	pub fn print(&self, max_depth: usize) {
		let _ = Self::dfs_print(
			&self.nodes,
			self.root_id,
			0,
			max_depth,
			None,
			&self.state_to_node,
			&mut std::io::stdout(),
		);
	}
}
impl<G: Game, Data> Display for GameTree<G, Data>
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
			None,
			&self.state_to_node,
			&mut fw,
		)
	}
}
