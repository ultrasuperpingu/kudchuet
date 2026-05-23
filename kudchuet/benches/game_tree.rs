#[macro_use]
extern crate bencher;
extern crate kudchuet;
#[path = "../examples/nim.rs"]
mod nim;

use std::hint::black_box;

use bencher::Bencher;
use kudchuet::ai::move_search::{UniformRolloutPolicy, gametree::GameTree, simulate};

use crate::nim::NimGame;

fn bench_simulate_func(b: &mut Bencher) {
	let board = nim::Board::new(100);
	b.iter(|| {
		for _ in 0..1000 {
			let _test = black_box(simulate::<NimGame>(black_box(&board), UniformRolloutPolicy::default()));
		}
	});
}

fn bench_expand_all(b: &mut Bencher) {
	let board = nim::Board::new(100);
	b.iter(|| {
		for _ in 0..100 {
			let mut s = black_box(GameTree::<NimGame, ()>::from(board.clone()));
			let _test = black_box(s.expand_all(black_box(0)));
		}
	});
}

benchmark_group!(
	benches,
	bench_simulate_func,
	bench_expand_all,
	//bench_expand_all_iterative,
);
benchmark_main!(benches);
