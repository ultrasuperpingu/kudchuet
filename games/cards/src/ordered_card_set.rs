use crate::{
	playing_cards::{CardSet, CardSuit, PlayingCard},
	playing_cards32::PlayingCard32,
	playing_cards54::PlayingCard54,
	playing_cards78::PlayingCard78,
	unordered_card_set::{UnorderedCardSet32, UnorderedCardSet54, UnorderedCardSet78},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrderedCardSet<C>(pub(crate) Vec<C>);
//pub struct OrderedCardSet<C>(pub(crate) Vec<C>);

//pub type OrderedCardSet32 = OrderedCardSet<PlayingCard32, 32>;
//pub type OrderedCardSet54 = OrderedCardSet<PlayingCard54, 54>;
//pub type OrderedCardSet78 = OrderedCardSet<PlayingCard78, 78>;
pub type OrderedCardSet32 = OrderedCardSet<PlayingCard32>;
pub type OrderedCardSet54 = OrderedCardSet<PlayingCard54>;
pub type OrderedCardSet78 = OrderedCardSet<PlayingCard78>;

impl<C> OrderedCardSet<C>
where
	C: Copy + PartialEq,
{
	pub const EMPTY: Self = Self(Vec::new());

	pub fn new(cards: impl IntoIterator<Item = C>) -> Self {
		Self(Vec::from_iter(cards))
	}

	#[inline]
	pub fn iter(&self) -> std::slice::Iter<'_, C> {
		self.0.iter()
	}

	#[inline]
	pub fn contains(&self, card: C) -> bool {
		self.0.contains(&card)
	}

	#[inline]
	pub fn insert(&mut self, card: C) {
		self.0.push(card);
	}
	#[inline]
	pub fn push(&mut self, card: C) {
		self.0.push(card);
	}
	#[inline]
	pub fn pop(&mut self) -> Option<C> {
		self.0.pop()
	}

	#[inline]
	pub fn remove(&mut self, card: C) {
		if let Some(index) = self.0.iter().position(|c| *c == card) {
			self.0.remove(index);
		}
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn draw_random(nb: u8, from: &mut Self) -> Result<Self, String> {
		if from.len() < nb as usize {
			return Err("Not enough cards in the deck".into());
		}

		let mut result = Vec::new();

		for _ in 0..nb {
			let idx = fastrand::usize(..from.0.len());
			result.push(from.0.swap_remove(idx));
		}

		Ok(Self(result))
	}
}
impl<C> From<C> for OrderedCardSet<C> {
	#[inline]
	fn from(card: C) -> Self {
		let mut vec = Vec::new();
		vec.push(card);
		Self(vec)
	}
}
impl<C> Default for OrderedCardSet<C> {
	fn default() -> Self {
		Self(Vec::new())
	}
}
impl<C> IntoIterator for OrderedCardSet<C> {
	type Item = C;
	type IntoIter = std::vec::IntoIter<C>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}
impl<C> FromIterator<C> for OrderedCardSet<C>
where
	C: Copy + PartialEq,
{
	fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
		let mut set = Self::default();

		for card in iter {
			set.insert(card);
		}

		set
	}
}

