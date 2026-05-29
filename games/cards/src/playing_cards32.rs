use std::convert::TryFrom;


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
		if v < 32 {
			// SAFETY:
			// repr(u8) guarantees discriminant layout
			// and we checked bounds.
			Ok(unsafe { std::mem::transmute(v) })
		} else {
			Err(())
		}
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
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Color {
	Spades,
	Hearts,
	Diamonds,
	Clubs
}
impl PlayingCard32 {
	pub const fn color(self) -> Color {
		let v = self as u8;

		match v / 8 {
			0 => Color::Spades,
			1 => Color::Hearts,
			2 => Color::Diamonds,
			3 => Color::Clubs,
			_ => unreachable!(),
		}
	}
}