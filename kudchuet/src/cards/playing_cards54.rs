use std::convert::TryFrom;

use crate::cards::playing_cards::{CardSuit, PlayingCard};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayingCard54 {
	AceOfSpades = 0,
	TwoOfSpades,
	ThreeOfSpades,
	FourOfSpades,
	FiveOfSpades,
	SixOfSpades,
	SevenOfSpades,
	EightOfSpades,
	NineOfSpades,
	TenOfSpades,
	JackOfSpades,
	QueenOfSpades,
	KingOfSpades,

	AceOfHearts,
	TwoOfHearts,
	ThreeOfHearts,
	FourOfHearts,
	FiveOfHearts,
	SixOfHearts,
	SevenOfHearts,
	EightOfHearts,
	NineOfHearts,
	TenOfHearts,
	JackOfHearts,
	QueenOfHearts,
	KingOfHearts,

	AceOfDiamonds,
	TwoOfDiamonds,
	ThreeOfDiamonds,
	FourOfDiamonds,
	FiveOfDiamonds,
	SixOfDiamonds,
	SevenOfDiamonds,
	EightOfDiamonds,
	NineOfDiamonds,
	TenOfDiamonds,
	JackOfDiamonds,
	QueenOfDiamonds,
	KingOfDiamonds,

	AceOfClubs,
	TwoOfClubs,
	ThreeOfClubs,
	FourOfClubs,
	FiveOfClubs,
	SixOfClubs,
	SevenOfClubs,
	EightOfClubs,
	NineOfClubs,
	TenOfClubs,
	JackOfClubs,
	QueenOfClubs,
	KingOfClubs,

	BlackJoker,
	RedJoker,
}
impl TryFrom<u8> for PlayingCard54 {
	type Error = ();

	#[inline]
	fn try_from(v: u8) -> Result<Self, Self::Error> {
		Self::from_index(v).ok_or(())
	}
}
impl std::fmt::Display for PlayingCard54 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let v = *self as u8;

		match v {
			52 => return write!(f, "BJ"),
			53 => return write!(f, "RJ"),
			_ => {}
		}

		let suit = match v / 13 {
			0 => "♠",
			1 => "♥",
			2 => "♦",
			3 => "♣",
			_ => "?",
		};

		let rank = match v % 13 {
			0 => "A",
			10 => "J",
			11 => "Q",
			12 => "K",
			n => {
				// 1..9 => 2..10
				return write!(f, "{}{}", n + 1, suit);
			}
		};

		write!(f, "{}{}", rank, suit)
	}
}

impl PlayingCard54 {
	pub const ALL_BUT_JOKERS: &'static [Self] = &[
		PlayingCard54::AceOfSpades,
		PlayingCard54::TwoOfSpades,
		PlayingCard54::ThreeOfSpades,
		PlayingCard54::FourOfSpades,
		PlayingCard54::FiveOfSpades,
		PlayingCard54::SixOfSpades,
		PlayingCard54::SevenOfSpades,
		PlayingCard54::EightOfSpades,
		PlayingCard54::NineOfSpades,
		PlayingCard54::TenOfSpades,
		PlayingCard54::JackOfSpades,
		PlayingCard54::QueenOfSpades,
		PlayingCard54::KingOfSpades,
		PlayingCard54::AceOfHearts,
		PlayingCard54::TwoOfHearts,
		PlayingCard54::ThreeOfHearts,
		PlayingCard54::FourOfHearts,
		PlayingCard54::FiveOfHearts,
		PlayingCard54::SixOfHearts,
		PlayingCard54::SevenOfHearts,
		PlayingCard54::EightOfHearts,
		PlayingCard54::NineOfHearts,
		PlayingCard54::TenOfHearts,
		PlayingCard54::JackOfHearts,
		PlayingCard54::QueenOfHearts,
		PlayingCard54::KingOfHearts,
		PlayingCard54::AceOfDiamonds,
		PlayingCard54::TwoOfDiamonds,
		PlayingCard54::ThreeOfDiamonds,
		PlayingCard54::FourOfDiamonds,
		PlayingCard54::FiveOfDiamonds,
		PlayingCard54::SixOfDiamonds,
		PlayingCard54::SevenOfDiamonds,
		PlayingCard54::EightOfDiamonds,
		PlayingCard54::NineOfDiamonds,
		PlayingCard54::TenOfDiamonds,
		PlayingCard54::JackOfDiamonds,
		PlayingCard54::QueenOfDiamonds,
		PlayingCard54::KingOfDiamonds,
		PlayingCard54::AceOfClubs,
		PlayingCard54::TwoOfClubs,
		PlayingCard54::ThreeOfClubs,
		PlayingCard54::FourOfClubs,
		PlayingCard54::FiveOfClubs,
		PlayingCard54::SixOfClubs,
		PlayingCard54::SevenOfClubs,
		PlayingCard54::EightOfClubs,
		PlayingCard54::NineOfClubs,
		PlayingCard54::TenOfClubs,
		PlayingCard54::JackOfClubs,
		PlayingCard54::QueenOfClubs,
		PlayingCard54::KingOfClubs,
	];
	pub const fn color(self) -> CardSuit {
		let v = self as u8;

		match v / 13 {
			0 => CardSuit::Spades,
			1 => CardSuit::Hearts,
			2 => CardSuit::Diamonds,
			3 => CardSuit::Clubs,
			_ => unreachable!(),
		}
	}
	pub const fn from_index(v: u8) -> Option<Self> {
		if v < 54 {
			// SAFETY:
			// repr(u8) guarantees discriminant layout
			// and we checked bounds.
			Some(unsafe { std::mem::transmute(v) })
		} else {
			None
		}
	}
}

