use std::fmt;

use crate::gui::Place;

use kudchuet::{GameOutcome, Player, ai::minimax::{Evaluation, Evaluator, Game}};
#[derive(Copy, Clone, Debug)]
pub struct Mancala {
	// First index by player.
	// Next index by pit, counting down from 6 to 1 for the pits in play.
	// Pit zero is that player's store.
	// If I wanted to be crazy bit twiddly I could put these in a pair of u64s and shift stuff around.
	pits: [[u8; 7]; 2],
	pub(crate) skipped: bool,
	// u1 of pits player index.
	pub(crate) to_move: bool,
}
impl Mancala {
	pub fn self_pit(&self, pit : usize) -> u8 {
		let player = self.to_move as usize;
		self.pits[player][pit]
	}
	pub fn opponent_pit(&self, pit : usize) -> u8 {
		let player = (!self.to_move) as usize;
		self.pits[player][pit]
	}
	pub fn bottom_pit(&self, pit : usize) -> u8 {
		self.pits[0][pit]
	}
	pub fn top_pit(&self, pit : usize) -> u8 {
		self.pits[1][pit]
	}
}
impl Default for Mancala {
	fn default() -> Mancala {
		Mancala { pits: [[0, 4, 4, 4, 4, 4, 4]; 2], skipped: false, to_move: false }
	}
}

// 1-6 means play from that pit.
// 0 means pass (because of being skipped).
pub type Move = Place;


impl Game for Mancala {
	type S = Mancala;
	type M = Move;

	fn generate_moves(board: &Mancala, moves: &mut Vec<Move>) -> GameOutcome {
		let res = Self::get_outcome(board);
		if res.is_ended()  {
			return res;
		}
		if board.skipped {
			moves.push(Place(0));
			return GameOutcome::OnGoing;
		}
		for i in 1..7 {
			if board.pits[board.to_move as usize][i] > 0 {
				moves.push(Place(i as u8) as Move);
			}
		}
		GameOutcome::OnGoing
	}

	fn apply(board: &mut Mancala, m: Move) -> Option<Mancala> {
		let mut board = board.clone();
		if board.skipped {
			board.skipped = false;
			board.to_move = !board.to_move;
			return Some(board);
		}

		// Grab the stones.
		let mut player = board.to_move as usize;
		let mut i = m.0 as usize;
		let mut stones = board.pits[player][i];
		board.pits[player][i] = 0;
		// At the beginning of each iteration, it points at the previous pit.
		while stones > 0 {
			if player == board.to_move as usize && i == 0 || player != board.to_move as usize && i == 1 {
				i = 6;
				player ^= 1;
			} else {
				i -= 1;
			}
			board.pits[player][i] += 1;
			stones -= 1;
		}

		if player == board.to_move as usize {
			if i == 0 {
				// End condition: ends in own bowl
				board.skipped = true;
			} else if board.pits[player][i] == 1 {
				// End condition: ends on own side in empty pit
				let captured = board.pits[player][i] + board.pits[player ^ 1][7 - i];
				board.pits[player][i] = 0;
				board.pits[player ^ 1][7 - i] = 0;
				board.pits[player][0] += captured;
			}
		}

		board.to_move = !board.to_move;
		Some(board)
	}

	fn get_outcome(board: &Mancala) -> GameOutcome {
		if board.pits[0][1..].iter().sum::<u8>() == 0 || board.pits[1][1..].iter().sum::<u8>() == 0
		{
			let to_move_total = board.pits[board.to_move as usize].iter().sum::<u8>();
			if to_move_total == 24 {
				GameOutcome::Draw
			} else if to_move_total > 24 {
				GameOutcome::Player(Player(board.to_move as u8))
			} else {
				GameOutcome::Player(Player((!board.to_move) as u8))
			}
		} else {
			GameOutcome::OnGoing
		}
	}
	fn get_current_player(board: &Self::S) -> Player {
		Player(board.to_move as u8)
	}
	fn get_hash(board: &Mancala) -> u64 {
		let mut hash = board.to_move as u64;
		for i in 0..7 {
			hash ^= HASHES[i].wrapping_mul(board.pits[0][i] as u64);
			hash ^= HASHES[i + 7].wrapping_mul(board.pits[1][i] as u64);
		}
		hash
	}

