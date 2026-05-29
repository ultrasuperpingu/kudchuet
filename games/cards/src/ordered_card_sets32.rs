use crate::playing_cards32::{Color, PlayingCard32};
use std::{fmt::Write, slice::Iter};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrderedCardSet32(Vec<PlayingCard32>);
impl OrderedCardSet32 {
	pub const EMPTY: Self = Self(vec![]);
	//pub const ALL: Self = Self(u32::MAX);
	pub fn by_color(color: Color) -> Self {
		match color {
			Color::Spades => Self(vec![
				PlayingCard32::SevenOfSpades,
				PlayingCard32::EightOfSpades,
				PlayingCard32::NineOfSpades,
				PlayingCard32::TenOfSpades,
				PlayingCard32::JackOfSpades,
				PlayingCard32::QueenOfSpades,
				PlayingCard32::KingOfSpades,
				PlayingCard32::AceOfSpades,
			]),
			Color::Hearts => Self(vec![
				PlayingCard32::SevenOfHearts,
				PlayingCard32::EightOfHearts,
				PlayingCard32::NineOfHearts,
				PlayingCard32::TenOfHearts,
				PlayingCard32::JackOfHearts,
				PlayingCard32::QueenOfHearts,
				PlayingCard32::KingOfHearts,
				PlayingCard32::AceOfHearts,
			]),
			Color::Diamonds => Self(vec![
				PlayingCard32::SevenOfDiamonds,
				PlayingCard32::EightOfDiamonds,
				PlayingCard32::NineOfDiamonds,
				PlayingCard32::TenOfDiamonds,
				PlayingCard32::JackOfDiamonds,
				PlayingCard32::QueenOfDiamonds,
				PlayingCard32::KingOfDiamonds,
				PlayingCard32::AceOfDiamonds,
			]),
			Color::Clubs => Self(vec![
				PlayingCard32::SevenOfClubs,
				PlayingCard32::EightOfClubs,
				PlayingCard32::NineOfClubs,
				PlayingCard32::TenOfClubs,
				PlayingCard32::JackOfClubs,
				PlayingCard32::QueenOfClubs,
				PlayingCard32::KingOfClubs,
				PlayingCard32::AceOfClubs,
			]),
		}
	}

	#[inline]
	pub fn iter(&self) -> Iter<'_, PlayingCard32> {
		self.0.iter()
	}
	#[inline]
	pub fn contains(&self, card: PlayingCard32) -> bool {
		self.0.contains(&card)
	}

	#[inline]
	pub fn insert(&mut self, card: PlayingCard32) {
		self.0.push(card);
	}

	#[inline]
	pub fn remove(&mut self, card: PlayingCard32) {
		if let Some(index) = self.0.iter().position(|c| *c == card) {
			self.0.remove(index);
		}
	}
	#[inline]
	pub const fn len(&self) -> usize {
		self.0.len()
	}

	#[inline]
	pub const fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
	pub fn of_color(self, color: Color) -> Self {
		self & Self::by_color(color)
	}
	pub fn draw_random(nb: u8, from: &mut Self) -> Result<Self, String> {
		if from.len() < nb as usize {
			return Err("Not enough cards in the deck".into());
		}

		let mut result = Vec::with_capacity(nb as usize);

		for _ in 0..nb {
			let idx = fastrand::usize(..from.0.len());
			result.push(from.0.swap_remove(idx));
		}

		Ok(Self(result))
	}
}
impl From<PlayingCard32> for OrderedCardSet32 {
	#[inline]
	fn from(card: PlayingCard32) -> Self {
		Self(vec![card])
	}
}
impl Default for OrderedCardSet32 {
	fn default() -> Self {
		Self::EMPTY
	}
}
impl IntoIterator for OrderedCardSet32 {
	type Item = PlayingCard32;
	type IntoIter = std::slice::Iter<'_, PlayingCard32>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
impl FromIterator<PlayingCard32> for OrderedCardSet32 {
	fn from_iter<T: IntoIterator<Item = PlayingCard32>>(iter: T) -> Self {
		let mut set = Self::EMPTY;

		for card in iter {
			set.insert(card);
		}

		set
	}
}
impl std::ops::BitOrAssign<PlayingCard32> for OrderedCardSet32 {
	#[inline]
	fn bitor_assign(&mut self, rhs: PlayingCard32) {
		self.0.push(rhs);
	}
}
impl std::ops::BitOr<PlayingCard32> for OrderedCardSet32 {
	type Output = OrderedCardSet32;

	fn bitor(self, rhs: PlayingCard32) -> Self::Output {
		let mut s = self.0.clone();
		s.push(rhs);
		Self(s)
	}
}
impl std::ops::BitOr for OrderedCardSet32 {
	type Output = OrderedCardSet32;

	fn bitor(self, rhs: Self) -> Self::Output {
		let mut s = self.0.clone();
		s.extend(rhs);
		Self(s)
	}
}
impl std::ops::BitAndAssign for OrderedCardSet32 {
	#[inline]
	fn bitand_assign(&mut self, rhs: Self) {
		self.0.retain(|c| rhs.0.contains(&c));
	}
}
impl std::ops::BitAnd for OrderedCardSet32 {
	type Output = OrderedCardSet32;

	#[inline]
	fn bitand(self, rhs: Self) -> Self::Output {
		let mut s = self.0.clone();
		s.retain(|c| rhs.0.contains(&c));
		Self(s)
	}
}
impl std::ops::SubAssign for OrderedCardSet32 {
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.0.retain(|c| !rhs.0.contains(&c));
	}
}
impl std::ops::Sub for OrderedCardSet32 {
	type Output = OrderedCardSet32;

	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		let mut s = self.0.clone();
		s.retain(|c| !rhs.0.contains(&c));
		Self(s)
	}
}
impl std::fmt::Display for OrderedCardSet32 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_char('[')?;

		let mut first = true;

		for c in self.iter() {
			if !first {
				f.write_str(", ")?;
			}
			first = false;

			write!(f, "{}", c)?;
		}

		f.write_char(']')
	}
}

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