impl PlayingCard for PlayingCard54 {
	const CARD_COUNT: u8 = 54;
	const ALL: &'static [Self] = &[
		PlayingCard54::AceOfSpades,
		PlayingCard54::TwoOfSpades,
		PlayingCard54::ThreeOfSpades,
		PlayingCard54::FourOfSpades,
		PlayingCard54::FiveOfSpades,
		PlayingCard54::SixOfSpades,
		PlayingCard54::SevenOfSpades,
		PlayingCard54::EightOfSpades,
		PlayingCard54::NineOfSpades,
		PlayingCard54::TenOfSpades,
		PlayingCard54::JackOfSpades,
		PlayingCard54::QueenOfSpades,
		PlayingCard54::KingOfSpades,
		PlayingCard54::AceOfHearts,
		PlayingCard54::TwoOfHearts,
		PlayingCard54::ThreeOfHearts,
		PlayingCard54::FourOfHearts,
		PlayingCard54::FiveOfHearts,
		PlayingCard54::SixOfHearts,
		PlayingCard54::SevenOfHearts,
		PlayingCard54::EightOfHearts,
		PlayingCard54::NineOfHearts,
		PlayingCard54::TenOfHearts,
		PlayingCard54::JackOfHearts,
		PlayingCard54::QueenOfHearts,
		PlayingCard54::KingOfHearts,
		PlayingCard54::AceOfDiamonds,
		PlayingCard54::TwoOfDiamonds,
		PlayingCard54::ThreeOfDiamonds,
		PlayingCard54::FourOfDiamonds,
		PlayingCard54::FiveOfDiamonds,
		PlayingCard54::SixOfDiamonds,
		PlayingCard54::SevenOfDiamonds,
		PlayingCard54::EightOfDiamonds,
		PlayingCard54::NineOfDiamonds,
		PlayingCard54::TenOfDiamonds,
		PlayingCard54::JackOfDiamonds,
		PlayingCard54::QueenOfDiamonds,
		PlayingCard54::KingOfDiamonds,
		PlayingCard54::AceOfClubs,
		PlayingCard54::TwoOfClubs,
		PlayingCard54::ThreeOfClubs,
		PlayingCard54::FourOfClubs,
		PlayingCard54::FiveOfClubs,
		PlayingCard54::SixOfClubs,
		PlayingCard54::SevenOfClubs,
		PlayingCard54::EightOfClubs,
		PlayingCard54::NineOfClubs,
		PlayingCard54::TenOfClubs,
		PlayingCard54::JackOfClubs,
		PlayingCard54::QueenOfClubs,
		PlayingCard54::KingOfClubs,
		PlayingCard54::BlackJoker,
		PlayingCard54::RedJoker,
	];
	
	type Color = CardSuit;

	fn index(self) -> u8 {
		self as u8
	}
	fn from_index(v: u8) -> Option<Self> {
		/*if v < 54 {
			// SAFETY:
			// repr(u8) guarantees discriminant layout
			// and we checked bounds.
			Some(unsafe { std::mem::transmute(v) })
		} else {
			None
		}*/
		Self::from_index(v)
	}
	fn color(self) -> Self::Color {
		self.color()
	}
}
#[cfg(test)]
mod tests {
	use crate::cards::{playing_cards::CardSet, unordered_card_set::UnorderedCardSet54};

	#[test]
	fn test_draw_random() {
		let mut deck = UnorderedCardSet54::ALL32;
		let player1 = UnorderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		let player2 = UnorderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		let player3 = UnorderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		let player4 = UnorderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		assert_eq!(deck.len(), 0);
		assert_eq!(player1.len(), 8);
		assert_eq!(player2.len(), 8);
		assert_eq!(player3.len(), 8);
		assert_eq!(player4.len(), 8);
		assert_eq!(
			player1 | player2 | player3 | player4,
			UnorderedCardSet54::ALL32
		);
		println!("Player1: {}", player1);
		println!("Player2: {}", player2);
		println!("Player3: {}", player3);
		println!("Player4: {}", player4);
	}
}
