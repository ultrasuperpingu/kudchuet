use crate::{
	playing_cards32::{Color, PlayingCard32},
	unordered_card_sets32::UnorderedCardSet32,
};
#[derive(Clone, Debug, Hash)]
pub struct Manille {
	pub players: [UnorderedCardSet32; 4],
	pub scores: [u8; 2],
	pub trump_card: PlayingCard32,
	pub ply: [Option<PlayingCard32>; 4],
	pub taking_team: Option<u8>,
	pub on_turn: u8,
}
impl Default for Manille {
	fn default() -> Self {
		let mut deck = UnorderedCardSet32::ALL;
		let player1 = UnorderedCardSet32::draw_random(8, &mut deck).unwrap();
		Self {
			players: [
				player1,
				UnorderedCardSet32::draw_random(8, &mut deck).unwrap(),
				UnorderedCardSet32::draw_random(8, &mut deck).unwrap(),
				UnorderedCardSet32::draw_random(8, &mut deck).unwrap(),
			],
			trump_card: fastrand::choice(player1).unwrap(),
			scores: Default::default(),
			ply: Default::default(),
			taking_team: None,
			on_turn: 0,
		}
	}
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
	Bid(bool),
	Play(PlayingCard32),
}

impl PlayingCard32 {
	pub fn rank(&self) -> u8 {
		let v = *self as u8;

		match v % 8 {
			0 => 7,  //7
			1 => 8,  //8
			2 => 9,  //9
			3 => 14, //10
			4 => 10, //J
			5 => 11, //Q
			6 => 12, //K
			7 => 13, //Ace
			_ => unreachable!(),
		}
	}
	pub fn value(&self) -> u8 {
		let v = *self as u8;

		match v % 8 {
			0 => 0, //7
			1 => 0, //8
			2 => 0, //9
			3 => 5, //10
			4 => 1, //J
			5 => 2, //Q
			6 => 3, //K
			7 => 4, //Ace
			_ => unreachable!(),
		}
	}
}
impl Manille {
	pub fn same_team(p1: u8, p2: u8) -> bool {
		p1 % 2 == p2 % 2
	}
	pub fn legal_moves_inplace(&self, moves: &mut Vec<Move>) {
		if self.taking_team.is_none() {
			moves.push(Move::Bid(false));
			moves.push(Move::Bid(true));
			return;
		}
		let status = self.get_ply_status();
		let set = match status {
			PlyStatus::FirstToPlay => self.get_current_player_cards(),
			PlyStatus::ColorPartnerLeads(color) => {
				let cards: UnorderedCardSet32 = self.get_current_player_cards();
				let mut valid = cards.of_color(color);
				//Don't need to cut
				//if valid.is_empty() {
				//	valid = cards & UnorderedCardSet32::by_color(self.trump_card.color());
				//}
				if valid.is_empty() {
					valid = cards;
				}
				valid
			}
			PlyStatus::ColorAdverseLeads(color, best_rank) => {
				let cards: UnorderedCardSet32 = self.get_current_player_cards();
				let mut valid = cards.of_color(color);
				valid = valid.iter().filter(|c| c.rank() > best_rank).collect();
				if valid.is_empty() {
					valid = cards.of_color(self.trump_card.color());
				}
				if valid.is_empty() {
					valid = cards;
				}
				valid
			}
			PlyStatus::ColorCuttedPartner(color) => {
				let cards: UnorderedCardSet32 = self.get_current_player_cards();
				let mut valid = cards.of_color(color);
				if valid.is_empty() {
					valid = cards;
				}
				valid
			}
			PlyStatus::ColorCuttedAdverse(color, best_rank) => {
				let cards: UnorderedCardSet32 = self.get_current_player_cards();
				let mut valid = cards.of_color(color);
				if valid.is_empty() {
					valid = cards.of_color(self.trump_card.color());
					valid = valid.iter().filter(|c| c.rank() > best_rank).collect();
				}
				if valid.is_empty() {
					valid = cards;
				}
				valid
			}
		};
		for c in set {
			moves.push(Move::Play(c));
		}
	}
	fn get_ply_status(&self) -> PlyStatus {
		let mut status = PlyStatus::FirstToPlay;
		let mut color = None;
		let mut color_is_trump = false;
		let mut current_leader = 0;
		let mut leader_rank = 0;
		let mut leader_cutted = false;

		for (i, c) in self.ply.iter().enumerate() {
			if let Some(c) = c {
				if i == 0 {
					color = Some(c.color());
					color_is_trump = c.color() == self.trump_card.color();
					current_leader = self.get_player_from_ply_index(i);
					leader_rank = c.rank();
				} else {
					if !color_is_trump && c.color() == self.trump_card.color() {
						if leader_cutted {
							if leader_rank < c.rank() {
								leader_rank = c.rank();
								current_leader = self.get_player_from_ply_index(i);
							}
						} else {
							leader_cutted = true;
							leader_rank = c.rank();
							current_leader = self.get_player_from_ply_index(i);
						}
					} else {
						if !leader_cutted {
							if c.color() == color.unwrap() && leader_rank < c.rank() {
								leader_rank = c.rank();
								current_leader = self.get_player_from_ply_index(i);
							}
						}
					}
				}
			} else {
				break;
			}
		}
		if let Some(color) = color {
			let same_team = Manille::same_team(current_leader, self.on_turn);
			if leader_cutted {
				if same_team {
					status = PlyStatus::ColorCuttedPartner(color);
				} else {
					status = PlyStatus::ColorCuttedAdverse(color, leader_rank);
				}
			} else {
				if same_team {
					status = PlyStatus::ColorPartnerLeads(color);
				} else {
					status = PlyStatus::ColorAdverseLeads(color, leader_rank);
				}
			}
		}
		status
	}
	fn get_player_from_ply_index(&self, idx: usize) -> u8 {
		let ply_len = self.ply.iter().take_while(|c| c.is_some()).count() as u8;

		(idx as u8 + self.on_turn + 4 - ply_len) % 4
	}
	pub fn get_current_player_cards(&self) -> UnorderedCardSet32 {
		self.players[self.on_turn as usize]
	}
	fn get_current_player_cards_mut(&mut self) -> &mut UnorderedCardSet32 {
		&mut self.players[self.on_turn as usize]
	}
	pub fn play(&mut self, m: Move) {
		match m {
			Move::Bid(b) => {
				if b {
					self.taking_team = Some(self.on_turn % 2);
					self.on_turn = 0;
					return;
				}
				self.on_turn += 1;
			}
			Move::Play(card) => {
				self.get_current_player_cards_mut().remove(card);
				if self.add_to_ply(card) {
					let (winner, score) = self.get_ply_winner_and_value();
					self.ply = Default::default();
					self.scores[winner as usize % 2] += score;
					self.on_turn = winner;
				} else {
					self.on_turn += 1;
					self.on_turn = self.on_turn % 4;
				}
			}
		}
	}
	fn add_to_ply(&mut self, c: PlayingCard32) -> bool {
		let mut i = 0;
		while i < self.ply.len() {
			if self.ply[i].is_none() {
				self.ply[i] = Some(c);
				return i == 3;
			}
			i += 1;
		}
		unreachable!()
	}
	fn get_ply_winner_and_value(&self) -> (u8, u8) {
		let mut color = None;
		let mut color_is_trump = false;
		let mut current_leader = 0;
		let mut leader_rank = 0;
		let mut leader_cutted = false;
		let mut value = 0;

		for (i, c) in self.ply.iter().map(|c| c.unwrap()).enumerate() {
			value += c.value();
			if i == 0 {
				color = Some(c.color());
				color_is_trump = c.color() == self.trump_card.color();
				current_leader = self.get_player_from_ply_index(i);
				leader_rank = c.rank();
			} else {
				if !color_is_trump && c.color() == self.trump_card.color() {
					if leader_cutted {
						if leader_rank < c.rank() {
							leader_rank = c.rank();
							current_leader = self.get_player_from_ply_index(i);
						}
					} else {
						leader_cutted = true;
						leader_rank = c.rank();
						current_leader = self.get_player_from_ply_index(i);
					}
				} else {
					if !leader_cutted {
						if c.color() == color.unwrap() && leader_rank < c.rank() {
							leader_rank = c.rank();
							current_leader = self.get_player_from_ply_index(i);
						}
					}
				}
			}
		}
		(current_leader, value)
	}
}
enum PlyStatus {
	FirstToPlay,
	ColorPartnerLeads(Color),
	ColorAdverseLeads(Color, u8),
	ColorCuttedPartner(Color),
	ColorCuttedAdverse(Color, u8),
}
