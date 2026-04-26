extern crate kudchuet;

#[path = "../examples/ttt.rs"]
mod ttt;

use kudchuet::{ai::minimax::{ExpectiMinimax, Random, mcts::MCTSTree, util::battle_royale}};

use crate::ttt::TTTGame;

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


// Ensure that a player using mcts against another always results in
// a draw.
#[test]
fn test_ttt_mcts_vs_mcts_always_draws() {
	for _ in 0..100 {
		let mut s1 = MCTSTree::<TTTGame>::default();
		let mut s2 = MCTSTree::<TTTGame>::default();
		assert_eq!(battle_royale(&mut s1, &mut s2), None);
	}
}

// Ensure that a player using mcts against a random one always results in
// either a draw or a win for the former player.
#[test]
fn test_ttt_mcts_vs_random_always_wins_or_draws() {
	//let mut state = ttt::Board::default();
	//while TTTGame::get_outcome(&state) == GameOutcome::OnGoing {
	//	let m = mcts::<TTTGame>(&state, 20000);
	//	TTTGame::apply(&mut state, m);
	//	println!("{}", state);
	//}
	//let state = ttt::Board::default();
	//let mut s1 = MonteCarloTreeSearch::<TTTGame>::new().with_alpha_beta_pruning(false);
	//s1.choose_move(&state);

	for _ in 0..100 {
		let mut s1 = MCTSTree::<TTTGame>::default();
		let mut s2 = Random::new();
		assert_ne!(battle_royale(&mut s1, &mut s2), Some(1));
	}
}
