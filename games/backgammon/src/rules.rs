use std::collections::HashSet;
use arrayvec::ArrayVec;
use kudchuet::common::Player;
use kudchuet::utils;
use std::fmt;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct Backgammon {
	pub board: [i8; 24],
	pub on_bar: [u8; 2],
	pub outside: [u8; 2],
	pub current_player: Player,
	pub dice: ArrayVec<u8, 4>,
	pub rng : utils::Rng,
	pub hash: u64,
}
impl Hash for Backgammon {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.board.hash(state);
		self.on_bar.hash(state);
		self.outside.hash(state);
		self.current_player.hash(state);
		self.dice.hash(state);
	}
}

pub struct Zobrist {
	pub board: [[[u64;15];2];24],
	pub outside: [[u64;15];2],
	pub on_bar: [[u64;15];2],
	pub turn: u64,
}

impl Zobrist {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);
		let mut board=[[[0;15];2];24];
		let mut outside=[[0;15];2];
		let mut on_bar=[[0;15];2];
		let mut i = 0;
		while i < board.len() {
			let mut j = 0;
			while j < board[i].len() {
				let mut k = 0;
				while k < board[i][j].len() {
					board[i][j][k] = rng.u64();
					k+=1;
				}
				j+=1;
			}
			i+=1;
		}
		let mut i = 0;
		while i < outside.len() {
			let mut j = 0;
			while j < outside[i].len() {
				outside[i][j] = rng.u64();
				on_bar[i][j] = rng.u64();
				j+=1;
			}
			i+=1;
		}
		Self {
			board,
			outside,
			on_bar,
			turn: rng.u64(),
		}
	}
}
impl Backgammon {
	const ZOBRIST: Zobrist = Zobrist::new(1254);
	fn compute_hash(&self) -> u64 {
		let mut h = 0u64;
		for (i, &count) in self.board.iter().enumerate() {
			if count != 0 {
				let sign = if count > 0 { 0 } else { 1 };
				let abs_count = count.unsigned_abs() as usize;
				for n in 0..abs_count {
					//println!("board[{}][{}][{}]",i, sign, n);
					h ^= Self::ZOBRIST.board[i][sign][n];
				}
			}
		}
		for (i, &count) in self.outside.iter().enumerate() {
			for n in 0..count as usize {
				//println!("outside[{}][{}]",i,n);
				h ^= Self::ZOBRIST.outside[i][n];
			}
		}
		for (i, &count) in self.on_bar.iter().enumerate() {
			for n in 0..count as usize {
				//println!("on_bar[{}][{}]",i,n);
				h ^= Self::ZOBRIST.on_bar[i][n];
			}
		}
		if self.current_player == Player::PLAYER2 {
			h ^= Self::ZOBRIST.turn;
		}
		h
	}
	pub fn update_turn_hash(&mut self) {
		self.hash ^= Self::ZOBRIST.turn;
	}
	pub fn update_hash_single_move(&mut self, m: &SingleMove, piece_sign: usize) {
		let opp_sign = 1 - piece_sign;

		// --- FROM ---
		match m.from {
			P1_BAR => {
				self.hash ^= Self::ZOBRIST.on_bar[0][(self.on_bar[0]-1) as usize];
			}
			P2_BAR => {
				self.hash ^= Self::ZOBRIST.on_bar[1][(self.on_bar[1]-1) as usize];
			}
			f if f < 24 => {
				let abs_count_from = self.board[f as usize].unsigned_abs() as usize;
				if abs_count_from > 0 {
					self.hash ^= Self::ZOBRIST.board[f as usize][piece_sign][abs_count_from-1];
				}
			}
			_ => {}
		}

		// --- CAPTURE ---
		if m.captured {
			if m.to < 24 {
				//println!("update board[{}][{}][{}], content: {}", m.to, opp_sign, 0, self.board[m.to as usize]);
				self.hash ^= Self::ZOBRIST.board[m.to as usize][opp_sign][0];
				//println!("update on_bar[{}][{}]", opp_sign, self.on_bar[opp_sign]);
				self.hash ^= Self::ZOBRIST.on_bar[opp_sign][self.on_bar[opp_sign] as usize];
			}
		}

		// --- TO ---
		match m.to {
			P1_OUT => {
				self.hash ^= Self::ZOBRIST.outside[0][self.outside[0] as usize];
			}
			P2_OUT => {
				self.hash ^= Self::ZOBRIST.outside[1][self.outside[1] as usize];
			}
			t if t < 24 => {
				let count = if m.captured { 0 } else { self.board[t as usize].unsigned_abs() as usize };
				self.hash ^= Self::ZOBRIST.board[t as usize][piece_sign][count];
			}
			_ => {}
		}
	}
}

