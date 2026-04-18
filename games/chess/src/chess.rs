use shakmaty::{Chess, Color, EnPassantMode, Move, Position, zobrist::Zobrist64};

use super::evaluation::evaluate_materials;

use kudchuet::Player;
use kudchuet::ai::minimax::{Evaluation, Evaluator, Game, Winner};

#[derive(Clone, Debug)]
pub struct ChessGame;
impl Game for ChessGame {
	type S = Chess;
	type M = Move;

	fn generate_moves(s: &Self::S, moves: &mut Vec<Self::M>) -> Option<Winner> {
		let legals = s.legal_moves();
		moves.clear();
		moves.reserve(legals.len());
		moves.extend_from_slice(&legals[..legals.len()]);
		if legals.is_empty() {
			if s.is_check() {
				if s.turn() == Color::White {
					Some(Winner::Player(0))
				} else {
					Some(Winner::Player(1))
				}
			} else {
				Some(Winner::Draw)
			}
		}
		else if s.is_insufficient_material() {
			Some(Winner::Draw)
		}
		else {
			None
		}
	}

	fn get_winner(state: &Self::S) -> Option<Winner> {
		match state.outcome() {
			shakmaty::Outcome::Known(known_outcome) => {
				match known_outcome {
						shakmaty::KnownOutcome::Decisive { winner } => {
							if winner == Color::White {
								Some(Winner::Player(0))
							} else {
								Some(Winner::Player(1))
							}
						},
						shakmaty::KnownOutcome::Draw => Some(Winner::Draw),
					}
			},
			shakmaty::Outcome::Unknown => None,
		}
	}

	fn apply(state: &mut Self::S, mov: Self::M) -> Option<Self::S> {
		let s=state.clone().play(mov);
		//println!("{:?}", s);
		s.ok()
	}
	fn zobrist_hash(pos: &Self::S) -> u64 {
		pos.zobrist_hash::<Zobrist64>(EnPassantMode::Legal).into()
	}
	fn current_player(state: &Self::S) -> Player {
		if state.turn() == Color::White {
			Player::PLAYER1
		} else {
			Player::PLAYER2
		}
	}
}


#[derive(Clone, Eq, PartialEq)]
pub struct ChessPosEval;
impl ChessPosEval {
	pub fn new() -> Self {
		Self {}
	}
}
impl Default for ChessPosEval {
	fn default() -> Self {
		Self::new()
	}
}
impl Evaluator for ChessPosEval {
	type G = ChessGame;
	fn evaluate_for(&self, state: &Chess, p: Player) -> Evaluation {
		//TODO
		evaluate_materials(state)
	}
}
#[cfg(test)]
mod tests {
	use kudchuet::ai::minimax::util::perft;
	use shakmaty::Chess;
	use super::super::chess::ChessGame;
	// cargo test --release -p chess@0.1 chess::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	//    0               1       7.7µs       129.9
	//    1              20      37.4µs       534.8
	//    2             400      65.9µs      6069.8
	//    3            8902     161.0µs     55291.9
	//    4          197281     874.4µs    225618.7
	//    5         4865609       6.5ms    752933.8
	//    6       119060324     189.8ms    627209.3
	//    7      3195901860        4.6s    688694.5
	//    8     84998978956      139.9s    607535.9
	#[test]
	fn perft_test() {
		let mut board = Chess::default();

		let _nodes = perft::<ChessGame>(&mut board, 8, true);
	}
}