use chess::mychess::ChessBoard;
use kudchuet::ai::move_search::{Game, gametree::{GameTree, ProofOutcome}, test_dfs::PerfectSolver};

//https://wtharvey.com/m8n2.txt
static TWO_MOVES: [&str; 11] = [
	"r2qkb1r/pp2nppp/3p4/2pNN1B1/2BnP3/3P4/PPP2PPP/R2bK2R w KQkq - 1 0",
	"1rb4r/pkPp3p/1b1P3n/1Q6/N3Pp2/8/P1P3PP/7K w - - 1 0",
	"4kb1r/p2n1ppp/4q3/4p1B1/4P3/1Q6/PPP2PPP/2KR4 w k - 1 0",
	"r1b2k1r/ppp1bppp/8/1B1Q4/5q2/2P5/PPP2PPP/R3R1K1 w - - 1 0",
	"5rkr/pp2Rp2/1b1p1Pb1/3P2Q1/2n3P1/2p5/P4P2/4R1K1 w - - 1 0",
	"1r1kr3/Nbppn1pp/1b6/8/6Q1/3B1P2/Pq3P1P/3RR1K1 w - - 1 0",
	"5rk1/1p1q2bp/p2pN1p1/2pP2Bn/2P3P1/1P6/P4QKP/5R2 w - - 1 0",
	"r1nk3r/2b2ppp/p3b3/3NN3/Q2P3q/B2B4/P4PPP/4R1K1 w - - 1 0",
	"r4br1/3b1kpp/1q1P4/1pp1RP1N/p7/6Q1/PPB3PP/2KR4 w - - 1 0",
	"r1b2k1r/ppppq3/5N1p/4P2Q/4PP2/1B6/PP5P/n2K2R1 w - - 1 0",
	"r2q1b1r/1pN1n1pp/p1n3k1/4Pb2/2BP4/8/PPP3PP/R1BQ1RK1 w - - 1 0",
];
//https://wtharvey.com/m8n3.txt
static THREE_MOVES: [&str; 11] = [
	"r1b1kb1r/pppp1ppp/5q2/4n3/3KP3/2N3PN/PPP4P/R1BQ1B1R b kq - 0 1",
	"r3k2r/ppp2Npp/1b5n/4p2b/2B1P2q/BQP2P2/P5PP/RN5K w kq - 1 0",
	"r1b3kr/ppp1Bp1p/1b6/n2P4/2p3q1/2Q2N2/P4PPP/RN2R1K1 w - - 1 0",
	"3q1r1k/2p4p/1p1pBrp1/p2Pp3/2PnP3/5PP1/PP1Q2K1/5R1R w - - 1 0",
	"6k1/ppp2ppp/8/2n2K1P/2P2P1P/2Bpr3/PP4r1/4RR2 b - - 0 1",
	"rn3rk1/p5pp/2p5/3Ppb2/2q5/1Q6/PPPB2PP/R3K1NR b - - 0 1",
	"N1bk4/pp1p1Qpp/8/2b5/3n3q/8/PPP2RPP/RNB1rBK1 b - - 0 1",
	"8/2p3N1/6p1/5PB1/pp2Rn2/7k/P1p2K1P/3r4 w - - 1 0",
	"r1b1k1nr/p2p1ppp/n2B4/1p1NPN1P/6P1/3P1Q2/P1P1K3/q5b1 w - - 1 0",
	"1q2r3/k4p2/prQ2b1p/R7/1PP1B1p1/6P1/P5K1/8 w - - 1 0",
	"r1bqr1k1/ppp2pp1/3p4/4n1NQ/2B1PN2/8/P4PPP/b4RK1 w - - 1 0",
];
//https://wtharvey.com/m8n4.txt
static FOUR_MOVES: [&str; 2] = [
	"r5rk/2p1Nppp/3p3P/pp2p1P1/4P3/2qnPQK1/8/R6R w - - 1 0",
	"1r2k1r1/pbppnp1p/1b3P2/8/Q7/B1PB1q2/P4PPP/3R2K1 w - - 1 0",
];
#[test]
fn test_solve() {
	for fen in TWO_MOVES {
		let board = ChessBoard::from_fen(fen).unwrap();
		println!("{}", board);
		println!("{:?}", board.status());
		//println!("{:?}", board.legal_moves());
		let mut tree = GameTree::<ChessBoard, ()>::from(board);
		//let res = tree.expand_to_depth(0, 3);
		let res = tree.iterative_deepening_solve(0, 3);
		println!("{:?} ({})", res, tree.get_root().depth_to_end());
		tree.print(1);
		assert_eq!(res, ProofOutcome::from(ChessBoard::get_current_player(&board)).with_depth(3));
	}
	for fen in THREE_MOVES {
		let board = ChessBoard::from_fen(fen).unwrap();
		println!("{}", board);
		println!("{:?}", board.status());
		//println!("{:?}", board.legal_moves());
		let mut tree = GameTree::<ChessBoard, ()>::from(board);
		let res = tree.iterative_deepening_solve(0, 5);
		println!("{:?} ({})", res, tree.get_root().depth_to_end());
		tree.print(1);
		assert_eq!(res,  ProofOutcome::from(ChessBoard::get_current_player(&board)).with_depth(5));
	}
	for fen in FOUR_MOVES {
		let board = ChessBoard::from_fen(fen).unwrap();
		println!("{}", board);
		println!("{:?}", board.status());
		//println!("{:?}", board.legal_moves());
		let mut tree = GameTree::<ChessBoard, ()>::from(board);
		let res = tree.expand_to_depth(0, 7);
		println!("{:?} ({})", res, tree.get_root().depth_to_end());
		tree.print(1);
		assert_eq!(res,  ProofOutcome::from(ChessBoard::get_current_player(&board)).with_depth(7));
	}
}


#[test]
fn test_solve_dfs() {
	for fen in TWO_MOVES {
		let mut board = ChessBoard::from_fen(fen).unwrap();
		println!("{}", board);
		println!("{:?}", board.status());
		//println!("{:?}", board.legal_moves());
		let mut solver = PerfectSolver::default();
		//let res = tree.expand_to_depth(0, 3);
		let res = solver.solve::<ChessBoard>(&mut board, 5);
		println!("{:?}", res);
		assert_eq!(res, ProofOutcome::from(ChessBoard::get_current_player(&board)).with_depth(3));
	}
	for fen in THREE_MOVES {
		let mut board = ChessBoard::from_fen(fen).unwrap();
		println!("{}", board);
		println!("{:?}", board.status());
		//println!("{:?}", board.legal_moves());
		let mut solver = PerfectSolver::default();
		//let res = tree.expand_to_depth(0, 3);
		let res = solver.solve::<ChessBoard>(&mut board, 5);
		println!("{:?}", res);
		assert_eq!(res,  ProofOutcome::from(ChessBoard::get_current_player(&board)).with_depth(5));
	}
	for fen in FOUR_MOVES {
		let mut board = ChessBoard::from_fen(fen).unwrap();
		println!("{}", board);
		println!("{:?}", board.status());
		//println!("{:?}", board.legal_moves());
		let mut solver = PerfectSolver::default();
		//let res = tree.expand_to_depth(0, 3);
		let res = solver.solve::<ChessBoard>(&mut board, 7);
		println!("{:?}", res);
		assert_eq!(res,  ProofOutcome::from(ChessBoard::get_current_player(&board)).with_depth(7));
	}
}

