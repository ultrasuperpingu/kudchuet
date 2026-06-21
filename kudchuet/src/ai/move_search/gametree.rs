use crate::{
	GameOutcome, Player,
	ai::move_search::{Game, util::AppliedMove},
	utils::NHHashMap,
};

use std::{
	collections::hash_map::Entry,
	fmt::{self, Debug, Display, Formatter},
};
/*
pub struct PlayerSet(u16);
impl PlayerSet {
	pub fn players(&self) -> [Option<Player>; 16] {
		let mut res: [Option<Player>; 16] = [None; 16];
		for i in 0..16 {
			if (self.0 & (1 << i)) != 0 {
				res[i] = Some(Player(i as u8));
			}
		}
		res
	}
	pub fn contains(&self, p: Player) -> bool {
		(self.0 & 1 << p.0) != 0
	}
}*/
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProofOutcome {
	Unproved,
	Draw(u16),
	Player(Player, u16),
	//PossibleWinners(PlayerSet, u16),
}
impl ProofOutcome {
	pub fn depth(&self) -> Option<u16> {
		match self {
			ProofOutcome::Unproved => None,
			ProofOutcome::Draw(d) => Some(*d),
			ProofOutcome::Player(_, d) => Some(*d),
			//ProvedOutcome::PossibleWinners(_, d) => Some(*d),
		}
	}

	pub fn is_proved(&self) -> bool {
		!matches!(self, Self::Unproved)
	}
	pub fn is_draw(&self) -> bool {
		matches!(self, Self::Draw(_))
	}

	pub fn is_ended(&self) -> bool {
		!matches!(self, Self::Unproved)
	}

	pub fn is_win_for(&self, p: Player) -> bool {
		matches!(self, Self::Player(w, _) if *w == p)
	}

	pub fn is_lose_for(&self, p: Player) -> bool {
		match self {
			Self::Player(w, _) => *w != p,
			_ => false,
		}
	}

	pub fn winner(&self) -> Option<Player> {
		match self {
			Self::Player(p, _) => Some(*p),
			_ => None,
		}
	}

	pub fn increment_depth(self) -> Self {
		match self {
			Self::Unproved => Self::Unproved,
			Self::Draw(d) => Self::Draw(d + 1),
			Self::Player(p, d) => Self::Player(p, d + 1),
		}
	}
	pub fn with_depth(self, depth: u16) -> Self {
		match self {
			Self::Unproved => Self::Unproved,
			Self::Draw(_) => Self::Draw(depth),
			Self::Player(p, _) => Self::Player(p, depth),
		}
	}
}
impl ProofOutcome {
	pub fn merge(&mut self, other: Self, is_max: bool) -> &mut Self {
		use ProofOutcome::*;

		match (&mut *self, other) {
			(Unproved, x) => {
				*self = x;
			}

			(_, Unproved) => {}

			(Draw(a), Draw(b)) => {
				if is_max {
					*a = (*a).max(b);
				} else {
					*a = (*a).min(b);
				}
			}

			(Player(pa, da), Player(pb, db)) => {
				if *pa == pb {
					if is_max {
						*da = (*da).max(db);
					} else {
						*da = (*da).min(db);
					}
				} else {
					panic!("Contradictory proved outcomes");
				}
			}

			(Player(_, _), Draw(_)) | (Draw(_), Player(_, _)) => {
				panic!("Contradictory proved outcomes");
			}
		}

		self
	}
}
impl From<GameOutcome> for ProofOutcome {
	fn from(value: GameOutcome) -> Self {
		match value {
			GameOutcome::Player(p) => ProofOutcome::Player(p, 0),
			GameOutcome::Draw => ProofOutcome::Draw(0),
			GameOutcome::OnGoing => ProofOutcome::Unproved,
		}
	}
}

impl From<ProofOutcome> for GameOutcome {
	fn from(value: ProofOutcome) -> Self {
		match value {
			ProofOutcome::Player(p, _) => GameOutcome::Player(p),
			ProofOutcome::Draw(_) => GameOutcome::Draw,
			ProofOutcome::Unproved => GameOutcome::OnGoing,
		}
	}
}

impl From<Player> for ProofOutcome {
	fn from(value: Player) -> Self {
		ProofOutcome::Player(value, 0)
	}
}
#[derive(Debug, Clone)]
pub struct Node<G: Game, Data> {
	pub(crate) state: G::S,
	pub(crate) parent: Option<usize>,
	pub(crate) children: Vec<Edge<G::M>>,

	pub(crate) data: Data,

