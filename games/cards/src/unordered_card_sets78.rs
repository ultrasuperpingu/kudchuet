use crate::playing_cards78::PlayingCard78;
use std::fmt::Write;
#[macro_export]
macro_rules! unordered_card_set78 {
	($($card:ident),* $(,)?) => {
		UnorderedCardSet(
			0 $(| (1 << (PlayingCard78::$card as u8)))*
		)
	};
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UnorderedCardSet78(u128);
impl UnorderedCardSet78 {
	pub const EMPTY: Self = Self(0);
	pub const ALL: Self = Self((1 << 78) - 1);

	#[inline]
	pub fn iter(self) -> CardSetIter {
		CardSetIter(self)
	}
	#[inline]
	pub const fn contains(self, card: PlayingCard78) -> bool {
		self.0 & (1 << (card as u8)) != 0
	}

	#[inline]
	pub const fn insert(&mut self, card: PlayingCard78) {
		self.0 |= 1 << (card as u8);
	}

	#[inline]
	pub const fn remove(&mut self, card: PlayingCard78) {
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
impl From<PlayingCard78> for UnorderedCardSet78 {
	#[inline]
	fn from(card: PlayingCard78) -> Self {
		Self(1 << card as u8)
	}
}
impl Default for UnorderedCardSet78 {
	fn default() -> Self {
		Self::EMPTY
	}
}
impl IntoIterator for UnorderedCardSet78 {
	type Item = PlayingCard78;
	type IntoIter = CardSetIter;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
impl std::ops::BitOrAssign<PlayingCard78> for UnorderedCardSet78 {
	#[inline]
	fn bitor_assign(&mut self, rhs: PlayingCard78) {
		self.0 |= 1 << rhs as u8;
	}
}
impl std::ops::BitOr for PlayingCard78 {
	type Output = UnorderedCardSet78;

	#[inline]
	fn bitor(self, rhs: Self) -> Self::Output {
		UnorderedCardSet78::from(self) | UnorderedCardSet78::from(rhs)
	}
}
impl std::ops::BitOr<PlayingCard78> for UnorderedCardSet78 {
	type Output = UnorderedCardSet78;

	fn bitor(self, rhs: PlayingCard78) -> Self::Output {
		UnorderedCardSet78(self.0 | (1 << rhs as u8))
	}
}
impl std::ops::BitOr for UnorderedCardSet78 {
	type Output = UnorderedCardSet78;

	fn bitor(self, rhs: Self) -> Self::Output {
		UnorderedCardSet78(self.0 | rhs.0)
	}
}
impl std::ops::BitAndAssign for UnorderedCardSet78 {
	#[inline]
	fn bitand_assign(&mut self, rhs: Self) {
		self.0 &= rhs.0
	}
}
impl std::ops::BitAnd for UnorderedCardSet78 {
	type Output = UnorderedCardSet78;

	#[inline]
	fn bitand(self, rhs: Self) -> Self::Output {
		UnorderedCardSet78(self.0 & rhs.0)
	}
}
impl std::ops::SubAssign for UnorderedCardSet78 {
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.0 &= !rhs.0
	}
}
impl std::ops::Sub for UnorderedCardSet78 {
	type Output = UnorderedCardSet78;

	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		UnorderedCardSet78(self.0 & !rhs.0)
	}
}
impl std::fmt::Display for UnorderedCardSet78 {
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
pub struct CardSetIter(UnorderedCardSet78);
impl Iterator for CardSetIter {
	type Item = PlayingCard78;

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
	use crate::unordered_card_sets78::UnorderedCardSet78;

	#[test]
	fn test_draw_random() {
		let mut deck = UnorderedCardSet78::ALL;
		let player1 = UnorderedCardSet78::draw_random(18, &mut deck).expect("Should work");
		let player2 = UnorderedCardSet78::draw_random(18, &mut deck).expect("Should work");
		let player3 = UnorderedCardSet78::draw_random(18, &mut deck).expect("Should work");
		let player4 = UnorderedCardSet78::draw_random(18, &mut deck).expect("Should work");
		assert_eq!(deck.len(), 6);
		assert_eq!(player1.len(), 18);
		assert_eq!(player2.len(), 18);
		assert_eq!(player3.len(), 18);
		assert_eq!(player4.len(), 18);
		assert_eq!(
			player1 | player2 | player3 | player4 | deck,
			UnorderedCardSet78::ALL
		);
		println!("Player1: {}", player1);
		println!("Player2: {}", player2);
		println!("Player3: {}", player3);
		println!("Player4: {}", player4);
		println!("Dog: {}", deck);
	}
}
