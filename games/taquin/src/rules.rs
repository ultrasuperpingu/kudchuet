use std::fmt;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Taquin<const W: usize, const H: usize, const NB: usize> {
	pub(crate) content: [[u8; H]; W],
	blank_pos: u8,
	hash: u64,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Move {
	pub from: u8,
}
impl<const W: usize, const H: usize, const NB: usize> Taquin<W, H, NB> {
	pub const NB_SQUARES: usize = W * H;
	const _TEST_LENGTH: () = assert!(Self::NB_SQUARES <= 256 && W > 2 && H > 2 && W * H == NB);
	pub const SOLVED: Self = Self::new_solved();

	pub const fn new_solved() -> Self {
		let _ = Self::_TEST_LENGTH;
		let mut content = [[0; H]; W];
		let mut j = 0;
		while j < H {
			let mut i = 0;
			while i < W {
				content[i][j] = (1 + j * W + i) as u8;
				i += 1;
			}
			j += 1;
		}
		content[W - 1][H - 1] = 0; //empty square
		let blank_pos = Self::NB_SQUARES as u8 - 1;
		let mut s = Self {
			content,
			blank_pos,
			hash: 0,
		};
		s.hash = s.compute_hash();
		s
	}
	pub fn new_random() -> Self {
		let _ = Self::_TEST_LENGTH;
		let (mut blank_pos, mut content) = Self::random_content();
		while !Self::is_solvable_content(&content) {
			(blank_pos, content) = Self::random_content();
		}
		let mut s = Self {
			content,
			blank_pos,
			hash: 0,
		};
		s.hash = s.compute_hash();
		s
	}
	pub fn is_solvable(&self) -> bool {
		Self::is_solvable_content(&self.content)
	}

	fn random_content() -> (u8, [[u8; H]; W]) {
		let mut random = vec![];
		for i in 0..Self::NB_SQUARES {
			random.push(i as u8);
		}
		let mut content = [[0; H]; W];
		let mut j = 0;
		let mut blank_pos = 0;
		while j < H {
			let mut i = 0;
			while i < W {
				let index = fastrand::usize(0..random.len());
				content[i][j] = random[index];
				if content[i][j] == 0 {
					blank_pos = Self::index_from_coords(i, j);
				}
				random.remove(index);
				i += 1;
			}
			j += 1;
		}
		(blank_pos, content)
	}
}
impl<const W: usize, const H: usize, const NB: usize> Taquin<W, H, NB> {
	pub fn legal_moves(&self) -> Vec<Move> {
		let (x, y) = Self::coords_from_index(self.blank_pos);

		let mut moves = Vec::with_capacity(4);

		if x > 0 {
			moves.push(Move {
				from: Self::index_from_coords(x - 1, y),
			});
		}

		if x + 1 < W {
			moves.push(Move {
				from: Self::index_from_coords(x + 1, y),
			});
		}

		if y > 0 {
			moves.push(Move {
				from: Self::index_from_coords(x, y - 1),
			});
		}

		if y + 1 < H {
			moves.push(Move {
				from: Self::index_from_coords(x, y + 1),
			});
		}

		moves
	}
	pub fn coords_from_index(index: u8) -> (usize, usize) {
		let idx = index as usize;
		(idx % W, idx / W)
	}
	pub fn index_from_coords(x: usize, y: usize) -> u8 {
		x as u8 + (y * W) as u8
	}
	pub fn play_unchecked(&mut self, m: Move) {
		let (bx, by) = Self::coords_from_index(self.blank_pos);
		let (from_x, from_y) = Self::coords_from_index(m.from);
		self.update_hash_move(&m);
		self.content[bx][by] = self.content[from_x][from_y];
		self.content[from_x][from_y] = 0;
		self.blank_pos = m.from;
	}
}
impl<const W: usize, const H: usize, const NB: usize> Taquin<W, H, NB> {
	fn is_solvable_content(content: &[[u8; H]; W]) -> bool {
		let v = Self::flatten(content);
		let inv = Self::inversions(&v);
		let blank_row = Self::blank_row_from_bottom(content);

		if W % 2 == 1 {
			inv % 2 == 0
		} else {
			(inv + blank_row) % 2 == 1
		}
	}
	fn flatten(content: &[[u8; H]; W]) -> Vec<u8> {
		let mut v = Vec::with_capacity(W * H);
		for j in 0..H {
			for i in 0..W {
				let x = content[i][j];
				if x != 0 {
					v.push(x);
				}
			}
		}
		v
	}
	fn inversions(v: &[u8]) -> usize {
		let mut inv = 0;
		for i in 0..v.len() {
			for j in i + 1..v.len() {
				if v[i] > v[j] {
					inv += 1;
				}
			}
		}
		inv
	}
	fn blank_row_from_bottom(content: &[[u8; H]; W]) -> usize {
		for j in 0..H {
			for i in 0..W {
				if content[i][j] == 0 {
					return H - j;
				}
			}
		}
		unreachable!()
	}
}
impl<const W: usize, const H: usize, const NB: usize> Default for Taquin<W, H, NB> {
	fn default() -> Self {
		Self::new_random()
	}
}
impl<const W: usize, const H: usize, const NB: usize> Taquin<W, H, NB> {
	pub fn manhattan_distance(&self) -> usize {
		let mut sum = 0;

		for j in 0..H {
			for i in 0..W {
				let v = self.content[i][j];
				if v == 0 {
					continue;
				}

				// goal position (1-based tiles)
				let goal = v as usize - 1;
				let goal_x = goal % W;
				let goal_y = goal / W;

				let dx = i.abs_diff(goal_x);
				let dy = j.abs_diff(goal_y);

				sum += dx + dy;
			}
		}

		sum
	}
	/*fn linear_conflict_row(&self, y: usize) -> usize {
		let mut tiles = vec![];

		for x in 0..W {
			let v = self.content[x][y];
			if v == 0 {
				continue;
			}

			let goal = (v as usize) - 1;
			let goal_x = goal % W;
			let goal_y = goal / W;

			if goal_y == y {
				tiles.push((x, goal_x));
			}
		}

		let mut conflicts = 0;

		for i in 0..tiles.len() {
			for j in i + 1..tiles.len() {
				if tiles[i].1 > tiles[j].1 {
					conflicts += 1;
				}
			}
		}

		conflicts
	}*/
	pub fn linear_conflict_row(&self, y: usize) -> usize {
		let mut goals = [0u8; W];
		let mut n = 0;

		for x in 0..W {
			let v = self.content[x][y];
			if v == 0 {
				continue;
			}

			let goal = (v as usize) - 1;
			let goal_y = goal / W;

			if goal_y == y {
				goals[n] = goal as u8 % W as u8;
				n += 1;
			}
		}

		let mut conflicts = 0;

		for i in 0..n {
			for j in i + 1..n {
				if goals[i] > goals[j] {
					conflicts += 1;
				}
			}
		}

		conflicts
	}
	fn linear_conflict_col(&self, x: usize) -> usize {
		let mut tiles = vec![];

		for y in 0..H {
			let v = self.content[x][y];
			if v == 0 {
				continue;
			}

			let goal = (v as usize) - 1;
			let goal_x = goal % W;
			let goal_y = goal / W;

			// seulement les tuiles qui doivent être dans cette colonne
			if goal_x == x {
				tiles.push((y, goal_y));
			}
		}

		let mut conflicts = 0;

		for i in 0..tiles.len() {
			for j in i + 1..tiles.len() {
				// inversion verticale dans la colonne
				if tiles[i].1 > tiles[j].1 {
					conflicts += 1;
				}
			}
		}

		conflicts
	}
	pub fn manhattan_with_linear_conflict(&self) -> usize {
		let manhattan = self.manhattan_distance();

		let mut conflicts = 0;

		for y in 0..H {
			conflicts += self.linear_conflict_row(y);
		}

		for x in 0..W {
			conflicts += self.linear_conflict_col(x);
		}

		manhattan + 2 * conflicts
	}
}

impl<const W: usize, const H: usize, const NB: usize> fmt::Display for Taquin<W, H, NB> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for j in 0..H {
			for i in 0..W {
				let v = self.content[i][j];

				if v == 0 {
					write!(f, "   .")?;
				} else {
					write!(f, "{:4}", v)?;
				}
			}

			if j + 1 < H {
				writeln!(f)?;
			}
		}

		Ok(())
	}
}

