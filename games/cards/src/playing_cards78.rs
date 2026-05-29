use std::convert::TryFrom;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
		if v < 78 {
			// SAFETY:
			// repr(u8) guarantees discriminant layout
			// and we checked bounds.
			Ok(unsafe { std::mem::transmute(v) })
		} else {
			Err(())
		}
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
			write!(f, "{}T", v-(14*4)+1)
		}
	}
}
