#[macro_use]
extern crate bencher;
extern crate kudchuet;
#[path = "../examples/nim.rs"]
mod nim;

use std::hint::black_box;

use bencher::Bencher;
use kudchuet::ai::minimax::gametree::GameTree;

use crate::nim::NimGame;

fn bench_simulate(b: &mut Bencher) {
	let board = nim::Board::new(100);
	b.iter(|| {
		let s = black_box(GameTree::<NimGame>::from(board.clone()));
		for _ in 0..1000 {
			let _test = black_box(s.simulate(black_box(0)));
		}
	});
}

fn bench_simulate2(b: &mut Bencher) {
	let board = nim::Board::new(100);
	b.iter(|| {
		let mut s = black_box(GameTree::<NimGame>::from(board.clone()));
		for _ in 0..1000 {
			let _test = black_box(s.simulate2(black_box(0)));
		}
	});
}
fn bench_expand_all(b: &mut Bencher) {
	let board = nim::Board::new(100);
	b.iter(|| {
		for _ in 0..100 {
			let mut s = black_box(GameTree::<NimGame>::from(board.clone()));
			let _test = black_box(s.expand_all(black_box(0), black_box(false)));
		}
	});
}

fn bench_expand_all_iterative(b: &mut Bencher) {
	let board = nim::Board::new(100);
	b.iter(|| {
		for _ in 0..100 {
			let mut s = black_box(GameTree::<NimGame>::from(board.clone()));
			let _test = black_box(s.expand_all_iterative(black_box(0), black_box(false)));
		}
	});
}
benchmark_group!(
	benches,
	bench_simulate,
	bench_simulate2,
	bench_expand_all,
	bench_expand_all_iterative,
);
benchmark_main!(benches);
