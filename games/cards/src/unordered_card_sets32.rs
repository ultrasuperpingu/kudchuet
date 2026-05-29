use crate::playing_cards32::{Color, PlayingCard32};
use std::fmt::Write;
#[macro_export]
macro_rules! unordered_card_set32 {
	($($card:ident),* $(,)?) => {
		UnorderedCardSet32(
			0 $(| (1 << (PlayingCard32::$card as u8)))*
		)
	};
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UnorderedCardSet32(u32);
impl UnorderedCardSet32 {
	pub const EMPTY: Self = Self(0);
	pub const ALL: Self = Self(u32::MAX);
	pub const fn by_color(color: Color) -> Self {
		match color {
			Color::Spades => unordered_card_set32![
				SevenOfSpades,
				EightOfSpades,
				NineOfSpades,
				TenOfSpades,
				JackOfSpades,
				QueenOfSpades,
				KingOfSpades,
				AceOfSpades,
			],
			Color::Hearts => unordered_card_set32![
				SevenOfHearts,
				EightOfHearts,
				NineOfHearts,
				TenOfHearts,
				JackOfHearts,
				QueenOfHearts,
				KingOfHearts,
				AceOfHearts,
			],
			Color::Diamonds => unordered_card_set32![
				SevenOfDiamonds,
				EightOfDiamonds,
				NineOfDiamonds,
				TenOfDiamonds,
				JackOfDiamonds,
				QueenOfDiamonds,
				KingOfDiamonds,
				AceOfDiamonds,
			],
			Color::Clubs => unordered_card_set32![
				SevenOfClubs,
				EightOfClubs,
				NineOfClubs,
				TenOfClubs,
				JackOfClubs,
				QueenOfClubs,
				KingOfClubs,
				AceOfClubs,
			],
		}
	}

	#[inline]
	pub fn iter(self) -> CardSetIter {
		CardSetIter(self)
	}
	#[inline]
	pub const fn contains(self, card: PlayingCard32) -> bool {
		self.0 & (1 << (card as u8)) != 0
	}

	#[inline]
	pub const fn insert(&mut self, card: PlayingCard32) {
		self.0 |= 1 << (card as u8);
	}

	#[inline]
	pub const fn remove(&mut self, card: PlayingCard32) {
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
	pub fn of_color(self, color: Color) -> Self {
		self & Self::by_color(color)
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
				let m = 1 << bit;

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
impl From<PlayingCard32> for UnorderedCardSet32 {
	#[inline]
	fn from(card: PlayingCard32) -> Self {
		Self(1 << card as u8)
	}
}
impl Default for UnorderedCardSet32 {
	fn default() -> Self {
		Self::EMPTY
	}
}
impl IntoIterator for UnorderedCardSet32 {
	type Item = PlayingCard32;
	type IntoIter = CardSetIter;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
impl FromIterator<PlayingCard32> for UnorderedCardSet32 {
	fn from_iter<T: IntoIterator<Item = PlayingCard32>>(iter: T) -> Self {
		let mut set = Self::EMPTY;

		for card in iter {
			set.insert(card);
		}

		set
	}
}
impl std::ops::BitOrAssign<PlayingCard32> for UnorderedCardSet32 {
	#[inline]
	fn bitor_assign(&mut self, rhs: PlayingCard32) {
		self.0 |= 1 << rhs as u8;
	}
}
impl std::ops::BitOr for PlayingCard32 {
	type Output = UnorderedCardSet32;

	#[inline]
	fn bitor(self, rhs: Self) -> Self::Output {
		UnorderedCardSet32::from(self) | UnorderedCardSet32::from(rhs)
	}
}
impl std::ops::BitOr<PlayingCard32> for UnorderedCardSet32 {
	type Output = UnorderedCardSet32;

	fn bitor(self, rhs: PlayingCard32) -> Self::Output {
		UnorderedCardSet32(self.0 | (1 << rhs as u8))
	}
}
impl std::ops::BitOr for UnorderedCardSet32 {
	type Output = UnorderedCardSet32;

	fn bitor(self, rhs: Self) -> Self::Output {
		UnorderedCardSet32(self.0 | rhs.0)
	}
}
impl std::ops::BitAndAssign for UnorderedCardSet32 {
	#[inline]
	fn bitand_assign(&mut self, rhs: Self) {
		self.0 &= rhs.0
	}
}
impl std::ops::BitAnd for UnorderedCardSet32 {
	type Output = UnorderedCardSet32;

	#[inline]
	fn bitand(self, rhs: Self) -> Self::Output {
		UnorderedCardSet32(self.0 & rhs.0)
	}
}
impl std::ops::SubAssign for UnorderedCardSet32 {
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.0 &= !rhs.0
	}
}
impl std::ops::Sub for UnorderedCardSet32 {
	type Output = UnorderedCardSet32;

	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		UnorderedCardSet32(self.0 & !rhs.0)
	}
}
impl std::fmt::Display for UnorderedCardSet32 {
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

#[derive(Clone, Copy)]
pub struct CardSetIter(UnorderedCardSet32);
impl Iterator for CardSetIter {
	type Item = PlayingCard32;

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
impl std::iter::FusedIterator for CardSetIter {}
#[cfg(test)]
mod tests {
	use crate::unordered_card_sets32::UnorderedCardSet32;

	#[test]
	fn test_draw_random() {
		let mut deck = UnorderedCardSet32::ALL;
		let player1 = UnorderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		let player2 = UnorderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		let player3 = UnorderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		let player4 = UnorderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		assert_eq!(deck.len(), 0);
		assert_eq!(player1.len(), 8);
		assert_eq!(player2.len(), 8);
		assert_eq!(player3.len(), 8);
		assert_eq!(player4.len(), 8);
		assert_eq!(
			player1 | player2 | player3 | player4,
			UnorderedCardSet32::ALL
		);
		println!("Player1: {}", player1);
		println!("Player2: {}", player2);
		println!("Player3: {}", player3);
		println!("Player4: {}", player4);
	}
}
