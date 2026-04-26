#![cfg(not(target_arch = "wasm32"))]

extern crate chess_lib;

use kudchuet::{GameOutcome, Player};
use kudchuet::ai::minimax::{Evaluation, Evaluator, Game};

use chess_lib::{Board, BoardStatus, ChessMove, Color, MoveGen};

//use minimax::{Game, Strategy};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Chess;

impl Game for Chess {
	type S = Board;
	type M = ChessMove;

	fn generate_moves(b: &Board, moves: &mut Vec<ChessMove>) -> GameOutcome {
		let movegen = MoveGen::new_legal(b);
		moves.clear();
		//moves.reserve(movegen.len());
		for m in movegen {
			moves.push(m);
		}
		//*moves = movegen.into_iter().collect();
		//let mut mv: [ChessMove; 256] = [ChessMove::default(); 256];
		//let len = b.enumerate_moves(&mut mv);
		//moves.reserve(len);
		//moves.extend_from_slice(&mv[0..len]);
		if moves.is_empty() {
			if *b.checkers() == chess_lib::EMPTY {
				GameOutcome::Draw
			} else {
				//TODO
				GameOutcome::PLAYER1
			}
		} else {
			GameOutcome::OnGoing
		}
	}

	fn get_outcome(b: &Board) -> GameOutcome {
		match b.status() {
			BoardStatus::Ongoing => GameOutcome::OnGoing,
			BoardStatus::Stalemate => GameOutcome::Draw,
			//TODO
			BoardStatus::Checkmate => GameOutcome::PLAYER1,
		}
	}

	fn apply(b: &mut Board, m: ChessMove) -> Option<Board> {
		Some(b.make_move_new(m))
	}

	fn get_hash(b: &Board) -> u64 {
		b.get_hash()
	}

	fn notation(_b: &Board, m: ChessMove) -> Option<String> {
		Some(format!("{}", m))
	}
	fn get_current_player(state: &Self::S) -> Player {
		match state.side_to_move() {
			chess_lib::Color::White => Player::PLAYER1,
			chess_lib::Color::Black => Player::PLAYER2,
		}
	}
}

#[derive(Default,Clone, Copy, PartialEq, Eq)]
pub struct Chess2Evaluator;

impl Evaluator for Chess2Evaluator {
	type G = Chess;
	fn evaluate_for(&self, board: &Board, p: Player) -> Evaluation {
		let mut score = 0;
		for sq in 0..64 {
			let sq = unsafe { chess_lib::Square::new(sq) };
			if let Some(piece) = board.piece_on(sq) {
				let value = match piece {
					chess_lib::Piece::Pawn => 1,
					chess_lib::Piece::Knight => 3,
					chess_lib::Piece::Bishop => 3,
					chess_lib::Piece::Rook => 5,
					chess_lib::Piece::Queen => 9,
					chess_lib::Piece::King => 0,
				};
				if board.color_on(sq).unwrap() == Color::White {
					score += value;
				} else {
					score -= value;
				}
			}
		}
		if p == Player::PLAYER1 {
			score
		} else {
			-score
		}
	}
}
/*
fn main() {
	let mut b = Board::default();
	let opts = IterativeOptions::new().verbose();
	let mut strategy = IterativeSearch::new(Evaluator::default(), opts);
	strategy.set_timeout(std::time::Duration::from_secs(1));
	while Chess::get_winner(&b).is_none() {
		println!("{}", b);
		match strategy.choose_move(&b) {
			Some(m) => b = Chess::apply(&mut b, m).unwrap(),
			None => break,
		}
	}
	println!("Checkmate {:?}", b.side_to_move());
}
*/
#[cfg(test)]
mod tests {
	// cargo test --release -p chess chess2::tests::perft_test -- --nocapture
	// depth           count        time        kn/s
	// 0               1       5.1µs       196.1
	// 1              20      30.6µs       653.6
	// 2             400      26.2µs     15267.2
	// 3            8902     132.5µs     67184.9
	// 4          197281     791.3µs    249312.5
	// 5         4865609       6.0ms    817035.4
	// 6       119060324     134.3ms    886308.7
	// 7      3195901860        3.4s    931380.4
	// 8     84998978956       96.8s    878211.9
	use chess_lib::Board;
	use kudchuet::ai::minimax::util::perft;
	use super::Chess;
	#[test]
	fn perft_test() {
		let mut board = Board::default();
		let _nodes = perft::<Chess>(&mut board, 8, true);
	}
}