extern crate kudchuet;

#[path = "../examples/ttt.rs"]
mod ttt;

use kudchuet::ai::minimax::{ExpectiMinimax, Random, util::battle_royale};
#[cfg(not(target_arch = "wasm32"))]
use kudchuet::ai::minimax::{MCTSOptions, MonteCarloTreeSearch};
//use kudchuet::ai::minimax::{Negamax, Random};

// Ensure that two players using negamax always results in a draw.
#[test]
fn test_ttt_negamax_always_draws() {
	let mut s1 = ExpectiMinimax::new(ttt::TTTEvaluator::default(), 10);
	let mut s2 = ExpectiMinimax::new(ttt::TTTEvaluator::default(), 10);
	for _ in 0..100 {
		assert_eq!(battle_royale(&mut s1, &mut s2), None);
	}
}

// Ensure that a player using negamax against a random one always results in
// either a draw or a win for the former player.
#[test]
fn test_ttt_negamax_vs_random_always_wins_or_draws() {
	let mut s1 = ExpectiMinimax::new(ttt::TTTEvaluator::default(), 10);
	let mut s2 = Random::new();
	for _ in 0..100 {
		assert_ne!(battle_royale(&mut s1, &mut s2), Some(1));
	}
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_ttt_mcts_vs_random_always_wins_or_draws() {
	let mut s1 = MonteCarloTreeSearch::new(MCTSOptions::default().with_num_threads(1));
	s1.set_max_rollouts(100);
	let mut s2 = Random::new();
	for _ in 0..50 {
		assert_ne!(
			battle_royale::<ttt::TTTGame, _, _>(&mut s1, &mut s2),
			Some(1)
		);
	}
}
