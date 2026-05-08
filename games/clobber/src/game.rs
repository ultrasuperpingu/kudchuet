//use std::hash::{DefaultHasher, Hash, Hasher};

use bitboard::{BitIter, Bitboard};
use kudchuet::{GameOutcome, Player};

use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

use crate::rules::{self, Bitboard5x6};

use super::rules::{Clobber, Move};

impl Game for Clobber {
	type S = Clobber;
	type M = Move;

	fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>) -> GameOutcome {
		state.legal_moves_inplace(moves);
		if moves.is_empty() {
			Self::get_current_player(state).opponent().into()
		} else {
			GameOutcome::OnGoing
		}
	}

	fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S> {
		let mut s = state.clone();
		s.play_unchecked(m);
		Some(s)
	}

	fn notation(_state: &Self::S, mv: Self::M) -> Option<String> {
		Some(format!("{:?}", mv))
	}

	fn get_hash(state: &Self::S) -> u64 {
		//let mut hasher = DefaultHasher::new();
		//state.hash(&mut hasher);
		//hasher.finish()
		state.get_hash()
	}
	fn get_current_player(state: &Self::S) -> Player {
		if state.is_black {
			Player::PLAYER2
		} else {
			Player::PLAYER1
		}
	}
	fn get_outcome(state: &Self::S) -> GameOutcome {
		state.result()
	}
}
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ClobberDumbEval;

impl Evaluator for ClobberDumbEval {
	type G = Clobber;

	fn evaluate_for(&self, _state: &Clobber, _p: Player) -> Evaluation {
		0
	}
}
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ClobberSimpleEval;

impl Evaluator for ClobberSimpleEval {
	type G = Clobber;

	fn evaluate_for(&self, state: &Clobber, p: Player) -> Evaluation {
		let (mine, other) = if p == Player::PLAYER1 {
			(state.white, state.black)
		} else {
			(state.black, state.white)
		};

		let mut score = 0;

		let my_count = mine.count() as i16;
		let other_count = other.count() as i16;
		score += (my_count - other_count) * 19;

		let mut isolated = 0;
		for p in mine.iter_bits() {
			if rules::NEIGHBORS[p as usize].and_const(&mine).is_empty() {
				isolated += 1;
			}
		}
		score -= isolated * 43;

		//let groups = count_groups(mine);
		//score -= (groups as i16 - 1) * 31;

		//let max_group = largest_group_size(mine);
		//score += max_group as i16 * 5;
		let (groups, max_group) = groups_and_max(mine);

		score -= (groups as i16 - 1) * 31;
		score += max_group as i16 * 5;

		score
	}
}
fn groups_and_max(bb: Bitboard5x6) -> (u32, u32) {
	let mut remaining = bb;
	let mut groups = 0;
	let mut max_group = 0;

	while !remaining.is_empty() {
		let p = remaining.lsb();
		let mut local = Bitboard5x6::EMPTY;

		flood_fill(bb, p as u8, &mut local);

		let size = local.count();
		if size > max_group {
			max_group = size;
		}

		groups += 1;
		remaining &= local.not_const(); // remove visited bits
	}

	(groups, max_group)
}
/*
fn count_groups(bb: Bitboard5x6) -> u32 {
	let mut remaining = bb;
	let mut count = 0;

	while remaining != Bitboard5x6::EMPTY {
		let p = remaining.lsb();
		let mut group = Bitboard5x6::EMPTY;

		flood_fill(bb, p as u8, &mut group);

		remaining &= !group;
		count += 1;
	}

	count
}

fn largest_group_size(bb: Bitboard5x6) -> u32 {
	let mut seen = Bitboard5x6::EMPTY;
	let mut best = 0;

	let mut remaining = bb;
	while remaining != Bitboard5x6::EMPTY {
		let p = remaining.lsb();
		let mut local = Bitboard5x6::EMPTY;

		flood_fill(bb, p as u8, &mut local);

		seen |= local;
		remaining &= !local;

		best = best.max(local.count());
	}

	best
}
*/
fn flood_fill(bb: Bitboard5x6, start: u8, out: &mut Bitboard5x6) {
	let mut stack = vec![start];
	while let Some(p) = stack.pop() {
		if out.get_at_index(p as usize) {
			continue;
		}
		out.set_at_index(p as usize);

		for n in rules::NEIGHBORS[p as usize]
			.and_const(&bb)
			.and_const(&out.not_const())
			.iter_bits()
		{
			stack.push(n as u8);
		}
	}
}

#[cfg(test)]
mod tests {

	use kudchuet::ai::minimax::util::perft;

	use super::Clobber;
	//cargo test -p clobber --release game::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	//    0               1      10.5µs        95.2
	//    1              49       5.1µs      9607.8
	//    2            2116       8.8µs    240454.5
	//    3           80063     198.8µs    402731.4
	//    4         2630382       1.6ms   1636623.9
	//    5        74662024      27.3ms   2735433.8
	//    6      1816303198     611.2ms   2971902.0
	//    7     37832381602       14.8s   2553971.4
	#[test]
	fn perft_test() {
		let mut board = Clobber::default();

		let nodes = perft::<Clobber>(&mut board, 7, true);
		const NB_NODES: [u64; 8] = [
			1,
			49,
			2116,
			80063,
			2630382,
			74662024,
			1816303198,
			37832381602,
		];
		for (i, n) in nodes.iter().enumerate() {
			assert_eq!(NB_NODES[i], *n, "Mismatch at depth {}", i);
		}
	}
}
