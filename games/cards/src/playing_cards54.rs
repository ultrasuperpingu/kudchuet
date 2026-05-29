use std::convert::TryFrom;


#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
		if v < 54 {
			// SAFETY:
			// repr(u8) guarantees discriminant layout
			// and we checked bounds.
			Ok(unsafe { std::mem::transmute(v) })
		} else {
			Err(())
		}
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

#[cfg(test)]
mod tests {
	use crate::unordered_card_sets54::UnorderedCardSet54;

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