	fn null_move(_: &Mancala) -> Option<Move> {
		Some(Place(0))
	}

	fn notation(_: &Mancala, m: Move) -> Option<String> {
		Some(if m.0 == 0 { "skipped".to_owned() } else { format!("pit {}", m.0) })
	}

	fn table_index(m: Move) -> u16 {
		m.0 as u16
	}
	fn max_table_index() -> u16 {
		6
	}
}

impl fmt::Display for Mancala {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "+-----------------------+\n|  |")?;
		for pit in &self.pits[1][1..] {
			write!(f, "{:>2}|", pit)?;
		}
		write!(f, "  |\n+{:>2}+--+--+--+--+--+--+{:>2}+\n|  ", self.pits[1][0], self.pits[0][0])?;
		for pit in self.pits[0][1..].iter().rev() {
			write!(f, "|{:>2}", pit)?;
		}
		write!(f, "|  |\n+-----------------------+\n")
	}
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub(crate) struct EvaluatorMancala;

impl Evaluator for EvaluatorMancala {
	type G = Mancala;
	fn evaluate_for(&self, board: &Mancala, p: Player) -> Evaluation {
		board.pits[p.idx()].iter().sum::<u8>() as Evaluation - 24
	}
}
const HASHES: [u64; 14] = [
	0x73399349585d196e,
	0xe512dc15f0da3dd1,
	0x4fbc1b81c6197db2,
	0x16b5034810111a66,
	0xa9a9d0183e33c311,
	0xbb9d7bdea0dad2d6,
	0x089d9205c11ca5c7,
	0x18d9db91aa689617,
	0x1336123120681e34,
	0xc902e6c0bd6ef6bf,
	0x16985ba0916238c1,
	0x6144c3f2ab9f6dc4,
	0xf24b4842de919a02,
	0xdd6dd35ba0c150a1,
];

#[cfg(test)]
mod tests {

	use kudchuet::ai::minimax::{IterativeOptions, iterative::IterativeSearch, util::perft, Strategy, Game};

use super::super::mancala::{Mancala, EvaluatorMancala};
	// cargo test --release awale::mancala::tests::perft_test -- --nocapture
	//depth           count        time        kn/s
	//    0               1       2.9µs       344.8
	//    1               6       4.9µs      1224.5
	//    2              31       1.3µs     23846.2
	//    3             136       2.3µs     59130.4
	//    4             623     274.9µs      2266.3
	//    5            2625      75.5µs     34768.2
	//    6           11627      97.3µs    119496.4
	//    7           50724     236.2µs    214750.2
	//    8          228194     476.0µs    479399.2
	//    9         1026604       1.8ms    569829.0
	//   10         4658951       7.1ms    655959.3
	//   11        21221913      41.8ms    507600.5
	//   12        95466517     196.1ms    486762.1
	//   13       430693965     829.2ms    519399.4
	//   14      1894133972        4.6s    415214.0
	#[test]
	fn perft_test() {
		let mut board = Mancala::default();

			let _nodes = perft::<Mancala>(&mut board, 14, true);
	}

	#[test]
	fn main() {
		let mut board = Mancala::default();
		let opts = IterativeOptions::new().verbose();
		let mut strategy = IterativeSearch::new(EvaluatorMancala::default(), opts);
		strategy.set_timeout(std::time::Duration::from_secs(1));
		while !Mancala::get_outcome(&board).is_ended() {
			println!("{}", board);
			match strategy.choose_move(&board) {
				Some(m) => board = Mancala::apply(&mut board, m).unwrap(),
				None => break,
			}
		}
		println!("Winner player {:?}", board.to_move as u8 + 1);
	}
}
