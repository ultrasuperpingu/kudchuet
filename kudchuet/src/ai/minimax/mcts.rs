use core::f64;
use std::ops::{Deref, DerefMut};

use crate::{
	GameOutcome, Player,
	ai::minimax::{Game, Strategy, util::AppliedMove},
};

pub struct Node<S, M> {
	state: S,
	parent: Option<usize>,
	children: Vec<usize>,

	visits: f64,
	wins: f64,

	untried_moves: Vec<M>,
	player_to_move: Player,
	outcome: GameOutcome,
	incoming_move: Option<M>,
}
impl<G: Game> MCTSTree<G> {
	fn get_outcome(&mut self, id: usize, root_player: Player) -> GameOutcome {
		if self.0[id].untried_moves.is_empty() && self.0[id].outcome == GameOutcome::OnGoing {
			let children = self.0[id].children.clone();

			let outcomes: Vec<_> = children
				.iter()
				.map(|c| self.get_outcome(*c, root_player))
				.collect();

			let result = if self.0[id].player_to_move == root_player {
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

			self.0[id].outcome = result;
		}

		self.0[id].outcome
	}
}

impl<S, M: std::fmt::Debug> Node<S, M> {
	pub fn winrate(&self) -> f64 {
		if self.visits == 0.0 {
			0.0
		} else {
			self.wins / self.visits
		}
	}
}

fn ucb1(parent_visits: f64, wins: f64, visits: f64) -> f64 {
	if visits == 0.0 {
		return f64::INFINITY;
	}
	let exploitation = wins / visits;
	let exploration = (parent_visits.ln().max(1.0) / visits).sqrt();
	exploitation + f64::consts::SQRT_2 * exploration
}
pub struct MCTSTree<G: Game>(Vec<Node<G::S, G::M>>);
impl<G: Game> Default for MCTSTree<G> {
	fn default() -> Self {
		Self(Default::default())
	}
}
impl<G: Game> Deref for MCTSTree<G> {
	type Target = Vec<Node<G::S, G::M>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<G: Game> DerefMut for MCTSTree<G> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
pub fn mcts<G: Game>(root_state: &G::S, iterations: usize) -> G::M
where
	G::S: Clone,
{
	let mut tree = MCTSTree::<G>::default();
	let mut moves = vec![];
	G::generate_moves(&root_state, &mut moves);
	tree.push(Node {
		state: root_state.clone(),
		parent: None,
		children: vec![],
		visits: 0.0,
		wins: 0.0,
		untried_moves: moves,
		player_to_move: G::get_current_player(&root_state),
		outcome: GameOutcome::OnGoing,
		incoming_move: None,
	});

	for _ in 0..iterations {
		// 1. SELECTION
		let mut node_id = 0;
		let mut already_computed = false;
		while tree[node_id].untried_moves.is_empty() && !tree[node_id].children.is_empty() {
			let parent_visits = tree[node_id].visits;

			let selected = tree[node_id]
				.children
				.iter()
				.filter(|id| !tree[**id].outcome.is_ended())
				.max_by(|&&a, &&b| {
					let ua = ucb1(parent_visits, tree[a].wins, tree[a].visits);
					let ub = ucb1(parent_visits, tree[b].wins, tree[b].visits);
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
		if already_computed {
			break;
		}

		// 2. EXPANSION
		if !tree[node_id].untried_moves.is_empty() {
			let m = tree[node_id].untried_moves.pop().unwrap();
			let mut state = tree[node_id].state.clone();
			let new_state = AppliedMove::<G>::new(&mut state, m);

			let child_id = tree.len();
			let mut moves = vec![];
			let outcome = G::generate_moves(&new_state, &mut moves);
			tree.push(Node {
				state: new_state.clone(),
				parent: Some(node_id),
				children: vec![],
				visits: 0.0,
				wins: 0.0,
				untried_moves: moves,
				player_to_move: G::get_current_player(&new_state),
				outcome,
				incoming_move: Some(m),
			});

			tree[node_id].children.push(child_id);
			node_id = child_id;
		}

		// 3. SIMULATION
		let mut sim_state = tree[node_id].state.clone();
		let mut result = G::get_outcome(&sim_state);

		while !result.is_ended() {
			let mut moves = vec![];
			result = G::generate_moves(&sim_state, &mut moves);
			if result == GameOutcome::OnGoing {
				let m = fastrand::choice(moves);
				sim_state = AppliedMove::<G>::new(&mut sim_state.clone(), m.unwrap()).clone();
			}
		}

		// 4. BACKPROP (ROOT POV)
		let root_player = tree[0].player_to_move;

		let mut current = Some(node_id);
		while let Some(id) = current {
			tree[id].visits += 1.0;

			if result.is_win_for(root_player) {
				tree[id].wins += 1.0;
			} else if result.is_draw() {
				tree[id].wins += 0.5;
			}
			tree.get_outcome(id, root_player);

			current = tree[id].parent;
		}
	}

	let mut filtered: Vec<_> = tree[0]
		.children
		.iter()
		.filter(|id| tree[**id].outcome == tree[0].player_to_move.into())
		.collect();
	if filtered.is_empty() {
		filtered = tree[0]
			.children
			.iter()
			.filter(|id| tree[**id].outcome == GameOutcome::Draw)
			.collect();
	}
	if filtered.is_empty() {
		filtered = tree[0].children.iter().collect();
	}

	let best_child = filtered
		.iter()
		.max_by(|&&a, &&b| tree[*a].visits.partial_cmp(&tree[*b].visits).unwrap())
		.unwrap();

	let best_move = tree[**best_child]
		.incoming_move
		.clone()
		.expect("root child must have a move");
	//println!("to_move: {}\n{}", tree.0[0].player_to_move, tree);
	best_move
}

use std::fmt::{self, Display, Formatter};

impl<G: Game> Display for MCTSTree<G>
where
	G::S: std::fmt::Debug,
	G::M: std::fmt::Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		fn dfs<S: std::fmt::Debug, M: std::fmt::Debug>(
			tree: &Vec<Node<S, M>>,
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

		dfs(&self.0, 0, 0, f)
	}
}

impl<G: Game> Strategy<G> for MCTSTree<G>
where
	G::S: Clone,
{
	fn choose_move(&mut self, state: &<G as Game>::S) -> Option<<G as Game>::M> {
		Some(mcts::<G>(&state, 20000))
	}
}