impl PartialEq for Backgammon {
	fn eq(&self, other: &Self) -> bool {
		self.board == other.board &&
			self.on_bar == other.on_bar &&
			self.outside == other.outside &&
			self.current_player == other.current_player &&
			self.dice == other.dice
	}
}
impl Eq for Backgammon {}
impl Default for Backgammon {
	fn default() -> Self {
		let mut board = [0i8; 24];

		// Player 1 (positive)
		board[0]  = 2;
		board[11] = 5;
		board[16] = 3;
		board[18] = 5;

		// Player 2 (negative)
		board[23] = -2;
		board[12] = -5;
		board[7]  = -3;
		board[5]  = -5;

		let mut s= Self {
			board,
			on_bar: [0, 0],
			outside: [0, 0],
			current_player: Player::PLAYER1,
			dice: ArrayVec::new(),
			rng: utils::Rng::new(),
			hash: 0,
		};
		s.hash = s.compute_hash();
		s
	}
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Move {
	Dice(u8, u8),
	Player(PlayerMove)
}
impl Move {
	pub fn to_player_move(&mut self) -> Option<PlayerMove> {
		match self {
			Self::Player(m) => Some(*m),
			Self::Dice(_,_) => None
		}
	}
	pub fn is_random(&self) -> bool {
		match self {
			Self::Player(_) => false,
			Self::Dice(_,_) => true
		}
	}
}
pub const P1_BAR: u8 = u8::MAX;
pub const P2_BAR: u8 = u8::MAX-1;
pub const P1_OUT: u8 = u8::MAX-2;
pub const P2_OUT: u8 = u8::MAX-3;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerMove {
	pub moves: [SingleMove; 4],
	pub len: u8,
}
impl Default for PlayerMove {
	fn default() -> Self {
		Self::new()
	}
}
impl PlayerMove {
	pub fn new() -> Self {
		Self {
			moves: [SingleMove::default(); 4],
			len: 0,
		}
	}

	pub fn push(&mut self, m: SingleMove) {
		self.moves[self.len as usize] = m;
		self.len += 1;
	}
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct SingleMove {
	pub from: u8,
	pub to: u8,
	pub captured: bool
}
impl SingleMove {
	pub fn new(from: u8, to: u8, capture: bool) -> Self {
		Self {
			from,
			to,
			captured: capture,
		}
	}
}
impl Backgammon {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn roll_dice(&mut self) {
		let d1 = self.rng.range(1, 7);
		let d2 = self.rng.range(1, 7);

		if d1 == d2 {
			self.dice = ArrayVec::from([d1, d1, d1, d1]);
		} else {
			self.dice = ArrayVec::new();
			self.dice.extend([d1, d2]);
		}
		//println!("{:?}", self.dice);
	}

	pub fn is_game_over(&self) -> bool {
		self.outside[0] == 15 || self.outside[1] == 15
	}

	pub fn winner(&self) -> Option<Player> {
		if self.outside[0] == 15 {
			Some(Player::PLAYER1)
		} else if self.outside[1] == 15 {
			Some(Player::PLAYER2)
		} else {
			None
		}
	}

	pub fn current_player(&self) -> Player {
		if self.dice.is_empty() {
			Player::RandomMove
		} else {
			self.current_player
		}
	}
	fn current_sign(&self) -> i8 {
		if self.current_player == Player::PLAYER1 { 1 } else { -1 }
	}

	fn entry_point(&self, die: u8) -> u8 {
		match self.current_player {
			Player::PLAYER1 => die - 1,
			Player::PLAYER2 => 24 - die,
			_ => unreachable!(),
		}
	}
	fn can_land(&self, pos: u8) -> Option<bool> {
		let target = self.board[pos as usize];
		let sign = self.current_sign();

		if !(target.abs() >= 2 && target.signum() != sign) {
			Some(target.abs() == 1 && target.signum() != sign)
		} else {
			None
		}
	}
	fn can_bear_off(&self) -> bool {
		let sign = self.current_sign();

		for i in 0..24 {
			if self.board[i] * sign > 0 {
				if self.current_player == Player::PLAYER1 && i < 18 {
					return false;
				}
				if self.current_player == Player::PLAYER2 && i > 5 {
					return false;
				}
			}
		}
		true
	}