impl<C> std::ops::BitOrAssign<C> for OrderedCardSet<C> {
	#[inline]
	fn bitor_assign(&mut self, rhs: C) {
		self.0.push(rhs);
	}
}
impl<C> std::ops::BitOr<C> for OrderedCardSet<C>
where
	C: Clone,
{
	type Output = Self;

	fn bitor(mut self, rhs: C) -> Self {
		self.0.push(rhs);
		self
	}
}
impl<C> std::ops::BitOr for OrderedCardSet<C>
where
	C: Clone,
{
	type Output = Self;

	fn bitor(mut self, rhs: Self) -> Self {
		self.0.extend(rhs.0);
		self
	}
}
impl<C> std::ops::BitAnd for OrderedCardSet<C>
where
	C: Clone + PartialEq,
{
	type Output = Self;

	fn bitand(mut self, rhs: Self) -> Self {
		self.0.retain(|c| rhs.0.contains(c));
		self
	}
}
impl<C> std::ops::BitAndAssign for OrderedCardSet<C>
where
	C: PartialEq,
{
	fn bitand_assign(&mut self, rhs: Self) {
		self.0.retain(|c| rhs.0.contains(c));
	}
}
impl<C> std::ops::Sub for OrderedCardSet<C>
where
	C: Clone + PartialEq,
{
	type Output = Self;

	fn sub(mut self, rhs: Self) -> Self {
		self.0.retain(|c| !rhs.0.contains(c));
		self
	}
}
impl<C> std::ops::SubAssign for OrderedCardSet<C>
where
	C: PartialEq,
{
	fn sub_assign(&mut self, rhs: Self) {
		self.0.retain(|c| !rhs.0.contains(c));
	}
}
impl<C> std::fmt::Display for OrderedCardSet<C>
where
	C: std::fmt::Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use std::fmt::Write;

		f.write_char('[')?;

		let mut first = true;

		for c in &self.0 {
			if !first {
				f.write_str(", ")?;
			}

			first = false;
			write!(f, "{c}")?;
		}

		f.write_char(']')
	}
}
impl<C> OrderedCardSet<C>
where
	C: PlayingCard + Copy + PartialEq,
	C::Color: Copy + PartialEq,
{
	pub fn of_color(self, color: C::Color) -> Self {
		self.into_iter().filter(|c| c.color() == color).collect()
	}
}

impl<C> CardSet for OrderedCardSet<C>
where
	C: PlayingCard + Copy + PartialEq,
	C::Color: Copy + PartialEq,
{
	type Card = C;

	const EMPTY: Self = Self::EMPTY;
	//const ALL: Self = Self(Vec::from(*Self::Card::ALL));

	#[inline]
	fn contains(&self, card: Self::Card) -> bool {
		self.0.contains(&card)
	}
	#[inline]
	fn insert(&mut self, card: Self::Card) -> bool {
		self.0.push(card);
		true
	}
	#[inline]
	fn remove(&mut self, card: Self::Card) -> bool {
		if let Some(index) = self.0.iter().position(|c| *c == card) {
			self.0.remove(index);
			true
		} else {
			false
		}
	}
	#[inline]
	fn iter(&self) -> impl Iterator<Item = C> {
		self.iter().cloned()
	}
	#[inline]
	fn len(&self) -> usize {
		self.0.len()
	}
	#[inline]
	fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	#[inline]
	fn draw_random(nb: u8, from: &mut Self) -> Result<Self, String> {
		if from.len() < nb as usize {
			return Err("Not enough cards in the deck".into());
		}

		let mut result = Vec::<C>::new();

		for _ in 0..nb {
			let idx = fastrand::usize(..from.0.len());
			result.push(from.0.swap_remove(idx));
		}

		Ok(Self(result))
	}
	fn all() -> Self {
		let mut vec = Vec::new();
		for c in C::ALL {
			vec.push(*c);
		}
		Self(vec)
	}
}

