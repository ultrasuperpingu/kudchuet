extern crate kudchuet;

#[path = "../examples/ttt.rs"]
mod ttt;

use kudchuet::{
	GameOutcome,
	ai::move_search::{ExpectiMinimax, Game, PerfectSolver, Random, mcts::MCTS, util::battle_royale},
};

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
		let mut s1 = MCTS::<TTTGame>::default();
		let mut s2 = MCTS::<TTTGame>::default();
		assert_eq!(battle_royale(&mut s1, &mut s2), None);
	}
}

// Test TTT Perfect Solver
#[test]
fn test_ttt_recursive() {
	let mut game = ttt::Board::default();
	let mut tree: kudchuet::ai::move_search::gametree::GameTree<TTTGame, ()> =
		kudchuet::ai::move_search::gametree::GameTree::from(game.clone());
	assert_eq!(tree.expand_all(0), GameOutcome::Draw);
	assert_eq!(tree.get_outcome(0), GameOutcome::Draw);
	tree.print(2);
	assert_eq!(tree.get_root().depth_to_end(), 9);
	TTTGame::apply(&mut game, ttt::Place { i: 0 });
	TTTGame::apply(&mut game, ttt::Place { i: 1 });
	let node = tree
		.get_state_node_id(TTTGame::get_hash(&game))
		.unwrap();
	tree.set_root_id(node);
	tree.print(2);
	assert_eq!(tree.get_root().outcome(), GameOutcome::PLAYER1);
	assert_eq!(tree.get_root().depth_to_end(), 5);
}
/*
// Test TTT Perfect Solver
#[test]
fn test_ttt_iterative() {
	let mut game = ttt::Board::default();
	let mut tree: kudchuet::ai::move_search::gametree::GameTree<TTTGame, ()> =
		kudchuet::ai::move_search::gametree::GameTree::from(game.clone());
	assert_eq!(tree.expand_all_iterative(0, false), GameOutcome::Draw);
	assert_eq!(tree.get_outcome(0), GameOutcome::Draw);
	tree.print(2);
	assert_eq!(tree.get_root().depth_to_end(), 9);
	TTTGame::apply(&mut game, ttt::Place { i: 0 });
	TTTGame::apply(&mut game, ttt::Place { i: 1 });
	let node = tree
		.get_state_expanded_node_id(TTTGame::get_hash(&game))
		.unwrap();
	tree.set_root_id(node);
	assert_eq!(tree.get_root().outcome(), GameOutcome::PLAYER1);
	assert_eq!(tree.get_root().depth_to_end(), 5);
	tree.print(2);
}
*/

// Ensure that two player using perfect solver always draws.
#[test]
fn test_ttt_perfect_always_draws() {
	for _ in 0..100 {
		let mut s1 = PerfectSolver::<TTTGame>::default();
		let mut s2 = PerfectSolver::<TTTGame>::default();
		assert_eq!(battle_royale(&mut s1, &mut s2), None);
	}
}

// Ensure that a player using mcts against a random one always results in
// either a draw or a win for the former player.
#[test]
fn test_ttt_mcts_vs_random_always_wins_or_draws() {
	for _ in 0..100 {
		let mut s1 = MCTS::<TTTGame>::default();
		let mut s2 = Random::new();
		assert_ne!(battle_royale(&mut s1, &mut s2), Some(1));
	}
}
