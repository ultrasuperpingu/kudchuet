use crate::{playing_cards78::PlayingCard78, unordered_card_sets78::UnorderedCardSet78};

pub struct Tarot {
	players: [UnorderedCardSet78; 4],
	dog: UnorderedCardSet78,
	best_bid: Option<Bid>,
	scores: [u8; 2],
	ply: [Option<PlayingCard78>; 4],
	on_turn: u8,
}
impl Default for Tarot {
	fn default() -> Self {
		let mut deck = UnorderedCardSet78::ALL;
		Self {
			players: [
				UnorderedCardSet78::draw_random(18, &mut deck).unwrap(),
				UnorderedCardSet78::draw_random(18, &mut deck).unwrap(),
				UnorderedCardSet78::draw_random(18, &mut deck).unwrap(),
				UnorderedCardSet78::draw_random(18, &mut deck).unwrap(),
			],
			dog: deck,
			best_bid: Default::default(),
			scores: Default::default(),
			ply: Default::default(),
			on_turn: 0,
		}
	}
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Bid {
	Pass,
	Prise(u8),
	Garde(u8),
	GardeSans(u8),
	GardeContre(u8),
}
impl PartialOrd for Bid {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		match (&self, other) {
			(Bid::Pass, Bid::Pass) => Some(std::cmp::Ordering::Equal),
			(Bid::Pass, _) => Some(std::cmp::Ordering::Greater),
			(_, Bid::Pass) => Some(std::cmp::Ordering::Less),
			(Bid::Prise(_), Bid::Prise(_)) => Some(std::cmp::Ordering::Equal),
			(Bid::Prise(_), _) => Some(std::cmp::Ordering::Greater),
			(Bid::Garde(_), Bid::Prise(_)) => Some(std::cmp::Ordering::Less),
			(Bid::Garde(_), Bid::Garde(_)) => Some(std::cmp::Ordering::Equal),
			(Bid::Garde(_), _) => Some(std::cmp::Ordering::Greater),
			(Bid::GardeSans(_), Bid::GardeSans(_)) => Some(std::cmp::Ordering::Equal),
			(Bid::GardeSans(_), Bid::GardeContre(_)) => Some(std::cmp::Ordering::Greater),
			(Bid::GardeSans(_), _) => Some(std::cmp::Ordering::Less),
			(Bid::GardeContre(_), Bid::GardeContre(_)) => Some(std::cmp::Ordering::Equal),
			(Bid::GardeContre(_), _) => Some(std::cmp::Ordering::Less),
		}
	}
}
pub enum Move {
	Bid(Bid),
	MakeDog(UnorderedCardSet78),
	Play(PlayingCard78)
}
