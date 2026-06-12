use std::{cmp::Reverse, marker::PhantomData};

use crate::{
	StrategyWithOptions,
	ai::move_search::{Game, Strategy, util::AppliedMove},
	utils::{NHHashMap, NHHashSet},
};
use priority_queue::PriorityQueue;

type Queue = PriorityQueue<u64, Reverse<u32>>;
pub trait Heuristic {
	type G: Game;
	fn heuristic(&self, state: &<Self::G as Game>::S) -> u32;
}
#[derive(Debug, Default)]
pub struct AStar<E: Heuristic>
where
	<E::G as Game>::S: Clone,
{
	evaluator: E,
	computed: NHHashMap<u64, <E::G as Game>::M>,
	principal: Vec<<E::G as Game>::M>,
}
impl<E: Heuristic> AStar<E>
where
	<E::G as Game>::S: Clone + Eq,
{
	pub fn search_fastest_win(
		&mut self,
		state: &mut <E::G as Game>::S,
	) -> Vec<(u64, <E::G as Game>::M)> {
		//let mut tree= GameTree::from(state);
		let hash = E::G::get_hash(state);
		let root_player = E::G::get_current_player(&state);

		let mut states = NHHashMap::<u64, <E::G as Game>::S>::default();
		states.insert(hash, state.clone());
		let mut closed = NHHashSet::<u64>::default();
		let mut came_from = NHHashMap::<u64, (u64, <E::G as Game>::M)>::default();

		let mut g_score = NHHashMap::<u64, u32>::default();
		g_score.insert(hash, 0);

		let mut open = Queue::new();
		open.push(hash, Reverse(self.evaluator.heuristic(state)));
		while let Some((current, _)) = open.pop() {
			let mut s = states.remove(&current).unwrap();
			if E::G::get_outcome(&s).is_win_for(root_player) {
				let res=Self::build_path(&came_from, current);
				println!("found len: {}", res.len());
				return res;
			}
			closed.insert(current);
			let mut moves = vec![];
			E::G::generate_and_filter_moves(&s, &mut moves);
			for m in moves {
				let child_state = AppliedMove::<E::G>::applied_clone(&mut s, m);
				let child_hash = E::G::get_hash(&child_state);
				states.insert(child_hash, child_state.clone());
				if closed.contains(&child_hash) {
					continue;
				}
				let score = g_score.get(&current).unwrap() + 1;
				if !open.contains(&child_hash) || score < *g_score.get(&child_hash).unwrap() {
					open.remove(&child_hash); //if any
					came_from.insert(child_hash, (current, m));
					g_score.insert(child_hash, score);
					open.push(
						child_hash,
						Reverse(score + self.evaluator.heuristic(&child_state)),
					);
				}
			}
		}
		vec![]
	}
	fn build_path(
		came_from: &NHHashMap<u64, (u64, <E::G as Game>::M)>,
		current_node: u64,
	) -> Vec<(u64, <E::G as Game>::M)> {
		if came_from.contains_key(&current_node) {
			let m = came_from.get(&current_node).unwrap();
			let mut path = Self::build_path(came_from, m.0);
			path.push(*m);
			return path;
		}
		vec![]
	}
}
pub struct TrivialHeuristic<G: Game>(PhantomData<G>);
impl<G: Game> Default for TrivialHeuristic<G> {
	fn default() -> Self {
		Self(Default::default())
	}
}
impl<G: Game> Heuristic for TrivialHeuristic<G> {
	type G = G;

	fn heuristic(&self, _state: &<Self::G as Game>::S) -> u32 {
		0
	}
}
impl<E: Heuristic> Strategy<E::G> for AStar<E>
where
	E::G: Game,
	<E::G as Game>::M: PartialEq,
	<E::G as Game>::S: Eq + Clone,
{
	fn choose_move(&mut self, state: &<E::G as Game>::S) -> Option<<E::G as Game>::M> {
		let h = E::G::get_hash(state);

		if let Some(m) = self.computed.get(&h) {
			// TODO: check prefix removing is ok...
			if let Some(pos) = self.principal.iter().position(|e| e == m) {
				self.principal.drain(0..pos);
			}
			return Some(*m);
		} else {
			self.computed.clear();
			self.principal.clear();
		}

		let path = self.search_fastest_win(&mut state.clone());
		if !path.is_empty() {
			for (key, val) in &path {
				self.computed.insert(*key, *val);
			}
			self.principal = path.iter().map(|e| e.1).collect();
			path.first().map(|m| m.1)
		} else {
			None
		}
	}

	fn root_value(&self) -> crate::ai::move_search::Evaluation {
		0
	}
	fn principal_variation(&self) -> Vec<<E::G as Game>::M> {
		self.principal.clone()
	}
}

impl<E: Heuristic> StrategyWithOptions<E::G> for AStar<E>
where
	E::G: Game,
	<E::G as Game>::M: PartialEq,
	<E::G as Game>::S: Eq + Clone,
{
	fn get_options(&self) -> std::collections::HashMap<String, crate::ai::uci::UciValue> {
		std::collections::HashMap::new()
	}

	fn set_options(&mut self, _opts: &std::collections::HashMap<String, crate::ai::uci::UciValue>) {
		//todo!()
	}
}