	pub(crate) untried_moves: Vec<G::M>,
	pub(crate) player_to_move: Player,
	pub(crate) outcome: ProofOutcome,
	//pub(crate) depth_to_end: u16,
}
impl<G: Game, Data> Node<G, Data> {
	pub fn player_to_move(&self) -> Player {
		self.player_to_move
	}
	pub fn depth_to_end(&self) -> u16 {
		self.outcome.depth().unwrap_or(u16::MAX)
	}
	pub fn outcome(&self) -> ProofOutcome {
		self.outcome
	}
	pub fn untried_moves(&self) -> &Vec<G::M> {
		&self.untried_moves
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
			outcome: ProofOutcome::Unproved,
			//depth_to_end: u16::MAX,
		});
		s.state_to_node.insert(hash, 0);
		s
	}
}
impl<G: Game, Data: Default> GameTree<G, Data>
where
	G::S: Clone,
{
	pub fn expand_all(&mut self, node_id: usize) -> ProofOutcome
	where
		G::S: Clone,
	{
		self.expand_to_depth(node_id, u16::MAX)
	}
	pub fn expand_to_depth(&mut self, node_id: usize, max_depth: u16) -> ProofOutcome
	where
		G::S: Clone,
	{
		self.expand_to_depth_recursive(node_id, max_depth, 0)
	}
	pub fn iterative_deepening_solve(&mut self, node_id: usize, max_depth: u16) -> ProofOutcome
	where
		G::S: Clone,
	{
		let mut res = ProofOutcome::Unproved;
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
	) -> ProofOutcome
	where
		G::S: Clone,
	{
		// TODO: check this is really correct...
		if self.nodes[node_id].outcome.is_ended()
			&& (depth + self.nodes[node_id].outcome.depth().unwrap()) <= max_depth
		{
			return self.nodes[node_id].outcome;
		}
		if max_depth <= depth {
			return ProofOutcome::Unproved;
		}
		while let Some(m) = self.nodes[node_id].untried_moves.pop() {
			let (_child_id, res) = self.expand_single_child(node_id, m);
			if res.is_win_for(self.nodes[node_id].player_to_move) {
				self.nodes[node_id]
					.outcome
					.merge(res.increment_depth(), false);
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
				self.nodes[node_id]
					.outcome
					.merge(res.increment_depth(), false);
				is_lose = false;
			} else if res.is_draw() {
				is_draw = true;
				draw_depth = res.depth().unwrap() + 1;
				is_lose = false;
			} else if !res.is_ended() {
				has_unknown = true;
				is_lose = false;
			} else if is_lose {
				//lose
				debug_assert!(res.is_lose_for(self.nodes[node_id].player_to_move));
				loose_depth = u16::max(loose_depth, res.depth().unwrap() + 1);
			} else {
				debug_assert!(res.is_lose_for(self.nodes[node_id].player_to_move));
			}
		}
		if is_lose {
			//TODO: real player win (for multiplayer)
			self.nodes[node_id].outcome =
				ProofOutcome::Player(self.nodes[node_id].player_to_move.opponent(), loose_depth);
		} else if self.nodes[node_id].outcome == ProofOutcome::Unproved && !has_unknown && is_draw {
			self.nodes[node_id].outcome = ProofOutcome::Draw(draw_depth);
		}
		self.nodes[node_id].outcome
		//self.get_outcome(node_id)
		//GameOutcome::OnGoing
	}

	pub fn expand_single_child(&mut self, node_id: usize, m: G::M) -> (usize, ProofOutcome) {
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
					outcome: outcome.into(),
					//depth_to_end: if outcome.is_ended() { 0 } else { u16::MAX },
				});
				self.nodes[node_id].children.push(Edge {
					mv: m,
					child: child_id,
				});
				new_state_entry.insert(child_id);
				(child_id, outcome.into())
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
					.min_by_key(|c| self.get_node(c.child).unwrap().depth_to_end())
			} else {
				filtered
					.iter()
					.max_by_key(|c| self.get_node(c.child).unwrap().depth_to_end())
			};
			if let Some(best_child) = best_child {
				let best_move = best_child.mv;
				return Some(best_move);
			}
		}

		None
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
		self.nodes[self.root_id].depth_to_end()
	}
	pub fn get_root_id(&self) -> usize {
		self.root_id
	}
	pub fn len(&self) -> usize {
		self.nodes.len()
	}
	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty()
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
		let (outcome, depth_to_end) = { (node.outcome, node.depth_to_end()) };

		let _ = writeln!(
			f,
			//"{}Node {} | player: {:?} | move: {:?} | visits: {} | winrate: {:.2} | outcome: {:?} ({})",
			"{}Node {} | player: {:?} | move: {:?} | outcome: {:?} ({})",
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
