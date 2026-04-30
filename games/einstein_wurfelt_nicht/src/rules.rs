use kudchuet::GameOutcome;
use kudchuet::Player;

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct EinsteinWurfeltNicht {
	pub(crate) red: [u8; 6],
	pub(crate) blue: [u8; 6],
	pub(crate) is_red: bool,
	pub(crate) dice: Option<u8>,
}
static NEIGHBORS_BLUE: [[u8; 3]; 25] = [
	[5, 25, 25],  // 0
	[0, 5, 6],    // 1
	[1, 6, 7],    // 2
	[2, 7, 8],    // 3
	[3, 8, 9],    // 4
	[10, 25, 25], // 5
	[5, 10, 11],  // 6
	[6, 11, 12],  // 7
	[7, 12, 13],  // 8
	[8, 13, 14],  // 9
	[15, 25, 25], // 10
	[10, 15, 16], // 11
	[11, 16, 17], // 12
	[12, 17, 18], // 13
	[13, 18, 19], // 14
	[20, 25, 25], // 15
	[15, 20, 21], // 16
	[16, 21, 22], // 17
	[17, 22, 23], // 18
	[18, 23, 24], // 19
	[25, 25, 25], // 20
	[20, 25, 25], // 21
	[21, 25, 25], // 22
	[22, 25, 25], // 23
	[23, 25, 25], // 24
];

static NEIGHBORS_RED: [[u8; 3]; 25] = [
	[1, 25, 25],  // 0
	[2, 25, 25],  // 1
	[3, 25, 25],  // 2
	[4, 25, 25],  // 3
	[25, 25, 25], // 4
	[0, 1, 6],    // 5
	[1, 2, 7],    // 6
	[2, 3, 8],    // 7
	[3, 4, 9],    // 8
	[4, 25, 25],  // 9
	[5, 6, 11],   // 10
	[6, 7, 12],   // 11
	[7, 8, 13],   // 12
	[8, 9, 14],   // 13
	[9, 25, 25],  // 14
	[10, 11, 16], // 15
	[11, 12, 17], // 16
	[12, 13, 18], // 17
	[13, 14, 19], // 18
	[14, 25, 25], // 19
	[15, 16, 21], // 20
	[16, 17, 22], // 21
	[17, 18, 23], // 22
	[18, 19, 24], // 23
	[19, 25, 25], // 24
];

impl Default for EinsteinWurfeltNicht {
	fn default() -> Self {
		Self {
			red: [20, 21, 22, 15, 16, 10],
			blue: [2, 3, 4, 8, 9, 14],
			is_red: false,
			dice: None,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MovePlay {
	Dice(u8),
	Move(Move),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
	pub from: u8,
	pub to: u8,
}
impl EinsteinWurfeltNicht {
	pub fn legal_moves(&self) -> Vec<MovePlay> {
		let mut moves = vec![];
		if self.dice.is_none() {
			moves = vec![
				MovePlay::Dice(1),
				MovePlay::Dice(2),
				MovePlay::Dice(3),
				MovePlay::Dice(4),
				MovePlay::Dice(5),
				MovePlay::Dice(6),
			];
		} else {
			let d = self.dice.unwrap();
			let array = if self.is_red { &self.red } else { &self.blue };
			let neighbors = if self.is_red {
				&NEIGHBORS_RED
			} else {
				&NEIGHBORS_BLUE
			};
			let index = array[d as usize - 1];
			if index == 25 {
				for i in (0..d - 1).rev() {
					let val = array[i as usize];
					if val < 25 {
						for e in neighbors[val as usize] {
							if e < 25 {
								moves.push(MovePlay::Move(Move { from: val, to: e }));
							}
						}
						break;
					}
				}

				for i in d..6 {
					let val = array[i as usize];
					if val < 25 {
						for e in neighbors[val as usize] {
							if e < 25 {
								moves.push(MovePlay::Move(Move { from: val, to: e }));
							}
						}
						break;
					}
				}
			} else {
				for e in neighbors[index as usize] {
					if e < 25 {
						moves.push(MovePlay::Move(Move { from: index, to: e }));
					}
				}
			}
		}
		moves
	}
	pub fn roll_dice(&mut self) {
		let d = fastrand::u8(1..=6);
		println!("{}", d);
		self.dice = Some(d)
	}
	pub fn play_unchecked(&mut self, m: MovePlay) {
		match m {
			MovePlay::Dice(d) => self.dice = Some(d),
			MovePlay::Move(m) => {
				if self.is_red {
					for e in self.red.iter_mut() {
						if e == &m.to {
							*e = 25;
						}
						if e == &m.from {
							*e = m.to;
						}
					}
					for e in self.blue.iter_mut() {
						if e == &m.to {
							*e = 25;
							break;
						}
					}
				} else {
					for e in self.blue.iter_mut() {
						if e == &m.to {
							*e = 25;
						}
						if e == &m.from {
							*e = m.to;
						}
					}
					for e in self.red.iter_mut() {
						if e == &m.to {
							*e = 25;
							break;
						}
					}
				}
			}
		}
		self.dice = None;
		self.is_red = !self.is_red;
	}
	pub fn result(&self) -> GameOutcome {
		if self.blue.iter().any(|e| e == &20) || self.red.iter().all(|e| e == &25) {
			return GameOutcome::Player(Player::PLAYER1);
		}
		if self.red.iter().any(|e| e == &4) || self.blue.iter().all(|e| e == &25) {
			return GameOutcome::Player(Player::PLAYER2);
		}
		GameOutcome::OnGoing
	}
}
