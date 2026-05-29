use crate::{
	GameOutcome, Player,
	ai::move_search::{Game, gametree::ProofOutcome, util::AppliedMove},
	utils::NHHashMap,
};
pub trait DfsHandler<G: Game, Context> {
	type Output;
	/// Called when entering a state
	///
	/// Typically use to provide early return
	fn on_enter(
		&mut self,
		state: &G::S,
		hash: u64,
		depth: u16,
		max_depth: u16,
	) -> Result<Context, Self::Output>;

	/// Called before child iteration, just after move generation.
	///
	/// Can be used to create a custom Context object that will be pass to other functions
	fn before_iteration(
		&mut self,
		state: &G::S,
		hash: u64,
		context: &mut Context,
		state_outcome: GameOutcome,
	) -> Option<Self::Output>;

	/// Called just before child recursion
	///
	/// Returns if the children should recurse
	fn on_child_recursing(
		&mut self,
		parent: &G::S,
		hash: u64,
		mv: G::M,
		depth: u16,
		context: &mut Context,
	) -> ControlFlow<Self::Output>;

	/// Called just after child recursion
	///
	/// Returns if the children iteration should stop
	fn on_child_recursed(
		&mut self,
		parent: &G::S,
		hash: u64,
		mv: G::M,
		depth: u16,
		child_result: Self::Output,
		context: &mut Context,
	) -> ControlFlow<Self::Output>;
	/// Called when exiting a state
	fn on_exit(&mut self, state: &G::S, hash: u64, depth: u16, context: Context) -> Self::Output;
}
pub enum ControlFlow<Output> {
	Proceed,
	Break,
	Continue,
	Return(Output),
}
pub fn dfs<G, H, Context>(
	state: &mut G::S,
	depth: u16,
	max_depth: u16,
	handler: &mut H,
) -> H::Output
where
	G: Game,
	H: DfsHandler<G, Context>,
{
	let hash = G::get_hash(state);

	let mut context = match handler.on_enter(state, hash, depth, max_depth) {
		Ok(context) => context,
		Err(res) => return res,
	};

	let mut moves = vec![];
	let res = G::generate_and_filter_moves(state, &mut moves);

	match handler.before_iteration(state, hash, &mut context, res) {
		Some(res) => return res,
		None => {}
	}
	for mv in moves {
		match handler.on_child_recursing(state, hash, mv, depth, &mut context) {
			ControlFlow::Proceed => {}
			ControlFlow::Break => break,
			ControlFlow::Continue => continue,
			ControlFlow::Return(res) => return res,
		}
		let res = {
			let mut child = AppliedMove::<G>::new(state, mv);
			//TODO: pass additional arguments like best depth already proven
			dfs::<G, H, Context>(&mut child, depth + 1, max_depth, handler)
		};
		match handler.on_child_recursed(state, hash, mv, depth, res, &mut context) {
			ControlFlow::Proceed => {}
			ControlFlow::Break => break,
			ControlFlow::Continue => continue,
			ControlFlow::Return(res) => return res,
		}
	}

	handler.on_exit(state, hash, depth, context)
}
#[derive(Default)]
pub struct PerfectSolver {
	table: NHHashMap<u64, ProofOutcome>,
}
struct SolveContext {
	player: Player,
	is_lose: bool,
	loose_depth: u16,
	is_draw: bool,
	draw_depth: u16,
	has_unknown: bool,
	has_child: bool,
}
impl PerfectSolver {
	pub fn solve<G: Game>(&mut self, state: &mut G::S, max_depth: u16) -> ProofOutcome {
		dfs::<G, Self, SolveContext>(state, 0, max_depth, self)
	}
}
impl<G: Game> DfsHandler<G, SolveContext> for PerfectSolver {
	type Output = ProofOutcome;

	#[inline]
	fn on_enter(
		&mut self,
		state: &G::S,
		hash: u64,
		depth: u16,
		max_depth: u16,
	) -> Result<SolveContext, Self::Output> {
		if let Some(v) = self.table.get(&hash) {
			//TODO: check condition
			if v.is_ended() && (depth + v.depth().unwrap()) <= max_depth {
				return Err(*v);
			}
		}
		if max_depth < depth {
			return Err(ProofOutcome::Unproved);
		}
		Ok(SolveContext {
			player: G::get_current_player(state),
			is_lose: true,
			loose_depth: 0,
			is_draw: false,
			draw_depth: u16::MAX,
			has_unknown: false,
			has_child: false,
		})
	}

	#[inline]
	fn before_iteration(
		&mut self,
		_state: &<G as Game>::S,
		hash: u64,
		_context: &mut SolveContext,
		state_outcome: GameOutcome,
	) -> Option<Self::Output> {
		match self.table.entry(hash) {
			std::collections::hash_map::Entry::Occupied(occupied_entry) => {
				Some(*occupied_entry.get())
			},
			std::collections::hash_map::Entry::Vacant(vacant_entry) => {
				vacant_entry.insert(state_outcome.into());
				None
			}
		}
	}
	#[inline]
	fn on_child_recursing(
		&mut self,
		_parent: &G::S,
		_hash: u64,
		_mv: G::M,
		_depth: u16,
		context: &mut SolveContext,
	) -> ControlFlow<Self::Output> {
		context.has_child = true;
		ControlFlow::Proceed
	}
	#[inline]
	fn on_child_recursed(
		&mut self,
		_parent: &G::S,
		hash: u64,
		_mv: G::M,
		_depth: u16,
		res: Self::Output,
		context: &mut SolveContext,
	) -> ControlFlow<Self::Output> {
		if res.is_win_for(context.player) {
			self.table
				.get_mut(&hash)
				.unwrap()
				.merge(res.increment_depth(), false);
			context.is_lose = false;
		} else if res.is_draw() {
			context.is_draw = true;
			context.draw_depth = u16::min(context.draw_depth, res.depth().unwrap() + 1);
			context.is_lose = false;
		} else if !res.is_ended() {
			context.has_unknown = true;
			context.is_lose = false;
		} else if context.is_lose {
			//lose
			debug_assert!(res.is_lose_for(context.player));
			context.loose_depth = u16::max(context.loose_depth, res.depth().unwrap() + 1);
		} else {
			debug_assert!(res.is_lose_for(context.player));
		}
		ControlFlow::Proceed
	}
	#[inline]
	fn on_exit(
		&mut self,
		_state: &G::S,
		hash: u64,
		_depth: u16,
		context: SolveContext,
	) -> ProofOutcome {
		let outcome = self.table.entry(hash).or_insert(ProofOutcome::Unproved);
		if context.has_child {
			if context.is_lose {
				//println!(" ==> {:?} {:?}", *outcome, context.player);
				debug_assert!(
					*outcome == ProofOutcome::Unproved || outcome.is_lose_for(context.player)
				);
				//TODO: PlayerSet for multiple player games...
				*outcome = ProofOutcome::Player(context.player.opponent(), context.loose_depth);
			} else if *outcome == ProofOutcome::Unproved && !context.has_unknown && context.is_draw
			{
				*outcome = ProofOutcome::Draw(context.draw_depth);
			}
			//println!("{}{:?}", "  ".repeat(_depth as usize), *outcome);
		}
		*outcome
	}
}