impl<const W: usize, const H: usize, const NB: usize> Taquin<W, H, NB> {
	pub const ZOBRIST_KEYS: Zobrist<W, H, NB> = Zobrist::new(0x15A4CDE);
	#[inline(always)]
	pub fn get_hash(&self) -> u64 {
		self.hash
	}
	pub const fn compute_hash(&self) -> u64 {
		let mut h = 0u64;

		let mut i = 0;
		while i < W {
			let mut j = 0;
			while j < H {
				let tile = self.content[i][j] as usize;
				h ^= Self::ZOBRIST_KEYS.content[i][j][tile as usize];
				j += 1;
			}
			i += 1;
		}

		h
	}

	fn update_hash_move(&mut self, m: &Move) {
		let (bx, by) = Self::coords_from_index(self.blank_pos);
		let (fx, fy) = Self::coords_from_index(m.from);

		let tile = self.content[fx][fy] as usize;

		// remove tile from old position
		self.hash ^= Self::ZOBRIST_KEYS.content[fx][fy][tile as usize];
		self.hash ^= Self::ZOBRIST_KEYS.content[fx][fy][0];

		// place tile in blank position
		self.hash ^= Self::ZOBRIST_KEYS.content[bx][by][0];
		self.hash ^= Self::ZOBRIST_KEYS.content[bx][by][tile as usize];

		// update blank implicitly
	}
}
pub struct Zobrist<const W: usize, const H: usize, const NB: usize> {
	pub content: [[[u64; NB]; H]; W],
}
impl<const W: usize, const H: usize, const NB: usize> Zobrist<W, H, NB> {
	pub const fn new(seed: u64) -> Self {
		let mut rng = kudchuet::utils::Rng::from_seed(seed);
		let mut content = [[[0; NB]; H]; W];

		let mut i = 0;
		while i < W {
			let mut j = 0;
			while j < H {
				let mut k = 0;
				while k < H * W {
					content[i][j][k] = rng.u64();
					k += 1;
				}
				j += 1;
			}
			i += 1;
		}

		Self { content }
	}
}
#[cfg(test)]
mod tests {
	use crate::{astar::AStar, rules::Taquin};
	use kudchuet::ai::move_search::Strategy;
	use kudchuet::gui::GUIGame as _;

