#[macro_use]
extern crate bencher;
extern crate kudchuet;

use bencher::Bencher;
use kudchuet::ai::minimax::gametree::GameTree;

use gomoku::rules::Gomoku;

fn bench_simulate(b: &mut Bencher) {
	let board = Gomoku::default();
	b.iter(|| {
		let s = GameTree::<Gomoku>::from(board.clone());
		for _ in 0..1000 {
			s.simulate(0);
		}
	});
}


fn bench_simulate2(b: &mut Bencher) {
	let board = Gomoku::default();
	b.iter(|| {
		let mut s = GameTree::<Gomoku>::from(board.clone());
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
