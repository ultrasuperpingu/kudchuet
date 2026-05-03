#[macro_use]
extern crate bencher;
extern crate kudchuet;
#[path = "../examples/nim.rs"]
mod nim;

use bencher::Bencher;
use kudchuet::ai::minimax::gametree::GameTree;

use crate::nim::NimGame;

fn bench_simulate(b: &mut Bencher) {
	let board = nim::Board::new(100);
	b.iter(|| {
		let mut s = GameTree::<NimGame>::from(board.clone());
		for _ in 0..1000 {
			s.simulate(0);
		}
	});
}

fn bench_simulate2(b: &mut Bencher) {
	let board = nim::Board::new(100);
	b.iter(|| {
		let s = GameTree::<NimGame>::from(board.clone());
		for _ in 0..1000 {
			s.simulate2(0);
		}
	});
}
benchmark_group!(
	benches,
	bench_simulate,
	bench_simulate2,
);
benchmark_main!(benches);