impl OrderedCardSet32 {
	//pub const ALL: Self = Self(u32::MAX);
	pub fn by_color(color: CardSuit) -> Self {
		match color {
			CardSuit::Spades => Self(Vec::from_iter([
				PlayingCard32::SevenOfSpades,
				PlayingCard32::EightOfSpades,
				PlayingCard32::NineOfSpades,
				PlayingCard32::TenOfSpades,
				PlayingCard32::JackOfSpades,
				PlayingCard32::QueenOfSpades,
				PlayingCard32::KingOfSpades,
				PlayingCard32::AceOfSpades,
			])),
			CardSuit::Hearts => Self(Vec::from_iter([
				PlayingCard32::SevenOfHearts,
				PlayingCard32::EightOfHearts,
				PlayingCard32::NineOfHearts,
				PlayingCard32::TenOfHearts,
				PlayingCard32::JackOfHearts,
				PlayingCard32::QueenOfHearts,
				PlayingCard32::KingOfHearts,
				PlayingCard32::AceOfHearts,
			])),
			CardSuit::Diamonds => Self(Vec::from_iter([
				PlayingCard32::SevenOfDiamonds,
				PlayingCard32::EightOfDiamonds,
				PlayingCard32::NineOfDiamonds,
				PlayingCard32::TenOfDiamonds,
				PlayingCard32::JackOfDiamonds,
				PlayingCard32::QueenOfDiamonds,
				PlayingCard32::KingOfDiamonds,
				PlayingCard32::AceOfDiamonds,
			])),
			CardSuit::Clubs => Self(Vec::from_iter([
				PlayingCard32::SevenOfClubs,
				PlayingCard32::EightOfClubs,
				PlayingCard32::NineOfClubs,
				PlayingCard32::TenOfClubs,
				PlayingCard32::JackOfClubs,
				PlayingCard32::QueenOfClubs,
				PlayingCard32::KingOfClubs,
				PlayingCard32::AceOfClubs,
			])),
			CardSuit::Joker => Self(Vec::new()),
		}
	}
}
impl OrderedCardSet54 {
	//pub const ALL: Self = Self(u32::MAX);
	pub fn by_color(color: CardSuit) -> Self {
		match color {
			CardSuit::Spades => Self(Vec::from_iter([
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
				PlayingCard54::AceOfSpades,
			])),
			CardSuit::Hearts => Self(Vec::from_iter([
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
				PlayingCard54::AceOfHearts,
			])),
			CardSuit::Diamonds => Self(Vec::from_iter([
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
				PlayingCard54::AceOfDiamonds,
			])),
			CardSuit::Clubs => Self(Vec::from_iter([
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
				PlayingCard54::AceOfClubs,
			])),
			CardSuit::Joker => Self(Vec::from_iter([
				PlayingCard54::BlackJoker,
				PlayingCard54::RedJoker,
			])),
		}
	}
}
impl From<UnorderedCardSet32> for OrderedCardSet32 {
	fn from(set: UnorderedCardSet32) -> Self {
		set.into_iter().collect()
	}
}
impl From<UnorderedCardSet54> for OrderedCardSet54 {
	fn from(set: UnorderedCardSet54) -> Self {
		set.into_iter().collect()
	}
}
impl From<UnorderedCardSet78> for OrderedCardSet78 {
	fn from(set: UnorderedCardSet78) -> Self {
		set.into_iter().collect()
	}
}
#[cfg(test)]
mod tests {
	use crate::{
		ordered_card_set::{OrderedCardSet32, OrderedCardSet54},
		playing_cards::CardSet,
	};

	#[test]
	fn test_draw_random_ordered32() {
		let mut deck = OrderedCardSet32::all();
		let player1 = OrderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		let player2 = OrderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		let player3 = OrderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		let player4 = OrderedCardSet32::draw_random(8, &mut deck).expect("Should work");
		assert_eq!(deck.len(), 0);
		assert_eq!(player1.len(), 8);
		assert_eq!(player2.len(), 8);
		assert_eq!(player3.len(), 8);
		assert_eq!(player4.len(), 8);
		assert_eq!(
			player1.clone() | player2.clone() | player3.clone() | player4.clone(),
			OrderedCardSet32::all()
		);
		println!("Player1: {}", player1);
		println!("Player2: {}", player2);
		println!("Player3: {}", player3);
		println!("Player4: {}", player4);
	}
	#[test]
	fn test_draw_random_ordered54() {
		let mut deck = OrderedCardSet54::all();
		let player1 = OrderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		let player2 = OrderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		let player3 = OrderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		let player4 = OrderedCardSet54::draw_random(8, &mut deck).expect("Should work");
		assert_eq!(deck.len(), 0);
		assert_eq!(player1.len(), 8);
		assert_eq!(player2.len(), 8);
		assert_eq!(player3.len(), 8);
		assert_eq!(player4.len(), 8);
		assert_eq!(
			player1.clone() | player2.clone() | player3.clone() | player4.clone(),
			OrderedCardSet54::all()
		);
		println!("Player1: {}", player1);
		println!("Player2: {}", player2);
		println!("Player3: {}", player3);
		println!("Player4: {}", player4);
	}
}
