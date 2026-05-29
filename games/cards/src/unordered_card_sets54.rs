use crate::playing_cards54::PlayingCard54;
use std::fmt::Write;
#[macro_export]
macro_rules! unordered_card_set54 {
	($($card:ident),* $(,)?) => {
		UnorderedCardSet54(
			0 $(| (1 << (PlayingCard54::$card as u8)))*
		)
	};
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UnorderedCardSet54(u64);
impl UnorderedCardSet54 {
	pub const EMPTY: Self = Self(0);
	pub const ALL_BUT_JOKERS: Self = Self((1 << 52) - 1);
	pub const ALL: Self = Self((1 << 54) - 1);
	pub const ALL32: Self = unordered_card_set54![
		AceOfSpades,
		AceOfHearts,
		AceOfDiamonds,
		AceOfClubs,
		SevenOfSpades,
		SevenOfHearts,
		SevenOfDiamonds,
		SevenOfClubs,
		EightOfSpades,
		EightOfHearts,
		EightOfDiamonds,
		EightOfClubs,
		NineOfSpades,
		NineOfHearts,
		NineOfDiamonds,
		NineOfClubs,
		TenOfSpades,
		TenOfHearts,
		TenOfDiamonds,
		TenOfClubs,
		JackOfSpades,
		JackOfHearts,
		JackOfDiamonds,
		JackOfClubs,
		QueenOfSpades,
		QueenOfHearts,
		QueenOfDiamonds,
		QueenOfClubs,
		KingOfSpades,
		KingOfHearts,
		KingOfDiamonds,
		KingOfClubs
	];
	#[inline]
	pub fn iter(self) -> CardSetIter {
		CardSetIter(self)
	}
	#[inline]
	pub const fn contains(self, card: PlayingCard54) -> bool {
		self.0 & (1 << (card as u8)) != 0
	}

	#[inline]
	pub const fn insert(&mut self, card: PlayingCard54) {
		self.0 |= 1 << (card as u8);
	}

	#[inline]
	pub const fn remove(&mut self, card: PlayingCard54) {
		self.0 &= !(1 << (card as u8));
	}
	#[inline]
	pub const fn len(self) -> usize {
		self.0.count_ones() as usize
	}

	#[inline]
	pub const fn is_empty(self) -> bool {
		self.0 == 0
	}
	pub fn draw_random(nb: u8, from: &mut Self) -> Result<Self, String> {
		let mut remaining = from.len() as u32;

		if remaining < nb as u32 {
			return Err("Not enough cards in the deck".into());
		}

		let mut result = Self::EMPTY;

		for _ in 0..nb {
			let mut idx = fastrand::u32(0..remaining);
			let mut x = from.0;

			let mask = loop {
				let bit = x.trailing_zeros();
				let m = 1u64 << bit;

				if idx == 0 {
					break m;
				}

				x &= x - 1;
				idx -= 1;
			};

			from.0 ^= mask;
			result.0 |= mask;

			remaining -= 1;
		}

		Ok(result)
	}
}
impl From<PlayingCard54> for UnorderedCardSet54 {
	#[inline]
	fn from(card: PlayingCard54) -> Self {
		Self(1u64 << card as u8)
	}
}
impl Default for UnorderedCardSet54 {
	fn default() -> Self {
		Self::EMPTY
	}
}
impl IntoIterator for UnorderedCardSet54 {
	type Item = PlayingCard54;
	type IntoIter = CardSetIter;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
impl std::ops::BitOrAssign<PlayingCard54> for UnorderedCardSet54 {
	#[inline]
	fn bitor_assign(&mut self, rhs: PlayingCard54) {
		self.0 |= 1u64 << rhs as u8;
	}
}
impl std::ops::BitOr for PlayingCard54 {
	type Output = UnorderedCardSet54;

	#[inline]
	fn bitor(self, rhs: Self) -> Self::Output {
		UnorderedCardSet54::from(self) | UnorderedCardSet54::from(rhs)
	}
}
impl std::ops::BitOr<PlayingCard54> for UnorderedCardSet54 {
	type Output = UnorderedCardSet54;

	fn bitor(self, rhs: PlayingCard54) -> Self::Output {
		UnorderedCardSet54(self.0 | (1u64 << rhs as u8))
	}
}
impl std::ops::BitOr for UnorderedCardSet54 {
	type Output = UnorderedCardSet54;

	fn bitor(self, rhs: Self) -> Self::Output {
		UnorderedCardSet54(self.0 | rhs.0)
	}
}
impl std::ops::BitAndAssign for UnorderedCardSet54 {
	#[inline]
	fn bitand_assign(&mut self, rhs: Self) {
		self.0 &= rhs.0
	}
}
impl std::ops::BitAnd for UnorderedCardSet54 {
	type Output = UnorderedCardSet54;

	#[inline]
	fn bitand(self, rhs: Self) -> Self::Output {
		UnorderedCardSet54(self.0 & rhs.0)
	}
}
impl std::ops::SubAssign for UnorderedCardSet54 {
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.0 &= !rhs.0
	}
}
impl std::ops::Sub for UnorderedCardSet54 {
	type Output = UnorderedCardSet54;

	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		UnorderedCardSet54(self.0 & !rhs.0)
	}
}
impl std::fmt::Display for UnorderedCardSet54 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_char('[')?;

		let mut first = true;

		for c in *self {
			if !first {
				f.write_str(", ")?;
			}
			first = false;

			write!(f, "{}", c)?;
		}

		f.write_char(']')
	}
}

#[derive(Clone)]
pub struct CardSetIter(UnorderedCardSet54);
impl Iterator for CardSetIter {
	type Item = PlayingCard54;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.0.0 != 0 {
			let res = self.0.0.trailing_zeros() as u8;
			self.0.0 &= self.0.0 - 1;
			res.try_into().ok()
		} else {
			None
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let count = self.0.len();
		(count, Some(count))
	}
}
impl ExactSizeIterator for CardSetIter {}

#[cfg(test)]
mod tests {
	use crate::unordered_card_sets54::UnorderedCardSet54;

	#[test]
	fn test_draw_random() {
		let mut deck = UnorderedCardSet54::ALL;
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
			UnorderedCardSet54::ALL
		);
		println!("Player1: {}", player1);
		println!("Player2: {}", player2);
		println!("Player3: {}", player3);
		println!("Player4: {}", player4);
	}
}
