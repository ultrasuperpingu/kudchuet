use std::convert::TryFrom;

use crate::playing_cards::{CardSuit, PlayingCard};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayingCard32 {
	SevenOfSpades,
	EightOfSpades,
	NineOfSpades,
	TenOfSpades,
	JackOfSpades,
	QueenOfSpades,
	KingOfSpades,
	AceOfSpades,

	SevenOfHearts,
	EightOfHearts,
	NineOfHearts,
	TenOfHearts,
	JackOfHearts,
	QueenOfHearts,
	KingOfHearts,
	AceOfHearts,

	SevenOfDiamonds,
	EightOfDiamonds,
	NineOfDiamonds,
	TenOfDiamonds,
	JackOfDiamonds,
	QueenOfDiamonds,
	KingOfDiamonds,
	AceOfDiamonds,

	SevenOfClubs,
	EightOfClubs,
	NineOfClubs,
	TenOfClubs,
	JackOfClubs,
	QueenOfClubs,
	KingOfClubs,
	AceOfClubs,
}
impl TryFrom<u8> for PlayingCard32 {
	type Error = ();

	#[inline]
	fn try_from(v: u8) -> Result<Self, Self::Error> {
		Self::from_index(v).ok_or(())
	}
}
impl std::fmt::Display for PlayingCard32 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let v = *self as u8;

		let suit = match v / 8 {
			0 => "♠",
			1 => "♥",
			2 => "♦",
			3 => "♣",
			_ => "?",
		};

		let rank = match v % 8 {
			4 => "J",
			5 => "Q",
			6 => "K",
			7 => "A",
			n => {
				// 0..=3 => 7..=10
				return write!(f, "{}{}", n + 7, suit);
			}
		};

		write!(f, "{}{}", rank, suit)
	}
}
impl PlayingCard32 {
	pub const fn color(self) -> CardSuit {
		let v = self as u8;

		match v / 8 {
			0 => CardSuit::Spades,
			1 => CardSuit::Hearts,
			2 => CardSuit::Diamonds,
			3 => CardSuit::Clubs,
			_ => unreachable!(),
		}
	}
	pub const fn from_index(v: u8) -> Option<Self> {
		if v < 32 {
			// SAFETY:
			// repr(u8) guarantees discriminant layout
			// and we checked bounds.
			Some(unsafe { std::mem::transmute(v) })
		} else {
			None
		}
	}
}
impl PlayingCard for PlayingCard32 {
	const CARD_COUNT: u8 = 32;
	const ALL: &'static [Self] = &[
		PlayingCard32::SevenOfSpades,
		PlayingCard32::EightOfSpades,
		PlayingCard32::NineOfSpades,
		PlayingCard32::TenOfSpades,
		PlayingCard32::JackOfSpades,
		PlayingCard32::QueenOfSpades,
		PlayingCard32::KingOfSpades,
		PlayingCard32::AceOfSpades,
		PlayingCard32::SevenOfHearts,
		PlayingCard32::EightOfHearts,
		PlayingCard32::NineOfHearts,
		PlayingCard32::TenOfHearts,
		PlayingCard32::JackOfHearts,
		PlayingCard32::QueenOfHearts,
		PlayingCard32::KingOfHearts,
		PlayingCard32::AceOfHearts,
		PlayingCard32::SevenOfDiamonds,
		PlayingCard32::EightOfDiamonds,
		PlayingCard32::NineOfDiamonds,
		PlayingCard32::TenOfDiamonds,
		PlayingCard32::JackOfDiamonds,
		PlayingCard32::QueenOfDiamonds,
		PlayingCard32::KingOfDiamonds,
		PlayingCard32::AceOfDiamonds,
		PlayingCard32::SevenOfClubs,
		PlayingCard32::EightOfClubs,
		PlayingCard32::NineOfClubs,
		PlayingCard32::TenOfClubs,
		PlayingCard32::JackOfClubs,
		PlayingCard32::QueenOfClubs,
		PlayingCard32::KingOfClubs,
		PlayingCard32::AceOfClubs,
	];
	type Color = CardSuit;

	fn index(self) -> u8 {
		self as u8
	}
	fn color(self) -> Self::Color {
		self.color()
	}
	fn from_index(v: u8) -> Option<Self> {
		/*if v < 32 {
			// SAFETY:
			// repr(u8) guarantees discriminant layout
			// and we checked bounds.
			Some(unsafe { std::mem::transmute(v) })
		} else {
			None
		}*/
		Self::from_index(v)
	}
}