	pub fn legal_moves(&self) -> Vec<Move> {
		let mut results = Vec::new();

		if self.dice.is_empty() {
			//println!("empty dices: {:?}", self.dice);
			for d1 in 0..6 {
				for d2 in 0..6 {
					results.push(Move::Dice(d1+1, d2+1));
				}
			}
			return results;
		}
		let mut player_res = HashSet::new();
		self.gen_moves_recursive(&self.clone(), &self.dice.clone(), &PlayerMove::new(), &mut player_res);

		let max_len = player_res.iter().map(|m| m.len).max().unwrap_or(0);

		//println!("max_len: {}", max_len);
		let res: Vec<PlayerMove> = player_res.into_iter()
			.filter(|m| m.len == max_len)
			.collect();
		// TODO: max dice
		//if max_len == 1 {
		//	let max_val = player_res.iter().map(|m| m.moves[0].1.abs_diff(m.moves[0].1)).max().unwrap_or(0);

		//}
		//TODO: higher bearing off
		res.into_iter().map(Move::Player).collect()
	}
	fn gen_moves_recursive(
		&self,
		game: &Backgammon,
		dice: &ArrayVec<u8, 4>,
		current: &PlayerMove,
		results: &mut HashSet<PlayerMove>,
	) {
		let mut played_any = false;

		for (i, &die) in dice.iter().enumerate() {
			let possible = game.single_die_moves(die);

			for m in possible {
				let mut next_game = game.clone();

				next_game.apply_single_move(m.from, m.to);
				let mut next_dice = dice.clone();
				next_dice.remove(i);

				let mut next_moves = current.clone();
				next_moves.push(m);

				self.gen_moves_recursive(&next_game, &next_dice, &next_moves, results);

				played_any = true;
			}
		}

		if !played_any {
			results.insert(*current);
		}
	}

	fn single_die_moves(&self, die: u8) -> Vec<SingleMove> {
		let mut moves = Vec::new();

		let player = self.current_player;
		let dir: i32 = if player == Player::PLAYER1 { 1 } else { -1 };
		let piece = if player == Player::PLAYER1 { 1 } else { -1 };

		// priority: out of bar
		if self.on_bar[player.idx()] > 0 {
			let entry = self.entry_point(die);

			if let Some(capture) = self.can_land(entry) {
				moves.push(SingleMove::new(if player == Player::PLAYER1 {P1_BAR} else {P2_BAR}, entry, capture));
			}

			return moves;
		}

		for from in 0..24 {
			if self.board[from as usize] * piece > 0 {
				let to = from as i32 + dir * die as i32;

				if let Some(captured) = self.is_legal_target(to) {
					if to < 0 {
						moves.push(SingleMove::new(from, P2_OUT, captured));
					} else if to >= 24 {
						moves.push(SingleMove::new(from, P1_OUT, captured));
					} else {
						moves.push(SingleMove::new(from, to as u8, captured));
					}
				}
			}
		}
		moves
	}

	fn is_legal_target(&self, to: i32) -> Option<bool> {
		if to < 0 || to >= 24 {
			return if self.can_bear_off() {
				Some(false)
			} else {
				None
			};
		}

		let target = self.board[to as usize];

		if target.abs() >= 2 && target.signum() != self.current_sign() {
			return None;
		}

		Some(target.abs() == 1 && target.signum() != self.current_sign())
	}
	pub(crate) fn apply_single_move(&mut self, from: u8, to: u8) {
		let player = self.current_player;
		let piece = if player == Player::PLAYER1 { 1 } else { -1 };
		
		// Remove pawn
		if from == P1_BAR {
			debug_assert!(player == Player::PLAYER1);
			self.on_bar[0] -= 1;
		} else if from == P2_BAR {
			debug_assert!(player == Player::PLAYER2);
			self.on_bar[1] -= 1;
		} else {
			self.board[from as usize] -= piece;
		}

		if to == P1_OUT {
			debug_assert!(player == Player::PLAYER1);
			self.outside[0] += 1;
			return;
		}
		else if to == P2_OUT {
			debug_assert!(player == Player::PLAYER2);
			self.outside[1] += 1;
			return;
		}

		let target = self.board[to as usize];

		// capture
		if target == -piece {
			self.board[to as usize] = 0;
			self.on_bar[player.opponent().idx()] += 1;
		}

		self.board[to as usize] += piece;

	}
	pub fn play_unchecked(&mut self, mv: Move) -> bool {
		match mv {
			Move::Dice(d1, d2) => { 
				if d1 == d2 {
					self.dice = ArrayVec::from([d1, d1, d1, d1]);
				} else {
					self.dice = ArrayVec::new();
					self.dice.extend([d1, d2]);
				}
			}
			Move::Player(mv) => {
				for i in 0..mv.len as usize {
					let m = &mv.moves[i];
					
					self.update_hash_single_move(m, self.current_player.idx());
					self.apply_single_move(m.from, m.to);
					//println!("{:?}", m);
					debug_assert_eq!(self.hash,self.compute_hash());
				}

				self.dice.clear();
				self.update_turn_hash();
				self.switch_player();
			}
		}
				
		true
	}
	pub fn undo_unchecked(&mut self, mv: Move) -> bool {
		self.switch_player();
		match mv {
			Move::Dice(_d1, _d2) => { 
				self.dice.clear();
			}
			Move::Player(mv) => {
				for i in (0..mv.len as usize).rev() {
					let m = mv.moves[i];
					
					self.undo_single_move(&m);
				}
			}
		}
		true
	}
	fn undo_single_move(&mut self, mv: &SingleMove) {
		let player = self.current_player;
		let piece = if player == Player::PLAYER1 { 1 } else { -1 };

		if mv.to == P1_OUT {
			self.outside[0] -= 1;
		} else if mv.to == P2_OUT {
			self.outside[1] -= 1;
		} else {
			self.board[mv.to as usize] -= piece;
		}

		if mv.captured {
			self.on_bar[player.opponent().idx()] -= 1;
			self.board[mv.to as usize] = -piece;
		}

		if mv.from == P1_BAR {
			self.on_bar[0] += 1;
		} else if mv.from == P2_BAR {
			self.on_bar[1] += 1;
		} else {
			self.board[mv.from as usize] += piece;
		}
	}
	