	#[test]
	fn display() {
		let taquin = Taquin::<3, 3, 9>::new_random();
		println!("{}\n\n", taquin);
		let taquin = Taquin::<12, 5, 60>::new_solved();
		println!("{}\n\n", taquin);
		let taquin = Taquin::<16, 16, 256>::new_random();
		println!("{}\n\n", taquin);
	}
	#[test]
	fn test_legal_moves() {
		let mut state = Taquin::<3, 3, 9>::new_random();
		println!("{}", state);
		let mut moves = state.legal_moves();
		println!("{:?}", moves);
		let mut i = 0;
		while i < 500000 && !moves.is_empty() {
			let mv = fastrand::choice(&moves).unwrap();

			state.play_unchecked(*mv);
			assert_eq!(state.compute_hash(), state.hash);
			moves = state.legal_moves();
			i += 1;
		}

		println!("{}", i);
		println!("{:?}", state.result());
		println!("{}", state);
		println!("{:?}", moves);
	}
	#[test]
	fn test_play() {
		let solved = Taquin::<3, 4, 12>::new_solved();
		println!("{}", solved.is_solvable());
		const W: usize = 4;
		const H: usize = 4;
		const NB: usize = W * H;
		let mut state = Taquin::<W, H, NB>::new_random();
		println!("{}", state);
		let mut i = 0;
		let mut strategy = AStar::<Taquin<W, H, NB>>::default();
		//strategy.set_depth_or_timeout(12, Duration::from_secs(1));
		while i < 500000 {
			let mv = strategy.choose_move(&mut state);
			assert_eq!(state.compute_hash(), state.get_hash());
			if let Some(mv) = mv {
				state.play(mv);
				println!("{}", state);
			} else {
				break;
			}
			i += 1;
		}

		println!("{}", i);
		println!("{:?}", state.result());
	}
}
