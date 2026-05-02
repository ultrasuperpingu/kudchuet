use crate::{
	StrategyWithOptions,
	ai::{
		AIOptions,
		minimax::{Evaluation, Game, Strategy, gametree::GameTree},
	},
};

#[derive(Clone)]
pub struct PerfectSolver<G: Game>(Option<GameTree<G>>)
where
	G::S: Clone;

impl<G: Game> PerfectSolver<G>
where
	G::S: Clone,
{
	pub fn get_tree(&self) -> Option<&GameTree<G>> {
		self.0.as_ref()
	}
}

impl<G: Game> Strategy<G> for PerfectSolver<G>
where
	G::S: Clone,
{
	fn choose_move(&mut self, state: &G::S) -> Option<G::M> {
		if self.0.is_none() {
			let mut tree = GameTree::<G>::from(state.clone());
			let res = tree.expand_all(tree.root_id, false);
			println!("choose_move res: {:?}", res);
			println!("len: {:?}", tree.get_root().children.len());
			self.0 = Some(tree);
		} else {
			let tree = self.0.as_mut().unwrap();
			let hash = G::get_hash(state);
			if let Some(si) = tree.states.get(&hash) {
				tree.set_root_id(si.expanded_node);
				println!("choose_move res 2: {:?}", tree.get_root().outcome);
				tree.cleanup();
				//tree.expand_all(tree.root_id);
			} else {
				let mut tree = GameTree::<G>::from(state.clone());
				tree.expand_all(tree.root_id, false);
				println!("choose_move res 3: {:?}", tree.get_root().outcome);
				self.0 = Some(tree);
			}
		}
		self.0.as_ref()?.find_best_proved_move()
	}
	fn root_value(&self) -> Evaluation {
		if let Some(t) = &self.0 {
			t.get_root().score()
		} else {
			0
		}
	}
}
impl<G: Game> Default for PerfectSolver<G>
where
	G::S: Clone,
{
	fn default() -> Self {
		Self(Default::default())
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