	pub fn switch_player(&mut self) {
		self.current_player = self.current_player.opponent();
		self.dice.clear();
	}
}

impl fmt::Display for Backgammon {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for i in 12..24 {
			let piece = self.board[i];
			match piece {
				p if p > 0 => write!(f, "{:>2} ", format!("W{}", p))?,
				p if p < 0 => write!(f, "{:>2} ", format!("B{}", -p))?,
				_ => write!(f, " . ")?,
			}
		}
		writeln!(f)?;
		for i in (0..12).rev() {
			let piece = self.board[i];
			match piece {
				p if p > 0 => write!(f, "{:>2} ", format!("W{}", p))?,
				p if p < 0 => write!(f, "{:>2} ", format!("B{}", -p))?,
				_ => write!(f, " . ")?,
			}
		}
		writeln!(f)?;
		write!(f, "Current player: {}, ", self.current_player)?;
		write!(f, "Player 1 out: {}, ", self.outside[0])?;
		write!(f, "Player 2 out: {}, ", self.outside[1])?;
		write!(f, "Player 1 bar: {}, ", self.on_bar[0])?;
		write!(f, "Player 2 bar: {}", self.on_bar[1])?;
		if self.dice.len() >= 2 {
			write!(f, ", Dice: {}-{}", self.dice[0], self.dice[1])?;
		}
		Ok(())
	}
}
#[cfg(test)]
mod tests {
	use super::Backgammon;

	#[test]
	fn test_simple() {
		let mut game = Backgammon::new();
		println!("{}", game);
		game.roll_dice();
		println!("{}", game);
		let moves = game.legal_moves();
		println!("{}", moves.len());
		for m in moves {
			let mut g = game.clone();
			g.play_unchecked(m);
			println!("{:?} =>\n{}", m, g);
			assert_eq!(g.hash, g.compute_hash());
		}
	}
	#[test]
	fn test_random_game_until_end() {
		let mut game = Backgammon::new();
		println!("{}", game);
		let mut rng = kudchuet::utils::Rng::new();

		let mut turn = 0;
		let max_turns = 10_000; // max number of turns

		while !game.is_game_over() && turn < max_turns {
			if game.dice.is_empty() {
				game.roll_dice();
				println!("{:?}", game.dice);
			}

			let moves = game.legal_moves();

			if moves.is_empty() {
				// no move → pass
				game.switch_player();
				turn += 1;
				continue;
			}

			let mv = moves[rng.range(0, moves.len())];

			let ok = game.play_unchecked(mv);
			assert!(ok, "apply_move failed on a legal move");
			println!("{:?}", mv);
			assert_eq!(game.hash, game.compute_hash());

			turn += 1;
		}

		println!("Game finished in {} turns", turn);
		println!("Final state:\n{}", game);

		assert!(
			game.is_game_over(),
			"Game did not finish within {} turns",
			max_turns
		);

		let winner = game.winner();
		assert!(winner.is_some(), "Game ended without a winner");
	}
}