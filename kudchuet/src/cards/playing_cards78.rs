use std::convert::TryFrom;

use crate::cards::playing_cards::PlayingCard;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayingCard78 {
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
	KnightOfSpades,
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
	KnightOfHearts,
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
	KnightOfDiamonds,
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
	KnightOfClubs,
	QueenOfClubs,
	KingOfClubs,

	OneOfTrumps,
	TwoOfTrumps,
	ThreeOfTrumps,
	FourOfTrumps,
	FiveOfTrumps,
	SixOfTrumps,
	SevenOfTrumps,
	EightOfTrumps,
	NineOfTrumps,
	TenOfTrumps,
	ElevenOfTrumps,
	TwelveOfTrumps,
	ThirteenOfTrumps,
	FourteenOfTrumps,
	FifteenOfTrumps,
	SixteenOfTrumps,
	SeventeenOfTrumps,
	EighteenOfTrumps,
	NineteenOfTrumps,
	TwentyOfTrumps,
	TwentyOneOfTrumps,
	Excuse,
}
impl TryFrom<u8> for PlayingCard78 {
	type Error = ();

	#[inline]
	fn try_from(v: u8) -> Result<Self, Self::Error> {
		Self::from_index(v).ok_or(())
	}
}
impl std::fmt::Display for PlayingCard78 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let v = *self as u8;

		match v {
			77 => return write!(f, "Exc"),
			_ => {}
		}
		if v < 14 * 4 {
			let suit = match v / 14 {
				0 => "♠",
				1 => "♥",
				2 => "♦",
				3 => "♣",
				_ => "T",
			};

			let rank = match v % 14 {
				0 => "A",
				10 => "J",
				11 => "Kn",
				12 => "Q",
				13 => "K",
				n => {
					// 1..9 => 2..10
					return write!(f, "{}{}", n + 1, suit);
				}
			};

			write!(f, "{}{}", rank, suit)
		} else {
			write!(f, "{}T", v - (14 * 4) + 1)
		}
	}
}

impl PlayingCard78 {
	pub const fn color(self) -> TarotColors {
		let v = self as u8;

		if v == 77 {
			return TarotColors::Excuse;
		}
		match v / 13 {
			0 => TarotColors::Spades,
			1 => TarotColors::Hearts,
			2 => TarotColors::Diamonds,
			3 => TarotColors::Clubs,
			_ => TarotColors::Trump,
		}
	}
	pub const fn from_index(v: u8) -> Option<Self> {
		if v < 78 {
			// SAFETY:
			// repr(u8) guarantees discriminant layout
			// and we checked bounds.
			Some(unsafe { std::mem::transmute(v) })
		} else {
			None
		}
	}
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TarotColors {
	Spades,
	Hearts,
	Diamonds,
	Clubs,
	Trump,
	Excuse,
}
impl PlayingCard for PlayingCard78 {
	const CARD_COUNT: u8 = 78;
	const ALL: &'static [Self] = &[
		PlayingCard78::AceOfSpades,
		PlayingCard78::TwoOfSpades,
		PlayingCard78::ThreeOfSpades,
		PlayingCard78::FourOfSpades,
		PlayingCard78::FiveOfSpades,
		PlayingCard78::SixOfSpades,
		PlayingCard78::SevenOfSpades,
		PlayingCard78::EightOfSpades,
		PlayingCard78::NineOfSpades,
		PlayingCard78::TenOfSpades,
		PlayingCard78::JackOfSpades,
		PlayingCard78::KnightOfSpades,
		PlayingCard78::QueenOfSpades,
		PlayingCard78::KingOfSpades,
		PlayingCard78::AceOfHearts,
		PlayingCard78::TwoOfHearts,
		PlayingCard78::ThreeOfHearts,
		PlayingCard78::FourOfHearts,
		PlayingCard78::FiveOfHearts,
		PlayingCard78::SixOfHearts,
		PlayingCard78::SevenOfHearts,
		PlayingCard78::EightOfHearts,
		PlayingCard78::NineOfHearts,
		PlayingCard78::TenOfHearts,
		PlayingCard78::JackOfHearts,
		PlayingCard78::KnightOfHearts,
		PlayingCard78::QueenOfHearts,
		PlayingCard78::KingOfHearts,
		PlayingCard78::AceOfDiamonds,
		PlayingCard78::TwoOfDiamonds,
		PlayingCard78::ThreeOfDiamonds,
		PlayingCard78::FourOfDiamonds,
		PlayingCard78::FiveOfDiamonds,
		PlayingCard78::SixOfDiamonds,
		PlayingCard78::SevenOfDiamonds,
		PlayingCard78::EightOfDiamonds,
		PlayingCard78::NineOfDiamonds,
		PlayingCard78::TenOfDiamonds,
		PlayingCard78::JackOfDiamonds,
		PlayingCard78::KnightOfDiamonds,
		PlayingCard78::QueenOfDiamonds,
		PlayingCard78::KingOfDiamonds,
		PlayingCard78::AceOfClubs,
		PlayingCard78::TwoOfClubs,
		PlayingCard78::ThreeOfClubs,
		PlayingCard78::FourOfClubs,
		PlayingCard78::FiveOfClubs,
		PlayingCard78::SixOfClubs,
		PlayingCard78::SevenOfClubs,
		PlayingCard78::EightOfClubs,
		PlayingCard78::NineOfClubs,
		PlayingCard78::TenOfClubs,
		PlayingCard78::JackOfClubs,
		PlayingCard78::KnightOfClubs,
		PlayingCard78::QueenOfClubs,
		PlayingCard78::KingOfClubs,
		PlayingCard78::OneOfTrumps,
		PlayingCard78::TwoOfTrumps,
		PlayingCard78::ThreeOfTrumps,
		PlayingCard78::FourOfTrumps,
		PlayingCard78::FiveOfTrumps,
		PlayingCard78::SixOfTrumps,
		PlayingCard78::SevenOfTrumps,
		PlayingCard78::EightOfTrumps,
		PlayingCard78::NineOfTrumps,
		PlayingCard78::TenOfTrumps,
		PlayingCard78::ElevenOfTrumps,
		PlayingCard78::TwelveOfTrumps,
		PlayingCard78::ThirteenOfTrumps,
		PlayingCard78::FourteenOfTrumps,
		PlayingCard78::FifteenOfTrumps,
		PlayingCard78::SixteenOfTrumps,
		PlayingCard78::SeventeenOfTrumps,
		PlayingCard78::EighteenOfTrumps,
		PlayingCard78::NineteenOfTrumps,
		PlayingCard78::TwentyOfTrumps,
		PlayingCard78::TwentyOneOfTrumps,
		PlayingCard78::Excuse,
	];
	type Color = TarotColors;

	fn index(self) -> u8 {
		self as u8
	}
	fn from_index(v: u8) -> Option<Self> {
		/*if v < 78 {
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
