extern crate kudchuet;

#[path = "../examples/ttt.rs"]
mod ttt;

use kudchuet::ai::minimax::{ExpectiMinimax, Random, util::battle_royale};

// Ensure that two players using negamax always results in a draw.
#[test]
fn test_ttt_minimax_always_draws() {
	let mut s1 = ExpectiMinimax::new(ttt::TTTEvaluator::default(), 10, true);
	let mut s2 = ExpectiMinimax::new(ttt::TTTEvaluator::default(), 10, true);
	for _ in 0..100 {
		assert_eq!(battle_royale(&mut s1, &mut s2), None);
	}
}

// Ensure that a player using negamax against a random one always results in
// either a draw or a win for the former player.
#[test]
fn test_ttt_minimax_vs_random_always_wins_or_draws() {
	let mut s1 = ExpectiMinimax::new(ttt::TTTEvaluator::default(), 10, true);
	let mut s2 = Random::new();
	for _ in 0..100 {
		assert_ne!(battle_royale(&mut s1, &mut s2), Some(1));
	}
}
