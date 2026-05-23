extern crate kudchuet;

#[path = "../examples/nim.rs"]
mod nim;
#[path = "../examples/ttt.rs"]
mod ttt;
use kudchuet::ai::move_search::{Game, Strategy, UniformRolloutPolicy};
use kudchuet::ai::move_search::mcts::MCTS;

use crate::nim::NimGame;
use crate::ttt::TTTGame;

#[test]
fn test_ttt_mcts() {
	let mut s1 = MCTS::<TTTGame, UniformRolloutPolicy<TTTGame>>::default();
	let mut state = ttt::Board::default();
	//s1.opts.use_min_max = false;
	s1.opts.max_nb_iteration = 20000;
	TTTGame::apply(&mut state, ttt::Place { i: 0 });
	TTTGame::apply(&mut state, ttt::Place { i: 1 });
	let m = s1.choose_move(&state);
	println!("{:?}", m);
	//println!("{}", s1.get_tree().unwrap());
	s1.get_tree().unwrap().print(2);
}

#[test]
fn test_nim_mcts() {
	let mut s1 = MCTS::<NimGame, UniformRolloutPolicy<NimGame>>::default();
	//s1.opts.use_min_max = false;
	s1.opts.max_nb_iteration = 20000;
	let state = nim::Board::new(8);
	let m = s1.choose_move(&state);
	println!("{:?}", m);
	//println!("{}", s1.get_tree().unwrap());
	s1.get_tree().unwrap().print(2);
}
