use crate::{
	GameOutcome, Player,
	ai::minimax::{Game, Strategy, util::AppliedMove},
};

use std::{
	collections::HashMap,
	fmt::{self, Display, Formatter},
};

pub struct Node<M> {
	pub(crate) state: u64,
	pub(crate) parent: Option<usize>,
	pub(crate) children: Vec<usize>,

	pub(crate) visits: f64,
	pub(crate) wins: f64,
	pub(crate) draws: f64,

	pub(crate) untried_moves: Vec<M>,
	pub(crate) player_to_move: Player,
	pub(crate) outcome: GameOutcome,
	pub(crate) incoming_move: Option<M>,
}

pub struct StateInfo<S> {
	pub(crate) state: S,
	pub(crate) expanded_node: usize,
}
impl<M> Node<M> {
	pub fn winrate(&self) -> f64 {
		if self.visits == 0.0 {
			0.0
		} else {
			self.wins / self.visits
		}
	}
	pub fn drawrate(&self) -> f64 {
		if self.visits == 0.0 {
			0.0
		} else {
			self.draws / self.visits
		}
	}
}
pub struct GameTree<G: Game> {
	pub(crate) root_id: usize,
	pub(crate) nodes: Vec<Node<G::M>>,
	pub states: HashMap<u64, StateInfo<G::S>>,
}
impl<G: Game> Default for GameTree<G> {
	fn default() -> Self {
		Self {
			root_id: usize::MAX,
			nodes: Default::default(),
			states: HashMap::new(),
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
	pub fn get_state(&self, id: u64) -> Option<&G::S> {
		self.states.get(&id).map(|s| &s.state)
	}
}
impl<G: Game> GameTree<G> {
	/*pub fn get_outcome(&mut self, id: usize, root_player: Player) -> GameOutcome {
		if self.nodes[id].untried_moves.is_empty() && self.nodes[id].outcome == GameOutcome::OnGoing
		{
			let player = self.nodes[id].player_to_move;
			let children = self.nodes[id].children.clone();

			let outcomes: Vec<_> = children
				.iter()
				.map(|c| self.get_outcome(*c, root_player))
				.collect();

			let result = if player == root_player {
				if outcomes.iter().any(|o| o.is_win_for(root_player)) {
					root_player.into()
				} else if outcomes.iter().all(|o| o.is_lose_for(root_player)) {
					root_player.opponent().into()
				} else if outcomes.iter().any(|o| o.is_draw()) {
					GameOutcome::Draw
				} else {
					GameOutcome::OnGoing
				}
			} else {
				if outcomes.iter().all(|o| o.is_win_for(root_player)) {
					root_player.into()
				} else if outcomes.iter().any(|o| o.is_lose_for(root_player)) {
					root_player.opponent().into()
				} else if outcomes.iter().any(|o| o.is_draw()) {
					GameOutcome::Draw
				} else {
					GameOutcome::OnGoing
				}
			};

			self.nodes[id].outcome = result;
		}

		self.nodes[id].outcome
	}*/
	pub fn get_outcome(&mut self, id: usize) -> GameOutcome {
		//let hash = self.nodes[id].state;
		//let canonical = self.states[&hash].expanded_node;
		//if canonical != id {
		//	return self.nodes[canonical].outcome;
		//}
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
		let state_hash = self.nodes[node_id].state;
		std::mem::swap(&mut self.nodes[node_id].untried_moves, &mut untried);
		for m in &untried {
			let state = self.states.get(&state_hash).unwrap();
			let state = &state.state;
			let new_state = AppliedMove::<G>::applied_clone(&state, *m);
			let new_state_hash = G::get_hash(&new_state);

			let mut moves = vec![];
			let outcome = G::generate_moves(&new_state, &mut moves);

			let new_state_entry = self.states.get(&new_state_hash);
			if let Some(entry) = new_state_entry {
				let child_id = entry.expanded_node;
				self.nodes[node_id].children.push(child_id);
			} else {
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
				self.nodes[node_id].children.push(child_id);
				self.states.insert(
					new_state_hash,
					StateInfo {
						state: new_state,
						expanded_node: child_id,
					},
				);

				self.expand_all(child_id);
			}
		}
		//let player = self.nodes[node_id].player_to_move;
		//self.get_outcome(node_id, player)
		self.get_outcome(node_id)
	}
	pub fn get_root(&self) -> &Node<G::M> {
		&self.nodes[self.root_id]
	}
	pub fn get_root_state(&self) -> &G::S {
		&self.states[&self.nodes[self.root_id].state].state
	}
}

impl<G: Game> Display for GameTree<G>
where
	G::S: std::fmt::Debug,
	G::M: std::fmt::Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		fn dfs<M: std::fmt::Debug>(
			tree: &Vec<Node<M>>,
			id: usize,
			depth: usize,
			f: &mut Formatter<'_>,
		) -> fmt::Result {
			let node = &tree[id];
			let indent = "  ".repeat(depth);

			writeln!(
				f,
				"{}Node {} | move: {:?} | visits: {} | winrate: {:.2} | outcome: {:?}",
				indent,
				id,
				node.incoming_move,
				node.visits as usize,
				node.winrate(),
				node.outcome
			)?;

			let mut children = node.children.clone();

			children.sort_by(|&a, &b| tree[b].visits.partial_cmp(&tree[a].visits).unwrap());

			for child in children {
				dfs(tree, child, depth + 1, f)?;
			}

			Ok(())
		}

		dfs(&self.nodes, 0, 0, f)
	}
}

pub struct PerfectSolver;
impl<G: Game> Strategy<G> for PerfectSolver
	where G::S: Clone,
{
	fn choose_move(&mut self, state: &G::S) -> Option<G::M> {
		let tree = GameTree::<G>::from(state.clone());
		Some(tree.find_best_move())
	}
}